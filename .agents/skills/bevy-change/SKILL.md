---
name: bevy-change
description: Change Low Kingdoms Bevy application composition, UI, input, navigation states, or platform behavior using the pinned engine and real lifecycle/rendering checks.
---

# Bevy change

1. Read root `AGENTS.md`, `crates/client/AGENTS.md`, and the affected source. Read
   `rust-toolchain.toml` and the Bevy dependency/feature entries in `Cargo.toml`.
2. Identify the smallest affected plugin or module and an observable acceptance condition.
   Keep the change to one coherent behavior. No gameplay types, placeholder controls, or future APIs.
3. Verify unfamiliar APIs in the **v0.19.1** tagged source or version-specific API documentation.
   Use `docs/references/bevy.md` to locate the relevant input/state/UI/capture example.
4. Implement through the shared production screen/navigation path. Preserve explicit screen
   ownership and singular persistent entities. Keep platform differences at the client platform edge.
5. Run `cargo check --locked -p low-kingdoms-client` and focused existing headless tests.
   Update lifecycle/input tests for the observable behavior using real production plugins/functions.
   Advance `App` enough to apply deferred commands and state transitions.
6. Run `just check`. For browser-affecting code, run `just check-web`; for platform/renderer changes,
   run the affected native and browser smoke checks using `verify-native-web`.
7. Inspect changed presentation captures for text, clipping, touch targets, focus, and viewport fit.
   Record check results and evidence paths in `docs/plans/scaffold.md`.

Completion reports distinguish compilation, headless behavior, and inspected rendering. Missing
renderer/browser capabilities are explicit blockers. ECS components, rendering state, and entity
IDs must not become the public representation of a future domain library.

Open `docs/architecture.md` only for dependency/lifecycle questions and `docs/testing.md` for the
full acceptance matrix. Do not copy upstream sample gameplay into this scaffold.
