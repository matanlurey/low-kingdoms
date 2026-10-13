import { expect } from "@playwright/test";

// These helpers control actual browser input and tab activity. They never change
// document visibility, authoritative game state, or the read-only test bridge.
export async function createBackgroundTab(page) {
  const background = await page.context().newPage();
  for (const tab of [page, background]) {
    const session = await page.context().newCDPSession(tab);
    await session.send("Emulation.setFocusEmulationEnabled", {
      enabled: false,
    });
    await session.detach();
  }
  return background;
}

export async function waitForTabActivity(page, active) {
  await expect
    .poll(() =>
      page.evaluate(
        (expected) =>
          (document.visibilityState === "visible" && document.hasFocus()) ===
          expected,
        active,
      ),
    )
    .toBe(true);
}

export async function holdTouchThroughPrivateRevision(page, before, observe) {
  const control = before.controls.find(
    (entry) => entry.id === "main-menu" && entry.visible !== false,
  );
  expect(control).toBeDefined();
  expect(control.enabled).not.toBe(false);
  const rectangle = control.visibleRect ?? control;
  expect(rectangle.width).toBeGreaterThanOrEqual(44);
  expect(rectangle.height).toBeGreaterThanOrEqual(44);
  expect(before.battle.panel).toBe("More");
  expect(before.battle.view.phase).toBe("InitialResourcing");
  expect(before.battle.view.decision_submitted).toBe(false);
  expect(before.battle.revision).toBe(2);
  const session = await page.context().newCDPSession(page);
  try {
    await session.send("Input.dispatchTouchEvent", {
      type: "touchStart",
      touchPoints: [
        {
          x: rectangle.x + rectangle.width / 2,
          y: rectangle.y + rectangle.height / 2,
          id: 1,
        },
      ],
    });
    // The pending AI resource choice changes the technical revision, but none
    // of the human-visible menu inputs. A held control must survive that update.
    await expect
      .poll(async () => (await observe()).battle.revision, {
        intervals: [10, 25, 50],
      })
      .toBe(before.battle.revision + 1);
    const during = await observe();
    expect(during.battle.panel).toBe("More");
    expect(during.battle.view).toEqual(before.battle.view);
    expect(during.battle.legal).toEqual(before.battle.legal);
    expect(during.battle.history).toEqual(before.battle.history);
    return {
      beforeRevision: before.battle.revision,
      duringRevision: during.battle.revision,
      viewUnchanged: true,
      legalUnchanged: true,
      historyUnchanged: true,
    };
  } finally {
    await session.send("Input.dispatchTouchEvent", {
      type: "touchEnd",
      touchPoints: [],
    });
    await session.detach();
  }
}
