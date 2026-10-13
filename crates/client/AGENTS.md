# Client-specific guidance

Read the repository [AGENTS.md](../../AGENTS.md) and [STYLE_GUIDE.md](../../STYLE_GUIDE.md) first.
This directory owns the Bevy application; host process/build tooling belongs in `tools/xtask`.

- Verify unfamiliar APIs against Bevy **v0.19.1** source or version-specific published docs.
- Keep window/renderer composition separable from application/navigation plugin composition.
  Headless tests must use the actual production gameplay, navigation, and screen functions.
- Screen-owned roots and children have state-scoped lifecycle. Persistent application entities,
  such as the camera, must remain singular across About/Back, battle/results, resume, and restart.
- Read [Milestone 1B](../../docs/plans/milestone-1b.md) before gameplay changes. The client owns
  presentation and scheduling, never match rules. The battle session keeps `Game` private.
  Obtain the human player view, legal commands, and filtered events through its API; submit typed
  commands for every change. Do not mutate damage, readiness, zones, initiative, or phases locally.
- Events drive feedback/history; views drive displayed state. Reconcile affected entities without
  rebuilding the whole UI each frame. Preserve scroll/target selection across health updates.
- The AI uses only its permitted view and validated legal actions. Schedule one action at a time
  without blocking, using Normal 750 ms, Fast 150 ms, or Instant zero delay. Pause while inactive
  and reset scheduling on return; never accumulate a background catch-up queue.
- Keep save I/O and entropy at the platform edge. Preserve the active slot until a replacement is
  confirmed, and save each accepted transition rather than every frame. Failed writes pause the
  affected path and retry the save, never the already successful command. No snapshots, command
  logs, seeds, or opposing hidden cards belong in UI/test-bridge snapshots or technical logs.
- Name entities and add small markers where lifecycle tests or live diagnostics need identity.
  Do not expose Bevy entity IDs as a future domain representation.
- Use Bevy UI, focus, and input machinery. Screen state has one authority, not a JavaScript copy.
  Use explicit ordering for actual dependencies and advance tests through deferred state changes.
- Keep platform differences in `src/platform/`. The browser has no Quit control; native Quit
  requests a normal Bevy exit. Escape returns from About and does not quit the title screen.
- Centralize presentation values. Layout uses logical dimensions, with a minimum 44-pixel control
  height, readable wrapping, visible keyboard focus, and hover/pressed feedback. The browser
  minimum is 375×667 CSS pixels. Use adaptive grids/scrollable rows, a collapsible compact hand,
  persistent Pass, and secondary Claim Initiative. Touch scrolling must not trigger card actions.
  Inspect and Attack/Play are distinct; legality comes from the engine, with invalid controls disabled.
- Do not block the browser main thread with file, network, thread, or process operations.
- A normal startup signal can dismiss loading UI. The non-default `test-support` bridge reads the
  live world only and grants no mutation. Production must not expose it.
- A screenshot request must complete before smoke-test exit, with a bounded failure path.

After changes, use the [bevy-change](../../.agents/skills/bevy-change/SKILL.md) procedure.
Run focused headless tests, `just check`, and the affected `just check-web`/rendering gates.
Before declaring changed presentation complete, inspect native/browser screenshots and record
the actual environment and limitations in
[Milestone 1B acceptance](../../docs/plans/milestone-1b-acceptance.md). Preserve original bootstrap
and navigation checks recorded in the [scaffold](../../docs/plans/scaffold.md).
