import fs from "node:fs/promises";
import os from "node:os";
import { expect, test } from "@playwright/test";

const routes = process.env.LOW_KINGDOMS_DEPLOYED_URL
  ? []
  : [
      ["root", process.env.LOW_KINGDOMS_BASE_URL],
      ["subpath", process.env.LOW_KINGDOMS_SUBPATH_URL],
    ];
const browserErrors = new WeakMap();

function observeErrors(page) {
  const errors = [];
  browserErrors.set(page, errors);
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

test.afterEach(async ({ page }, testInfo) => {
  const errors = browserErrors.get(page) ?? [];
  const filename = testInfo.outputPath("browser-errors.json");
  await fs.writeFile(filename, JSON.stringify(errors, null, 2));
  await testInfo.attach("browser-errors", {
    path: filename,
    contentType: "application/json",
  });
  const diagnosticPath = testInfo.outputPath("viewport-diagnostics.json");
  await fs.writeFile(
    diagnosticPath,
    JSON.stringify(
      {
        configuredViewport: page.viewportSize(),
        canvas: await canvasDimensions(page),
        application: (await bridge(page)) ?? null,
      },
      null,
      2,
    ),
  );
  await testInfo.attach("viewport-diagnostics", {
    path: diagnosticPath,
    contentType: "application/json",
  });
  expect(errors).toEqual([]);
});

async function bridge(page) {
  return page.evaluate(() => window.__LOW_KINGDOMS_TEST__);
}

async function recordEnvironment(page, testInfo, errors) {
  const graphics = await page.evaluate(() => {
    const context = document.querySelector("#game-canvas").getContext("webgl2");
    if (!context) return { webgl2: false };
    const debug = context.getExtension("WEBGL_debug_renderer_info");
    return {
      webgl2: true,
      version: context.getParameter(context.VERSION),
      vendor: debug ? context.getParameter(debug.UNMASKED_VENDOR_WEBGL) : null,
      renderer: debug
        ? context.getParameter(debug.UNMASKED_RENDERER_WEBGL)
        : null,
    };
  });
  expect(graphics.webgl2).toBe(true);
  const filename = testInfo.outputPath("browser-environment.json");
  await fs.writeFile(
    filename,
    JSON.stringify(
      {
        browser: page.context().browser().version(),
        node: process.version,
        os: `${os.platform()} ${os.release()} ${os.arch()}`,
        project: testInfo.project.name,
        graphics,
        input: testInfo.project.use.hasTouch
          ? "touchscreen and keyboard"
          : "mouse and keyboard",
        errors,
      },
      null,
      2,
    ),
  );
  await testInfo.attach("browser-environment", {
    path: filename,
    contentType: "application/json",
  });
}

async function waitForScreen(page, screen) {
  await expect
    .poll(async () => {
      const errors = browserErrors.get(page);
      if (errors?.length) throw new Error(errors.join("\n"));
      return (await bridge(page))?.screen;
    })
    .toBe(screen);
  await expect.poll(async () => (await bridge(page))?.ready).toBe(true);
}

async function assertLayout(page) {
  const state = await bridge(page);
  const viewport = page.viewportSize();
  expect(state.viewport.width).toBeCloseTo(viewport.width, 0);
  expect(state.viewport.height).toBeCloseTo(viewport.height, 0);
  expect(state.controls.map((control) => control.id)).not.toContain("quit");
  expect(state.controls).not.toHaveLength(0);
  for (const rectangle of [...state.controls, ...state.texts]) {
    expect(rectangle.width).toBeGreaterThan(0);
    expect(rectangle.height).toBeGreaterThan(0);
    expect(rectangle.x).toBeGreaterThanOrEqual(-0.5);
    expect(rectangle.y).toBeGreaterThanOrEqual(-0.5);
    expect(rectangle.x + rectangle.width).toBeLessThanOrEqual(
      viewport.width + 0.5,
    );
    expect(rectangle.y + rectangle.height).toBeLessThanOrEqual(
      viewport.height + 0.5,
    );
  }
  for (const control of state.controls) {
    expect(control.height).toBeGreaterThanOrEqual(44);
  }
  const dimensions = await assertCanvasLayout(page);
  expect(state.viewport.scaleFactor).toBeCloseTo(
    dimensions.deviceScaleFactor,
    2,
  );
  return state;
}

async function canvasDimensions(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("#game-canvas");
    if (!canvas) return null;
    const rectangle = canvas.getBoundingClientRect();
    return {
      physicalWidth: canvas.width,
      physicalHeight: canvas.height,
      width: rectangle.width,
      height: rectangle.height,
      scrollWidth: document.documentElement.scrollWidth,
      scrollHeight: document.documentElement.scrollHeight,
      deviceScaleFactor: window.devicePixelRatio,
      innerWidth: window.innerWidth,
      innerHeight: window.innerHeight,
      visualViewport: window.visualViewport
        ? {
            width: window.visualViewport.width,
            height: window.visualViewport.height,
            scale: window.visualViewport.scale,
          }
        : null,
    };
  });
}

async function assertCanvasLayout(page) {
  const viewport = page.viewportSize();
  const dimensions = await canvasDimensions(page);
  expect(dimensions).not.toBeNull();
  expect(dimensions.width).toBe(viewport.width);
  expect(dimensions.height).toBe(viewport.height);
  expect(dimensions.physicalWidth).toBeGreaterThan(0);
  expect(dimensions.physicalHeight).toBeGreaterThan(0);
  expect(
    Math.abs(
      dimensions.physicalWidth - viewport.width * dimensions.deviceScaleFactor,
    ),
  ).toBeLessThanOrEqual(2);
  expect(
    Math.abs(
      dimensions.physicalHeight -
        viewport.height * dimensions.deviceScaleFactor,
    ),
  ).toBeLessThanOrEqual(2);
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(viewport.width);
  expect(dimensions.scrollHeight).toBeLessThanOrEqual(viewport.height);
  return dimensions;
}

async function inspectPixels(page, screenshot) {
  // Inspect captured pixels, rather than the canvas backing buffer (WebGL may
  // discard that buffer). This is a blank-output guard; humans inspect images.
  return page.evaluate(async (encoded) => {
    const picture = new Image();
    picture.src = `data:image/png;base64,${encoded}`;
    await picture.decode();
    const inspection = new OffscreenCanvas(picture.width, picture.height);
    const context = inspection.getContext("2d");
    context.drawImage(picture, 0, 0);
    const data = context.getImageData(0, 0, picture.width, picture.height).data;
    const colors = new Set();
    let lightPixels = 0;
    for (let index = 0; index < data.length; index += 4) {
      colors.add(`${data[index]},${data[index + 1]},${data[index + 2]}`);
      if (data[index] + data[index + 1] + data[index + 2] > 450) lightPixels++;
    }
    return { colors: colors.size, lightPixels };
  }, screenshot.toString("base64"));
}

async function capture(page, testInfo, name, state) {
  const filename = testInfo.outputPath(`${name}.png`);
  let screenshot;
  let pixels;
  // Font/layout readiness can precede GPU pipeline completion. Require actual
  // nonblank captured pixels with a bounded retry, rather than a fixed delay.
  await expect
    .poll(
      async () => {
        screenshot = await page.locator("#game-canvas").screenshot({
          path: filename,
          scale: "css",
        });
        pixels = await inspectPixels(page, screenshot);
        return Math.min(pixels.colors / 20, pixels.lightPixels / 200);
      },
      { timeout: 60_000, intervals: [100, 250, 500] },
    )
    .toBeGreaterThan(1);
  expect(pixels.colors).toBeGreaterThan(20);
  expect(pixels.lightPixels).toBeGreaterThan(200);
  await testInfo.attach(name, { path: filename, contentType: "image/png" });
  if (state) {
    const diagnosticPath = testInfo.outputPath(`${name}.json`);
    await fs.writeFile(diagnosticPath, JSON.stringify(state, null, 2));
    await testInfo.attach(`${name}-diagnostics`, {
      path: diagnosticPath,
      contentType: "application/json",
    });
  }
  return screenshot;
}

async function changedControlPixels(page, before, after, rectangle) {
  return page.evaluate(
    async ({ previous, current, rectangle }) => {
      const decode = async (encoded) => {
        const image = new Image();
        image.src = `data:image/png;base64,${encoded}`;
        await image.decode();
        const canvas = new OffscreenCanvas(image.width, image.height);
        const context = canvas.getContext("2d");
        context.drawImage(image, 0, 0);
        return context.getImageData(0, 0, image.width, image.height);
      };
      const first = await decode(previous);
      const second = await decode(current);
      let changed = 0;
      const left = Math.max(0, Math.floor(rectangle.x - 4));
      const right = Math.min(
        second.width,
        Math.ceil(rectangle.x + rectangle.width + 4),
      );
      const top = Math.max(0, Math.floor(rectangle.y - 4));
      const bottom = Math.min(
        second.height,
        Math.ceil(rectangle.y + rectangle.height + 4),
      );
      for (let y = top; y < bottom; y++) {
        for (let x = left; x < right; x++) {
          const index = (y * second.width + x) * 4;
          const difference =
            Math.abs(first.data[index] - second.data[index]) +
            Math.abs(first.data[index + 1] - second.data[index + 1]) +
            Math.abs(first.data[index + 2] - second.data[index + 2]);
          if (difference > 10) changed++;
        }
      }
      return changed;
    },
    {
      previous: before.toString("base64"),
      current: after.toString("base64"),
      rectangle,
    },
  );
}

async function activate(page, controlId, touch) {
  const state = await bridge(page);
  const control = state.controls.find((item) => item.id === controlId);
  expect(control).toBeDefined();
  const x = control.x + control.width / 2;
  const y = control.y + control.height / 2;
  if (touch) await page.touchscreen.tap(x, y);
  else await page.mouse.click(x, y);
}

for (const [routeName, url] of routes) {
  test(`${routeName}: real input, focus, navigation, and resize`, async ({
    page,
  }, testInfo) => {
    const errors = observeErrors(page);
    await page.goto(url);
    await waitForScreen(page, "title");
    await expect(page.locator("#startup-status")).toBeHidden();
    expect(
      await page.evaluate(() => Object.isFrozen(window.__LOW_KINGDOMS_TEST__)),
    ).toBe(true);
    expect(
      await page.evaluate(() => window.__LOW_KINGDOMS_TEST__.navigate),
    ).toBeUndefined();
    const title = await assertLayout(page);
    expect(title.texts.map((item) => item.text)).toContain("Low Kingdoms");
    const titleImage = await capture(page, testInfo, "title", title);
    if (!testInfo.project.use.hasTouch) {
      const aboutControl = title.controls.find((item) => item.id === "about");
      await page.mouse.move(
        aboutControl.x + aboutControl.width / 2,
        aboutControl.y + aboutControl.height / 2,
      );
      const hoveredImage = await capture(
        page,
        testInfo,
        "title-hovered",
        await bridge(page),
      );
      expect(
        await changedControlPixels(
          page,
          titleImage,
          hoveredImage,
          aboutControl,
        ),
      ).toBeGreaterThan(100);
      await page.mouse.move(0, 0);
    }

    await page.keyboard.press("Tab");
    await expect
      .poll(async () => (await bridge(page))?.focusedControl)
      .toBe("play");
    await page.keyboard.press("Tab");
    await expect
      .poll(async () => (await bridge(page))?.focusedControl)
      .toBe("about");
    await expect
      .poll(async () => (await bridge(page))?.focusVisible)
      .toBe(true);
    const focused = await bridge(page);
    expect(focused.controls.find((item) => item.id === "about").focused).toBe(
      true,
    );
    const focusedImage = await capture(
      page,
      testInfo,
      "title-focused",
      focused,
    );
    expect(
      await changedControlPixels(
        page,
        titleImage,
        focusedImage,
        focused.controls.find((item) => item.id === "about"),
      ),
    ).toBeGreaterThan(100);
    await page.keyboard.press("Shift+Tab");
    await expect
      .poll(async () => (await bridge(page))?.focusedControl)
      .toBe("play");
    await page.keyboard.press("Tab");
    await expect
      .poll(async () => (await bridge(page))?.focusedControl)
      .toBe("about");
    await page.keyboard.press("Enter");
    await waitForScreen(page, "about");
    const about = await assertLayout(page);
    expect(about.texts.map((item) => item.text)).toContain(
      "Playable battle prototype against a basic AI.",
    );
    await capture(page, testInfo, "about", about);
    await page.keyboard.press("Escape");
    await waitForScreen(page, "title");
    await page.keyboard.press("Escape");
    await waitForScreen(page, "title");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Space");
    await waitForScreen(page, "about");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Enter");
    await waitForScreen(page, "title");

    const touch = Boolean(testInfo.project.use.hasTouch);
    await activate(page, "about", touch);
    await waitForScreen(page, "about");
    await activate(page, "back", touch);
    await waitForScreen(page, "title");
    const original = page.viewportSize();
    const resized =
      original.width > 900
        ? { width: 900, height: 550 }
        : { width: original.height, height: original.width };
    await page.setViewportSize(resized);
    await expect
      .poll(async () => (await bridge(page))?.viewport.width)
      .toBeCloseTo(resized.width, 0);
    const resizeState = await assertLayout(page);
    await capture(page, testInfo, "resized-title", resizeState);
    await activate(page, "about", touch);
    await waitForScreen(page, "about");
    await assertLayout(page);
    await activate(page, "back", touch);
    await waitForScreen(page, "title");
    if (testInfo.project.use.deviceScaleFactor === 2) {
      const session = await page.context().newCDPSession(page);
      await session.send("Emulation.setDeviceMetricsOverride", {
        ...resized,
        deviceScaleFactor: 1,
        mobile: true,
      });
      await expect
        .poll(async () => (await bridge(page))?.viewport.scaleFactor)
        .toBe(1);
      const changedDpi = await assertLayout(page);
      await capture(page, testInfo, "changed-dpi-title", changedDpi);
      await activate(page, "about", touch);
      await waitForScreen(page, "about");
      await assertLayout(page);
      await activate(page, "back", touch);
      await waitForScreen(page, "title");
      await session.detach();
    }
    expect(errors).toEqual([]);
    await recordEnvironment(page, testInfo, errors);
  });
}

test("production bundle renders without diagnostic bridge", async ({
  page,
}, testInfo) => {
  const errors = observeErrors(page);
  await page.addInitScript(() => {
    window.addEventListener(
      "low-kingdoms-ready",
      () => {
        document.documentElement.dataset.applicationReady = "true";
      },
      { once: true },
    );
  });
  await page.goto(
    process.env.LOW_KINGDOMS_DEPLOYED_URL ??
      process.env.LOW_KINGDOMS_PRODUCTION_URL,
  );
  await expect(page.locator("html")).toHaveAttribute(
    "data-application-ready",
    "true",
  );
  await expect(page.locator("#startup-status")).toBeHidden();
  expect(await bridge(page)).toBeUndefined();
  expect(await page.evaluate(() => "__LOW_KINGDOMS_TEST__" in window)).toBe(
    false,
  );
  await assertCanvasLayout(page);
  await capture(page, testInfo, "production-title");
  await page.locator("#game-canvas").focus();
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  const focusedTitle = await capture(
    page,
    testInfo,
    "production-title-focused",
  );
  const viewport = page.viewportSize();
  const wholeCanvas = {
    x: 0,
    y: 0,
    width: viewport.width,
    height: viewport.height,
  };
  await page.keyboard.press("Enter");
  let aboutImage;
  await expect
    .poll(async () => {
      aboutImage = await page
        .locator("#game-canvas")
        .screenshot({ scale: "css" });
      return changedControlPixels(page, focusedTitle, aboutImage, wholeCanvas);
    })
    .toBeGreaterThan(2000);
  await capture(page, testInfo, "production-about");
  await page.keyboard.press("Escape");
  await expect
    .poll(async () => {
      const returned = await page
        .locator("#game-canvas")
        .screenshot({ scale: "css" });
      return changedControlPixels(page, aboutImage, returned, wholeCanvas);
    })
    .toBeGreaterThan(2000);
  await capture(page, testInfo, "production-title-back");
  await assertCanvasLayout(page);
  expect(await bridge(page)).toBeUndefined();
  expect(errors).toEqual([]);
  await recordEnvironment(page, testInfo, errors);
});
