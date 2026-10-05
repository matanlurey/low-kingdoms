# Client-specific guidance

Read the repository [AGENTS.md](../../AGENTS.md) and [STYLE_GUIDE.md](../../STYLE_GUIDE.md) first.
This directory owns the Bevy application; host process/build tooling belongs in `tools/xtask`.

- Verify unfamiliar APIs against Bevy **v0.19.1** source or version-specific published docs.
- Keep window/renderer composition separable from application/navigation plugin composition.
  Headless tests must use the actual production navigation and screen functions.
- Screen-owned roots and children have state-scoped lifecycle. Persistent application entities,
  such as the camera, must remain singular across repeated About/Back navigation.
- Name entities and add small markers where lifecycle tests or live diagnostics need identity.
  Do not expose Bevy entity IDs as a future domain representation.
- Use Bevy UI, focus, and input machinery. Screen state has one authority, not a JavaScript copy.
  Use explicit ordering for actual dependencies and advance tests through deferred state changes.
- Keep platform differences in `src/platform/`. The browser has no Quit control; native Quit
  requests a normal Bevy exit. Escape returns from About and does not quit the title screen.
- Centralize presentation values. Layout uses logical dimensions, with a minimum 44-pixel control
  height, readable wrapping, visible keyboard focus, and hover/pressed feedback.
- Do not block the browser main thread with file, network, thread, or process operations.
- A normal startup signal can dismiss loading UI. The non-default `test-support` bridge reads the
  live world only and grants no mutation. Production must not expose it.
- A screenshot request must complete before smoke-test exit, with a bounded failure path.

After changes, use the [bevy-change](../../.agents/skills/bevy-change/SKILL.md) procedure.
Run focused headless tests, `just check`, and the affected `just check-web`/rendering gates.
Before declaring changed presentation complete, inspect native/browser screenshots and record
the actual environment and limitations in [the plan](../../docs/plans/scaffold.md).
