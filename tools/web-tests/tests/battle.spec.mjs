import fs from "node:fs/promises";
import { expect, test } from "@playwright/test";

// These tests observe the live, read-only player projection. Every match change
// is made through the same rendered controls a human uses, never a test command.
const routes = process.env.LOW_KINGDOMS_DEPLOYED_URL
  ? []
  : [
      ["root", process.env.LOW_KINGDOMS_BASE_URL],
      ["subpath", process.env.LOW_KINGDOMS_SUBPATH_URL],
    ];
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
  return errors;
}

async function bridge(page) {
  const errors = observedErrors.get(page);
  if (errors?.length) throw new Error(errors.join("\n"));
  return page.evaluate(() => window.__LOW_KINGDOMS_TEST__);
}

async function waitForScreen(page, screen) {
  await expect.poll(async () => (await bridge(page))?.screen).toBe(screen);
  await expect.poll(async () => (await bridge(page))?.ready).toBe(true);
}

async function waitForPanel(page, panel) {
  await expect.poll(async () => (await bridge(page)).battle.panel).toBe(panel);
}

function visible(control, viewport) {
  const rectangle = control?.visibleRect ?? control;
  return (
    control &&
    control.visible !== false &&
    rectangle.width >= 44 &&
    rectangle.height >= 44 &&
    (!control.id.startsWith("card-") ||
      (rectangle.width >= control.width - 0.5 &&
        rectangle.height >= control.height - 0.5)) &&
    rectangle.x + rectangle.width / 2 >= 0 &&
    rectangle.x + rectangle.width / 2 <= viewport.width &&
    rectangle.y + rectangle.height / 2 >= 0 &&
    rectangle.y + rectangle.height / 2 <= viewport.height
  );
}

function controlFor(state, id) {
  return state.controls
    .filter((control) => control.id === id && control.visible !== false)
    .sort(
      (left, right) =>
        (right.stackIndex ?? 0) - (left.stackIndex ?? 0) ||
        right.width - left.width,
    )[0];
}

async function activate(page, id, touch, quick = false) {
  await expect
    .poll(async () => {
      const control = controlFor(await bridge(page), id);
      return visible(control, page.viewportSize()) && control.enabled !== false;
    })
    .toBe(true);
  const control = controlFor(await bridge(page), id);
  expect(control.height).toBeGreaterThanOrEqual(44);
  const rectangle = control.visibleRect ?? control;
  const x = rectangle.x + rectangle.width / 2;
  const y = rectangle.y + rectangle.height / 2;
  // Canvas controls consume press and release on separate render frames. Keep
  // a real pointer down across a frame, as a physical click or tap does.
  if (quick) {
    if (touch) await page.touchscreen.tap(x, y);
    else await page.mouse.click(x, y);
  } else if (touch) {
    const session = await page.context().newCDPSession(page);
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [{ x, y, id: 1 }],
    });
    await page.evaluate(
      () =>
        new Promise((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(resolve)),
        ),
    );
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await session.detach();
  } else {
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.evaluate(
      () =>
        new Promise((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(resolve)),
        ),
    );
    await page.mouse.up();
  }
  await page.evaluate(() => new Promise(requestAnimationFrame));
}

async function assertLayout(page) {
  const state = await bridge(page);
  const viewport = page.viewportSize();
  expect(state.viewport.width).toBeCloseTo(viewport.width, 0);
  expect(state.viewport.height).toBeCloseTo(viewport.height, 0);
  expect(state.controls.map((control) => control.id)).not.toContain("quit");
  for (const control of state.controls.filter((item) =>
    visible(item, viewport),
  )) {
    expect(control.height).toBeGreaterThanOrEqual(44);
  }
  for (const id of ["pass", "more", "confirm-resources"]) {
    const control = controlFor(state, id);
    if (!control) continue;
    expect(control.x).toBeGreaterThanOrEqual(-0.5);
    expect(control.y).toBeGreaterThanOrEqual(-0.5);
    expect(control.x + control.width).toBeLessThanOrEqual(viewport.width + 0.5);
    expect(control.y + control.height).toBeLessThanOrEqual(
      viewport.height + 0.5,
    );
  }
  const canvas = await page.evaluate(() => {
    const node = document.querySelector("#game-canvas");
    const rectangle = node.getBoundingClientRect();
    return {
      width: rectangle.width,
      height: rectangle.height,
      physicalWidth: node.width,
      physicalHeight: node.height,
      dpr: window.devicePixelRatio,
      scrollWidth: document.documentElement.scrollWidth,
      scrollHeight: document.documentElement.scrollHeight,
    };
  });
  expect(canvas.width).toBe(viewport.width);
  expect(canvas.height).toBe(viewport.height);
  expect(canvas.physicalWidth).toBeGreaterThan(0);
  expect(canvas.physicalHeight).toBeGreaterThan(0);
  expect(
    Math.abs(canvas.physicalWidth - viewport.width * canvas.dpr),
  ).toBeLessThanOrEqual(2);
  expect(
    Math.abs(canvas.physicalHeight - viewport.height * canvas.dpr),
  ).toBeLessThanOrEqual(2);
  expect(canvas.scrollWidth).toBeLessThanOrEqual(viewport.width);
  expect(canvas.scrollHeight).toBeLessThanOrEqual(viewport.height);
  expect(state.viewport.scaleFactor).toBeCloseTo(canvas.dpr, 2);
  if (state.battle?.view) {
    const view = state.battle.view;
    expect(view.player).toBe("Zero");
    expect(view.players[1].hand).toBeUndefined();
    expect(view.players[1].deck).toBeUndefined();
    expect(view.players[1].resources.cards).toBeUndefined();
    expect(view.deck).toBeUndefined();
    expect(state.battle.snapshot).toBeUndefined();
    expect(state.battle.error).toBeNull();
    for (const unit of view.players.flatMap((player) => player.units)) {
      if (!visible(controlFor(state, `card-${unit.card.id}`), viewport))
        continue;
      const definition = state.battle.catalog.find(
        (card) => card.id === unit.card.definition,
      );
      const remaining = Math.max(0, unit.health - unit.damage);
      expect(
        state.texts.some(
          (entry) =>
            entry.text.includes(definition.name) &&
            entry.text.includes(`${unit.power} / ${remaining}`) &&
            entry.text.includes(`Damage: ${unit.damage}`) &&
            entry.text.includes(unit.ready ? "Ready" : "Exhausted"),
        ),
      ).toBe(true);
    }
  }
  return state;
}

async function capture(page, testInfo, name) {
  const filename = testInfo.outputPath(`${name}.png`);
  const screenshot = await page
    .locator("#game-canvas")
    .screenshot({ path: filename, scale: "css" });
  const pixels = await page.evaluate(async (encoded) => {
    const image = new Image();
    image.src = `data:image/png;base64,${encoded}`;
    await image.decode();
    const surface = new OffscreenCanvas(image.width, image.height);
    const context = surface.getContext("2d");
    context.drawImage(image, 0, 0);
    const data = context.getImageData(0, 0, image.width, image.height).data;
    const colors = new Set();
    let light = 0;
    for (let index = 0; index < data.length; index += 4) {
      colors.add(`${data[index]},${data[index + 1]},${data[index + 2]}`);
      if (data[index] + data[index + 1] + data[index + 2] > 450) light++;
    }
    return { colors: colors.size, light };
  }, screenshot.toString("base64"));
  expect(pixels.colors).toBeGreaterThan(20);
  expect(pixels.light).toBeGreaterThan(200);
  await testInfo.attach(name, { path: filename, contentType: "image/png" });
  const diagnostic = testInfo.outputPath(`${name}.json`);
  await fs.writeFile(diagnostic, JSON.stringify(await bridge(page), null, 2));
  await testInfo.attach(`${name}-state`, {
    path: diagnostic,
    contentType: "application/json",
  });
}

async function scrollGesture(page, rectangle, direction, touch) {
  const viewport = page.viewportSize();
  const left = Math.max(18, rectangle.x + 18);
  const right = Math.min(
    viewport.width - 18,
    rectangle.x + rectangle.width - 18,
  );
  const top = Math.max(18, rectangle.y + 18);
  const bottom = Math.min(
    viewport.height - 18,
    rectangle.y + rectangle.height - 18,
  );
  const horizontal = rectangle.horizontal !== false;
  const x = (left + right) / 2;
  const y = (top + bottom) / 2;
  if (touch) {
    const session = await page.context().newCDPSession(page);
    const start = horizontal
      ? { x: direction > 0 ? right : left, y }
      : { x, y: direction > 0 ? bottom : top };
    const end = horizontal
      ? { x: direction > 0 ? left : right, y }
      : { x, y: direction > 0 ? top : bottom };
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [start],
    });
    for (let step = 1; step <= 10; step++) {
      await session.send("Input.dispatchTouchEvent", {
        type: "touchMove",
        touchPoints: [
          {
            x: start.x + ((end.x - start.x) * step) / 10,
            y: start.y + ((end.y - start.y) * step) / 10,
          },
        ],
      });
      await page.evaluate(() => new Promise(requestAnimationFrame));
    }
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await session.detach();
  } else {
    await page.mouse.move(x, y);
    await page.mouse.wheel(
      horizontal ? direction * 360 : 0,
      horizontal ? 0 : direction * 360,
    );
  }
  await page.evaluate(() => new Promise(requestAnimationFrame));
}

function rowRectangle(state, ids, viewport) {
  const controls = ids
    .map((id) => controlFor(state, id))
    .filter((control) => visible(control, viewport));
  expect(controls.length).toBeGreaterThan(0);
  const top = Math.min(...controls.map((control) => control.y));
  const bottom = Math.max(
    ...controls.map((control) => control.y + control.height),
  );
  const pane = state.battle.scrollRegions
    .filter(
      (region) =>
        region.visible !== false &&
        region.x <= controls[0].x + controls[0].width / 2 &&
        region.x + region.width >= controls[0].x + controls[0].width / 2 &&
        region.y <= top + 1 &&
        region.y + region.height >= bottom - 1,
    )
    .sort((a, b) => a.width * a.height - b.width * b.height)[0];
  if (pane) return pane;
  return { x: 0, y: top, width: viewport.width, height: bottom - top };
}

async function revealCard(page, id, rowIds, touch) {
  let scrolled = false;
  for (const direction of [1, -1]) {
    for (let attempt = 0; attempt < 8; attempt++) {
      const state = await bridge(page);
      if (visible(controlFor(state, `card-${id}`), page.viewportSize()))
        return scrolled;
      await scrollGesture(
        page,
        rowRectangle(state, rowIds, page.viewportSize()),
        direction,
        touch,
      );
      scrolled = true;
    }
  }
  throw new Error(`Card ${id} was not reachable by real scrolling.`);
}

async function selectCard(page, id, cards, touch) {
  const rowIds = cards.map((card) => `card-${card.id ?? card.card.id}`);
  await revealCard(page, id, rowIds, touch);
  await activate(page, `card-${id}`, touch);
}

async function showHand(page, touch) {
  const state = await bridge(page);
  const visibleHand = state.battle.view.hand.some((card) =>
    visible(controlFor(state, `card-${card.id}`), page.viewportSize()),
  );
  if (!visibleHand && controlFor(state, "hand"))
    await activate(page, "hand", touch);
}

async function closePanel(page, touch) {
  if (controlFor(await bridge(page), "close")) {
    await activate(page, "close", touch);
    await waitForPanel(page, "None");
  }
}

async function setSpeed(page, speed, touch) {
  let state = await bridge(page);
  if (!controlFor(state, `speed-${speed}`)) await activate(page, "more", touch);
  await activate(page, `speed-${speed}`, touch);
  await expect
    .poll(async () => (await bridge(page)).battle.speed.toLowerCase())
    .toBe(speed);
  await waitForPanel(page, "None");
}

async function waitForDecision(page) {
  await expect
    .poll(
      async () => {
        const state = await bridge(page);
        if (state.screen === "results") return true;
        if (state.battle?.error) throw new Error(state.battle.error);
        return state.battle?.legal?.length > 0;
      },
      { timeout: 60_000, intervals: [50, 100, 250] },
    )
    .toBe(true);
  return bridge(page);
}

async function submitResources(page, touch) {
  const state = await waitForDecision(page);
  const choices = state.battle.legal;
  const initial = choices.find((command) => command.ChooseInitialResources);
  const subsequent = choices.find((command) => command.ChooseRegroupResource);
  expect(initial || subsequent).toBeDefined();
  // Preserve early units, then stop growing at six resources. Statistics are
  // public definitions; legality still comes solely from the engine's list.
  const catalog = state.battle.catalog;
  const definition = (card) =>
    catalog.find((item) => item.id === card.definition);
  const ordered = [...state.battle.view.hand].sort(
    (a, b) =>
      definition(b).cost - definition(a).cost ||
      definition(a).health - definition(b).health ||
      a.id - b.id,
  );
  const count = initial ? 2 : state.battle.view.resources.length < 6 ? 1 : 0;
  await showHand(page, touch);
  for (const card of ordered.slice(0, count)) {
    await selectCard(page, card.id, state.battle.view.hand, touch);
    await expect
      .poll(async () => (await bridge(page)).battle.resources.includes(card.id))
      .toBe(true);
  }
  const before = (await bridge(page)).battle.revision;
  await activate(page, "confirm-resources", touch);
  await expect
    .poll(async () => (await bridge(page)).battle.revision)
    .toBeGreaterThan(before);
}

async function playUnit(page, command, touch) {
  const state = await bridge(page);
  await showHand(page, touch);
  await selectCard(page, command.PlayUnit.card, state.battle.view.hand, touch);
  await expect
    .poll(async () => (await bridge(page)).battle.selected)
    .toBe(command.PlayUnit.card);
  await activate(page, "play-unit", touch);
}

async function playAndPauseAi(page, command, touch) {
  await setSpeed(page, "normal", touch);
  const background = await page.context().newPage();
  // Playwright normally forces every page to appear focused and active.
  // Disable that driver override so real browser tab focus/visibility governs
  // the production pause policy. No document or game state is substituted.
  for (const tab of [page, background]) {
    const session = await page.context().newCDPSession(tab);
    await session.send("Emulation.setFocusEmulationEnabled", {
      enabled: false,
    });
    await session.detach();
  }
  await page.bringToFront();
  const revision = (await bridge(page)).battle.revision;
  await playUnit(page, command, touch);
  await expect
    .poll(async () => (await bridge(page)).battle.revision, {
      intervals: [10, 25, 50],
    })
    .toBeGreaterThan(revision);
  const pending = (await bridge(page)).battle;
  expect(pending.view.active_player).toBe("One");
  await background.bringToFront();
  await expect
    .poll(() =>
      page.evaluate(
        () => document.visibilityState !== "visible" || !document.hasFocus(),
      ),
    )
    .toBe(true);
  // Keep an actual background tab inactive beyond two Normal AI delays. This
  // checks browser scheduling; no document visibility or game field is edited.
  await background.waitForTimeout(1_600);
  expect((await bridge(page)).battle.revision).toBe(pending.revision);
  // Reload the inactive page while the persisted AI turn is still pending.
  // Resume then permits exactly the next AI command, with no duplicate play.
  await page.reload();
  await background.close();
  await page.bringToFront();
  await expect
    .poll(() =>
      page.evaluate(
        () => document.visibilityState === "visible" && document.hasFocus(),
      ),
    )
    .toBe(true);
  await waitForScreen(page, "title");
  await activate(page, "resume", touch);
  await waitForScreen(page, "battle");
  const restored = (await bridge(page)).battle;
  expect(restored.revision).toBe(pending.revision);
  expect(restored.view).toEqual(pending.view);
  await waitForDecision(page);
  expect((await bridge(page)).battle.revision).toBe(pending.revision + 1);
  await setSpeed(page, "instant", touch);
}

function commandMessage(command, state) {
  if (command === "Pass") return "You passed.";
  if (command === "ClaimInitiative") return "You claimed initiative.";
  const cards = [
    ...state.battle.view.hand,
    ...state.battle.view.players.flatMap((player) =>
      player.units.map((unit) => unit.card),
    ),
  ];
  const name = (id) => {
    const card = cards.find((item) => item.id === id);
    expect(card, `public card ${id}`).toBeDefined();
    return state.battle.catalog.find(
      (definition) => definition.id === card.definition,
    ).name;
  };
  if (command.PlayUnit)
    return `You deployed ${name(command.PlayUnit.card)} exhausted.`;
  const { attacker, target } = command.Attack;
  const destination =
    target.Unit !== undefined
      ? `Opponent's ${name(target.Unit)}`
      : "Opponent's stronghold";
  return `Your ${name(attacker)} attacked ${destination}.`;
}

function newMessages(previous, current) {
  // The presentation history is bounded. Find the retained overlap rather than
  // assuming its length always grows, and require a newly accepted human event.
  for (
    let retained = Math.min(previous.length, current.length);
    retained >= 0;
    retained--
  ) {
    if (
      previous
        .slice(previous.length - retained)
        .every((line, index) => line === current[index])
    ) {
      return current.slice(retained);
    }
  }
  return current;
}

async function attack(page, command, touch, testInfo) {
  const state = await bridge(page);
  await closePanel(page, touch);
  await selectCard(
    page,
    command.Attack.attacker,
    state.battle.view.players[0].units,
    touch,
  );
  await expect
    .poll(async () => (await bridge(page)).battle.selected)
    .toBe(command.Attack.attacker);
  await activate(page, "attack-unit", touch);
  await expect
    .poll(async () => (await bridge(page)).battle.attacker)
    .toBe(command.Attack.attacker);
  if (testInfo) {
    const revision = (await bridge(page)).battle.revision;
    await capture(page, testInfo, "attack-targeting");
    await page.keyboard.press("Escape");
    await expect
      .poll(async () => (await bridge(page)).battle.attacker)
      .toBeNull();
    expect((await bridge(page)).battle.revision).toBe(revision);
    await selectCard(
      page,
      command.Attack.attacker,
      state.battle.view.players[0].units,
      touch,
    );
    await activate(page, "attack-unit", touch);
    await expect
      .poll(async () => (await bridge(page)).battle.attacker)
      .toBe(command.Attack.attacker);
  }
  if (command.Attack.target.Unit !== undefined) {
    const targets = (await bridge(page)).battle.view.players[1].units;
    const scrolled = await revealCard(
      page,
      command.Attack.target.Unit,
      targets.map((unit) => `card-${unit.card.id}`),
      touch,
    );
    expect((await bridge(page)).battle.attacker).toBe(command.Attack.attacker);
    if (scrolled && testInfo)
      await capture(page, testInfo, "offscreen-attack-target");
    await activate(page, `card-${command.Attack.target.Unit}`, touch);
    return scrolled;
  } else {
    await activate(page, "enemy-stronghold", touch);
  }
  return false;
}

async function claimInitiative(page, touch) {
  await activate(page, "more", touch);
  await activate(page, "claim-initiative", touch);
}

async function refreshAndResume(page, touch, testInfo) {
  await setSpeed(page, "normal", touch);
  const before = (await bridge(page)).battle.view;
  const humanIds = before.hand.map((card) => card.id);
  await page.reload();
  await waitForScreen(page, "title");
  expect(controlFor(await bridge(page), "resume")).toBeDefined();
  await activate(page, "resume", touch);
  await waitForScreen(page, "battle");
  const restored = (await bridge(page)).battle.view;
  // AI may make the next legitimate action; restoring must retain the human's
  // cards, already committed resources, and round rather than restart setup.
  expect(restored.hand.map((card) => card.id)).toEqual(humanIds);
  expect(restored.resources).toEqual(before.resources);
  expect(restored.round).toBe(before.round);
  await capture(page, testInfo, "resumed-battle");
  await setSpeed(page, "instant", touch);
}

test.afterEach(async ({ page }, testInfo) => {
  const errors = observedErrors.get(page) ?? [];
  const filename = testInfo.outputPath("battle-browser-errors.json");
  await fs.writeFile(filename, JSON.stringify(errors, null, 2));
  await testInfo.attach("battle-browser-errors", {
    path: filename,
    contentType: "application/json",
  });
  expect(errors).toEqual([]);
});

for (const [route, url] of routes) {
  test(`${route}: Play, mulligan, save replacement, and resume`, async ({
    page,
  }, testInfo) => {
    observeErrors(page);
    const touch = Boolean(testInfo.project.use.hasTouch);
    await page.goto(url);
    await waitForScreen(page, "title");
    await activate(page, "play", touch, true);
    await waitForScreen(page, "mulligan");
    expect((await bridge(page)).battle.view.hand).toHaveLength(6);
    if (testInfo.project.name === "phone-minimum") {
      const original = page.viewportSize();
      const hand = (await bridge(page)).battle.view.hand;
      const revision = (await bridge(page)).battle.revision;
      await page.setViewportSize({ width: 667, height: 375 });
      await expect
        .poll(async () => (await bridge(page)).viewport.width)
        .toBe(667);
      for (const card of [hand.at(-1), hand[0]]) {
        await selectCard(page, card.id, hand, touch);
        await waitForPanel(page, "Inspect");
        expect((await bridge(page)).battle.selected).toBe(card.id);
        expect((await bridge(page)).battle.revision).toBe(revision);
        await closePanel(page, touch);
      }
      await capture(page, testInfo, `${route}-rotated-mulligan`);
      await page.setViewportSize(original);
      await expect
        .poll(async () => (await bridge(page)).viewport.width)
        .toBe(original.width);
    }
    await activate(page, "mulligan", touch);
    await waitForScreen(page, "battle");
    await expect
      .poll(async () => (await bridge(page)).battle.view.phase)
      .toBe("InitialResourcing");
    await assertLayout(page);
    await capture(page, testInfo, `${route}-initial-resourcing`);
    const hand = (await bridge(page)).battle.view.hand;
    await page.reload();
    await waitForScreen(page, "title");
    await activate(page, "play", touch);
    await waitForPanel(page, "Replace");
    expect(controlFor(await bridge(page), "confirm-replace")).toBeDefined();
    await activate(page, "cancel", touch);
    await waitForPanel(page, "None");
    await activate(page, "resume", touch);
    await waitForScreen(page, "battle");
    expect((await bridge(page)).battle.view.hand).toEqual(hand);
    await activate(page, "more", touch);
    await activate(page, "main-menu", touch);
    await waitForScreen(page, "title");
    await activate(page, "play", touch);
    await waitForPanel(page, "Replace");
    await activate(page, "confirm-replace", touch);
    await waitForScreen(page, "mulligan");
    await assertLayout(page);
  });
}

if (routes.length) {
  test("complete battle through rendered controls, resume, and restart", async ({
    page,
  }, testInfo) => {
    test.setTimeout(600_000);
    observeErrors(page);
    const touch = Boolean(testInfo.project.use.hasTouch);
    await page.goto(process.env.LOW_KINGDOMS_BASE_URL);
    await waitForScreen(page, "title");
    await activate(page, "play", touch);
    await waitForScreen(page, "mulligan");
    const openingHand = (await bridge(page)).battle.view.hand;
    await capture(page, testInfo, "mulligan");
    await activate(page, "keep-hand", touch);
    await waitForScreen(page, "battle");
    await submitResources(page, touch);
    await waitForDecision(page);
    await assertLayout(page);
    await capture(page, testInfo, "battle-initial");
    expect(
      await page.evaluate(() =>
        Object.isFrozen(window.__LOW_KINGDOMS_TEST__.battle.view),
      ),
    ).toBe(true);
    await setSpeed(page, "fast", touch);
    await setSpeed(page, "normal", touch);
    await setSpeed(page, "instant", touch);
    const idleRevision = (await bridge(page)).battle.revision;
    await activate(page, "hand", touch);
    await waitForPanel(page, "Hand");
    await capture(page, testInfo, "hand-tray");
    await closePanel(page, touch);
    expect((await bridge(page)).battle.revision).toBe(idleRevision);
    await activate(page, "resources", touch);
    await waitForPanel(page, "Resources");
    await capture(page, testInfo, "resource-inspection");
    const resource = (await bridge(page)).battle.view.resources[0].card;
    await selectCard(page, resource.id, [resource], touch);
    await waitForPanel(page, "Inspect");
    expect((await bridge(page)).battle.selected).toBe(resource.id);
    await capture(page, testInfo, "resource-card-inspection");
    await closePanel(page, touch);
    await activate(page, "log", touch);
    await waitForPanel(page, "Log");
    await capture(page, testInfo, "event-log");
    await closePanel(page, touch);

    const exercised = {
      deployed: false,
      pass: false,
      claim: false,
      unitAttack: false,
      strongholdAttack: false,
      regroup: false,
      resumed: false,
    };
    let inspected = false;
    let resized = false;
    let targetScrolled = false;
    let observedOpponent = false;
    for (let action = 0; action < 220; action++) {
      const state = await waitForDecision(page);
      if (state.screen === "results") break;
      const { view, legal } = state.battle;
      observedOpponent ||=
        view.players[1].units.length > 0 || view.players[1].discard.length > 0;
      if (view.phase === "Regroup") {
        if (!exercised.regroup) await capture(page, testInfo, "regroup");
        exercised.regroup = true;
        await submitResources(page, touch);
        continue;
      }
      expect(view.phase).toBe("Action");
      expect(view.active_player).toBe("Zero");
      if (!exercised.resumed && view.round >= 2) {
        await refreshAndResume(page, touch, testInfo);
        exercised.resumed = true;
        continue;
      }
      if (!resized && view.round >= 2) {
        const original = page.viewportSize();
        const resizedViewport =
          original.width > 900
            ? { width: 900, height: 550 }
            : { width: original.height, height: original.width };
        await page.setViewportSize(resizedViewport);
        await expect
          .poll(async () => (await bridge(page)).viewport.width)
          .toBeCloseTo(resizedViewport.width, 0);
        await assertLayout(page);
        await capture(page, testInfo, "battle-resized");
        await page.setViewportSize(original);
        await expect
          .poll(async () => (await bridge(page)).viewport.width)
          .toBeCloseTo(original.width, 0);
        await assertLayout(page);
        if (testInfo.project.use.deviceScaleFactor === 2) {
          const session = await page.context().newCDPSession(page);
          await session.send("Emulation.setDeviceMetricsOverride", {
            ...original,
            deviceScaleFactor: 1,
            mobile: true,
          });
          await expect
            .poll(async () => (await bridge(page)).viewport.scaleFactor)
            .toBe(1);
          await assertLayout(page);
          await capture(page, testInfo, "battle-changed-dpi");
          await session.send("Emulation.setDeviceMetricsOverride", {
            ...original,
            deviceScaleFactor: 2,
            mobile: true,
          });
          await expect
            .poll(async () => (await bridge(page)).viewport.scaleFactor)
            .toBe(2);
          await session.detach();
        }
        resized = true;
        continue;
      }
      const plays = legal.filter((command) => command.PlayUnit);
      const attacks = legal.filter((command) => command.Attack);
      const unitAttacks = attacks.filter(
        (command) => command.Attack.target.Unit !== undefined,
      );
      const strongholdAttacks = attacks.filter(
        (command) => command.Attack.target.Stronghold !== undefined,
      );
      const before = state.battle.revision;
      const previousHistory = state.battle.history;
      let submitted;
      if (!exercised.pass && exercised.deployed && legal.includes("Pass")) {
        submitted = "Pass";
        await activate(page, "hand", touch);
        await waitForPanel(page, "Hand");
        // The bottom hand tray must leave the battle footer usable. A quick
        // click/tap also exercises same-frame input without a held press.
        expect(
          visible(controlFor(await bridge(page), "pass"), page.viewportSize()),
        ).toBe(true);
        await capture(page, testInfo, "hand-tray-before-pass");
        await activate(page, "pass", touch, true);
        exercised.pass = true;
      } else if (
        !exercised.claim &&
        exercised.pass &&
        view.round >= 2 &&
        legal.includes("ClaimInitiative")
      ) {
        submitted = "ClaimInitiative";
        await claimInitiative(page, touch);
        exercised.claim = true;
      } else if (
        !exercised.unitAttack &&
        unitAttacks.length &&
        view.players[1].units.length >= 3
      ) {
        submitted = unitAttacks.at(-1);
        // Seed 42 reaches three opponents in round three. At phone widths the
        // last unit is clipped; targeting it requires a real horizontal swipe.
        targetScrolled = await attack(
          page,
          unitAttacks.at(-1),
          touch,
          testInfo,
        );
        exercised.unitAttack = true;
        await capture(page, testInfo, "unit-combat");
      } else if (plays.length) {
        submitted = plays[0];
        const card = plays[0].PlayUnit.card;
        if (!inspected) {
          await showHand(page, touch);
          await selectCard(page, card, view.hand, touch);
          await capture(page, testInfo, "card-inspection");
          expect((await bridge(page)).battle.revision).toBe(before);
          await closePanel(page, touch);
          inspected = true;
        }
        if (!exercised.deployed) {
          await playUnit(page, plays[0], touch);
          await expect
            .poll(async () => (await bridge(page)).battle.revision)
            .toBeGreaterThan(before);
          const deployed = (
            await bridge(page)
          ).battle.view.players[0].units.find((unit) => unit.card.id === card);
          expect(deployed.ready).toBe(false);
          await capture(page, testInfo, "deployed-exhausted");
        } else {
          await playUnit(page, plays[0], touch);
        }
        exercised.deployed = true;
      } else if (strongholdAttacks.length) {
        submitted = strongholdAttacks[0];
        await attack(page, strongholdAttacks[0], touch);
        exercised.strongholdAttack = true;
      } else if (unitAttacks.length) {
        submitted = unitAttacks[0];
        await attack(page, unitAttacks[0], touch);
      } else if (legal.includes("ClaimInitiative")) {
        submitted = "ClaimInitiative";
        await claimInitiative(page, touch);
        exercised.claim = true;
      } else {
        submitted = "Pass";
        expect(legal).toContain("Pass");
        await activate(page, "pass", touch);
        exercised.pass = true;
      }
      await expect
        .poll(async () => (await bridge(page)).battle.revision)
        .toBeGreaterThan(before);
      await expect
        .poll(async () =>
          newMessages(previousHistory, (await bridge(page)).battle.history),
        )
        .toContain(commandMessage(submitted, state));
      await assertLayout(page);
    }
    await waitForScreen(page, "results");
    const results = await assertLayout(page);
    expect(results.battle.view.outcome).not.toBeNull();
    expect(
      results.texts.some((entry) => /Victory|Defeat|Draw/.test(entry.text)),
    ).toBe(true);
    expect(observedOpponent).toBe(true);
    expect(
      results.battle.history.some((line) =>
        line.startsWith("Opponent deployed"),
      ),
    ).toBe(true);
    expect(
      results.battle.history.some((line) =>
        /^Opponent's .+ attacked /.test(line),
      ),
    ).toBe(true);
    expect(results.battle.resumeAvailable).toBe(false);
    expect(inspected).toBe(true);
    expect(resized).toBe(true);
    if (touch && testInfo.project.use.viewport.width < 600)
      expect(targetScrolled).toBe(true);
    for (const [name, completed] of Object.entries(exercised))
      expect(completed, name).toBe(true);
    await capture(page, testInfo, "results");
    await activate(page, "play-again", touch);
    await waitForScreen(page, "mulligan");
    expect((await bridge(page)).battle.view.hand).toHaveLength(6);
    expect((await bridge(page)).battle.view.round).toBe(1);
    expect((await bridge(page)).battle.view.hand).not.toEqual(openingHand);
    await activate(page, "keep-hand", touch);
    await waitForScreen(page, "battle");
    await activate(page, "more", touch);
    await activate(page, "main-menu", touch);
    await waitForScreen(page, "title");
    expect(controlFor(await bridge(page), "resume")).toBeDefined();
    await capture(page, testInfo, "returned-title");
  });

  test("real background tab pauses pending AI and resumes exactly once after reload", async ({
    page,
  }, testInfo) => {
    observeErrors(page);
    const touch = Boolean(testInfo.project.use.hasTouch);
    await page.goto(process.env.LOW_KINGDOMS_BASE_URL);
    await waitForScreen(page, "title");
    await activate(page, "play", touch, true);
    await waitForScreen(page, "mulligan");
    await activate(page, "keep-hand", touch);
    await waitForScreen(page, "battle");
    await submitResources(page, touch);
    const state = await waitForDecision(page);
    const command = state.battle.legal.find((item) => item.PlayUnit);
    expect(command).toBeDefined();
    await playAndPauseAi(page, command, touch);
    const resumed = await assertLayout(page);
    expect(resumed.battle.view.active_player).toBe("Zero");
    expect(resumed.battle.history).toContain(commandMessage(command, state));
    await capture(page, testInfo, "pending-ai-resumed-once");
  });
}
