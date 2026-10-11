# Working in the rules engine

Read root `AGENTS.md`, `STYLE_GUIDE.md`, `.agents/skills/rust-quality/SKILL.md`, and the complete
`docs/plans/milestone-1a.md` before changing rules. The 17 decisions are authoritative.
Keep implementation evidence in `docs/plans/milestone-1a-acceptance.md`; leave the original spec intact.

This library depends only on Serde and JSON. Do not add Bevy, client dependencies, networking,
OS randomness, clocks, global mutable state, commanders, abilities, keywords, card content, or AI.
Public modules own their concrete APIs; private `state` and `rng` are implementation details.

Every command validates into an application plan before mutation. Never partially apply a failed
command or consume its randomness. Keep private decisions buffered until both players submit,
then resolve in player order. Resource selections/payment and unit destruction use ascending IDs.
Combat damage is one batch; both mandatory regroup draws precede outcome detection.

Send the UI only `Game::view_for` and `EventBatch::for_player` output. Snapshots contain all private
information and are privileged. Do not add raw-state getters or expose private events through Debug.
Restore must validate exact catalog compatibility, conservation, domain values, decisions, and phase.
Changes to PRNG, shuffle, ID allocation, or snapshot semantics need explicit compatibility review.

Run the focused native tests, strict Clippy, and WASM check, then `just check` and `just check-web`.
Use anonymous fixtures in `tests/headless.rs` and `tests/cases/`. Add meaningful boundary and invariant
checks for changed behavior, including failed-command atomicity and event/view privacy.
No game screens or client integration belong to Milestone 1A.
