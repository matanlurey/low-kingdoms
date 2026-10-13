import fs from "node:fs/promises";
import { expect, test } from "@playwright/test";

// Storage faults are browser API failures, not commands or authoritative-state
// hooks. Recovery is exercised through real rendered Bevy controls in every
// configured mouse/touch viewport. Subpath gameplay is covered by battle.spec.
const baseURL = process.env.LOW_KINGDOMS_DEPLOYED_URL
  ? undefined
  : process.env.LOW_KINGDOMS_BASE_URL;
const observedErrors = new WeakMap();

function observeErrors(page) {
  const errors = [];
  observedErrors.set(page, errors);
  page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  page.on("requestfailed", (request) => {
    errors.push(`request: ${request.url()}: ${request.failure()?.errorText}`);
  });
  page.on("response", (response) => {
    if (response.status() >= 400) {
      errors.push(`HTTP ${response.status()}: ${response.url()}`);
    }
  });
}

async function bridge(page) {
  const errors = observedErrors.get(page);
  if (errors?.length) throw new Error(errors.join("\n"));
  return page.evaluate(() => window.__LOW_KINGDOMS_TEST__);
}

function controlFor(state, id) {
  return state.controls.find(
    (control) => control.id === id && control.visible !== false,
  );
}

async function activate(page, id, touch) {
  await expect
    .poll(async () => {
      const state = await bridge(page);
      const control = state && controlFor(state, id);
      const rectangle = control?.visibleRect ?? control;
      return (
        control?.enabled !== false &&
        rectangle?.width >= 44 &&
        rectangle?.height >= 44
      );
    })
    .toBe(true);
  const control = controlFor(await bridge(page), id);
  const rectangle = control.visibleRect ?? control;
  const viewport = page.viewportSize();
  const x = rectangle.x + rectangle.width / 2;
  const y = rectangle.y + rectangle.height / 2;
  expect(x).toBeGreaterThanOrEqual(0);
  expect(x).toBeLessThanOrEqual(viewport.width);
  expect(y).toBeGreaterThanOrEqual(0);
  expect(y).toBeLessThanOrEqual(viewport.height);
  if (touch) await page.touchscreen.tap(x, y);
  else await page.mouse.click(x, y);
}

async function waitForScreen(page, screen) {
  await expect.poll(async () => (await bridge(page))?.screen).toBe(screen);
  await expect.poll(async () => (await bridge(page))?.ready).toBe(true);
}

async function waitForPanel(page, panel) {
  await expect.poll(async () => (await bridge(page))?.battle.panel).toBe(panel);
  await expect.poll(async () => (await bridge(page))?.ready).toBe(true);
}

async function assertVisibleMessage(page, expression) {
  await expect
    .poll(async () =>
      (await bridge(page)).texts.some(
        (entry) =>
          expression.test(entry.text) && entry.width > 0 && entry.height > 0,
      ),
    )
    .toBe(true);
}

async function installStorageFixture(page, mode = "none", corrupt = null) {
  await page.addInitScript(
    ({ initialMode, initialCorrupt }) => {
      if (!["http:", "https:"].includes(location.protocol)) return;
      const key = "low-kingdoms.active.v1";
      const storage = window.localStorage;
      const get = Storage.prototype.getItem;
      const set = Storage.prototype.setItem;
      const fixture = {
        mode: initialMode,
        failures: { read: 0, quota: 0, unavailable: 0 },
      };
      if (initialCorrupt !== null) set.call(storage, key, initialCorrupt);
      Storage.prototype.getItem = function (name) {
        if (name === key && fixture.mode === "read") {
          fixture.failures.read++;
          throw new DOMException(
            "Controlled storage read failure",
            "SecurityError",
          );
        }
        return get.call(this, name);
      };
      Storage.prototype.setItem = function (name, value) {
        if (name === key && fixture.mode === "quota") {
          fixture.failures.quota++;
          throw new DOMException(
            "Controlled storage quota failure",
            "QuotaExceededError",
          );
        }
        return set.call(this, name, value);
      };
      Object.defineProperty(window, "localStorage", {
        configurable: true,
        get() {
          if (fixture.mode === "unavailable") {
            fixture.failures.unavailable++;
            throw new DOMException(
              "Controlled storage access failure",
              "SecurityError",
            );
          }
          return storage;
        },
      });
      // Only opaque integrity metadata crosses into test output. Saved card
      // identities, deck order, and authoritative snapshots remain private.
      fixture.metadata = async () => {
        const value = get.call(storage, key);
        if (value === null) return { present: false };
        const bytes = new TextEncoder().encode(value);
        const digest = await crypto.subtle.digest("SHA-256", bytes);
        let version = null;
        try {
          const envelope = JSON.parse(value);
          if (Number.isInteger(envelope.version)) version = envelope.version;
        } catch {
          // A malformed envelope is an expected fixture, never diagnostic text.
        }
        return {
          present: true,
          bytes: bytes.length,
          version,
          checksum: Array.from(new Uint8Array(digest), (byte) =>
            byte.toString(16).padStart(2, "0"),
          ).join(""),
        };
      };
      Object.defineProperty(window, "__LOW_KINGDOMS_STORAGE_FIXTURE__", {
        value: fixture,
      });
    },
    { initialMode: mode, initialCorrupt: corrupt },
  );
}

async function setFault(page, mode) {
  await page.evaluate((value) => {
    window.__LOW_KINGDOMS_STORAGE_FIXTURE__.mode = value;
  }, mode);
}

async function slotMetadata(page) {
  return page.evaluate(() =>
    window.__LOW_KINGDOMS_STORAGE_FIXTURE__.metadata(),
  );
}

async function failureCount(page, mode) {
  return page.evaluate(
    (value) => window.__LOW_KINGDOMS_STORAGE_FIXTURE__.failures[value],
    mode,
  );
}

async function capture(page, testInfo, name) {
  const filename = testInfo.outputPath(`${name}.png`);
  const image = await page
    .locator("#game-canvas")
    .screenshot({ path: filename, scale: "css" });
  expect(image.length).toBeGreaterThan(1024);
  await testInfo.attach(name, { path: filename, contentType: "image/png" });
}

function keptHandCount(state) {
  return state.battle.history.filter(
    (line) => line === "You kept the opening hand.",
  ).length;
}

test.afterEach(async ({ page }, testInfo) => {
  if (!observedErrors.has(page)) return;
  const errors = observedErrors.get(page);
  const state = await bridge(page);
  const metadata = await slotMetadata(page);
  const filename = testInfo.outputPath("storage-diagnostics.json");
  await fs.writeFile(
    filename,
    JSON.stringify(
      {
        errors,
        slot: metadata,
        screen: state?.screen,
        panel: state?.battle.panel,
        revision: state?.battle.revision,
        phase: state?.battle.view?.phase,
        error: state?.battle.error,
        resumeAvailable: state?.battle.resumeAvailable,
        storageFailures: await page.evaluate(
          () => window.__LOW_KINGDOMS_STORAGE_FIXTURE__.failures,
        ),
      },
      null,
      2,
    ),
  );
  await testInfo.attach("storage-diagnostics", {
    path: filename,
    contentType: "application/json",
  });
  expect(errors).toEqual([]);
});

if (baseURL) {
  test("corrupt save requires visible replacement confirmation and survives cancellation", async ({
    page,
  }, testInfo) => {
    const touch = !!testInfo.project.use.hasTouch;
    observeErrors(page);
    const corrupt = "malformed private-card-marker save {";
    await installStorageFixture(page, "none", corrupt);
    await page.goto(baseURL);
    await waitForScreen(page, "title");
    const original = await slotMetadata(page);
    let state = await bridge(page);
    expect(state.battle.resumeAvailable).toBe(false);
    expect(controlFor(state, "resume")).toBeUndefined();
    expect(state.battle.view).toBeNull();
    await activate(page, "play", touch);
    await waitForPanel(page, "Replace");
    await assertVisibleMessage(page, /local battle save is damaged/i);
    expect(
      (await bridge(page)).texts.some((entry) => entry.text.includes(corrupt)),
    ).toBe(false);
    expect(await slotMetadata(page)).toEqual(original);
    await capture(page, testInfo, "corrupt-save-confirmation");
    await activate(page, "cancel", touch);
    await waitForPanel(page, "None");
    expect(await slotMetadata(page)).toEqual(original);
    expect((await bridge(page)).battle.view).toBeNull();
    await activate(page, "play", touch);
    await waitForPanel(page, "Replace");
    await activate(page, "confirm-replace", touch);
    await waitForScreen(page, "mulligan");
    state = await bridge(page);
    expect(state.battle.error).toBeNull();
    expect(state.battle.view.hand).toHaveLength(6);
    const replacement = await slotMetadata(page);
    expect(replacement.present).toBe(true);
    expect(replacement.version).toBe(1);
    expect(replacement.checksum).not.toBe(original.checksum);
    await capture(page, testInfo, "corrupt-save-replaced");
  });

  test("quota failure preserves the saved slot and retry never duplicates an accepted decision", async ({
    page,
  }, testInfo) => {
    const touch = !!testInfo.project.use.hasTouch;
    observeErrors(page);
    await installStorageFixture(page);
    await page.goto(baseURL);
    await waitForScreen(page, "title");
    await activate(page, "play", touch);
    await waitForScreen(page, "mulligan");
    // The AI submits its single setup decision first; the human decision is now
    // the only pending command. No private AI cards or decision fields are read.
    await expect.poll(async () => (await bridge(page)).battle.revision).toBe(1);
    const before = await bridge(page);
    const humanHand = before.battle.view.hand;
    expect(before.battle.view.decision_submitted).toBe(false);
    const savedBefore = await slotMetadata(page);
    await setFault(page, "quota");
    await activate(page, "keep-hand", touch);
    await waitForScreen(page, "battle");
    await waitForPanel(page, "Error");
    await assertVisibleMessage(page, /local storage is full/i);
    const failed = await bridge(page);
    expect(failed.battle.revision).toBe(before.battle.revision + 1);
    expect(failed.battle.view.phase).toBe("InitialResourcing");
    expect(failed.battle.view.hand).toEqual(humanHand);
    expect(keptHandCount(failed)).toBe(1);
    expect(await slotMetadata(page)).toEqual(savedBefore);
    await capture(page, testInfo, "quota-failure-paused");
    const quotaFailures = await failureCount(page, "quota");
    await activate(page, "retry", touch);
    await expect
      .poll(() => failureCount(page, "quota"))
      .toBeGreaterThan(quotaFailures);
    await waitForPanel(page, "Error");
    const failedAgain = await bridge(page);
    expect(failedAgain.battle.revision).toBe(failed.battle.revision);
    expect(failedAgain.battle.view).toEqual(failed.battle.view);
    expect(keptHandCount(failedAgain)).toBe(1);
    expect(await slotMetadata(page)).toEqual(savedBefore);
    await setFault(page, "none");
    await activate(page, "retry", touch);
    await expect.poll(async () => (await bridge(page)).battle.error).toBeNull();
    await waitForPanel(page, "None");
    let recovered = await bridge(page);
    expect(keptHandCount(recovered)).toBe(1);
    expect(recovered.battle.view.hand).toEqual(humanHand);
    expect(recovered.battle.revision).toBeGreaterThanOrEqual(
      failed.battle.revision,
    );
    expect(recovered.battle.revision).toBeLessThanOrEqual(
      failed.battle.revision + 1,
    );
    expect((await slotMetadata(page)).checksum).not.toBe(savedBefore.checksum);
    await page.reload();
    await waitForScreen(page, "title");
    expect((await bridge(page)).battle.resumeAvailable).toBe(true);
    await activate(page, "resume", touch);
    await waitForScreen(page, "battle");
    recovered = await bridge(page);
    expect(recovered.battle.error).toBeNull();
    expect(recovered.battle.view.phase).toBe("InitialResourcing");
    expect(recovered.battle.view.hand).toEqual(humanHand);
    expect(keptHandCount(recovered)).toBe(1);
    await capture(page, testInfo, "quota-recovered-resumed");
  });

  for (const mode of ["read", "unavailable"]) {
    test(`${mode} storage failure recovers through the rendered Retry control`, async ({
      page,
    }, testInfo) => {
      const touch = !!testInfo.project.use.hasTouch;
      observeErrors(page);
      await installStorageFixture(page, mode);
      await page.goto(baseURL);
      await waitForScreen(page, "title");
      await waitForPanel(page, "Error");
      await assertVisibleMessage(
        page,
        mode === "read"
          ? /could not read the saved battle/i
          : /local saves are unavailable/i,
      );
      const blocked = await bridge(page);
      expect(blocked.battle.view).toBeNull();
      expect(blocked.battle.revision).toBe(0);
      expect(blocked.battle.resumeAvailable).toBe(false);
      expect(controlFor(blocked, "resume")).toBeUndefined();
      expect(await slotMetadata(page)).toEqual({ present: false });
      await capture(page, testInfo, `${mode}-storage-error`);
      const failures = await failureCount(page, mode);
      await activate(page, "retry", touch);
      await expect
        .poll(() => failureCount(page, mode))
        .toBeGreaterThan(failures);
      await waitForPanel(page, "Error");
      expect((await bridge(page)).battle.view).toBeNull();
      expect((await bridge(page)).battle.revision).toBe(0);
      await setFault(page, "none");
      await activate(page, "retry", touch);
      await expect
        .poll(async () => (await bridge(page)).battle.error)
        .toBeNull();
      await waitForPanel(page, "None");
      const recovered = await bridge(page);
      expect(recovered.battle.view).toBeNull();
      expect(recovered.battle.revision).toBe(0);
      expect(recovered.battle.resumeAvailable).toBe(false);
      expect(
        recovered.controls.filter((control) => control.id === "play"),
      ).toHaveLength(1);
      expect(await slotMetadata(page)).toEqual({ present: false });
      await activate(page, "play", touch);
      await waitForScreen(page, "mulligan");
      expect((await bridge(page)).battle.view.hand).toHaveLength(6);
      expect((await slotMetadata(page)).version).toBe(1);
      await capture(page, testInfo, `${mode}-storage-recovered`);
    });
  }
}
