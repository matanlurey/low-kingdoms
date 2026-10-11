# Milestone 1A implementation acceptance

The authoritative specification remains `milestone-1a.md`, including all 17 confirmed decisions.
Implementation is based on PR #1 commit `aaf4ec5b6f667274afba668ee7a64c92410d315f`.
That PR is unmerged; the implementation branch inherits its unchanged specification commit.

## Implemented

`crates/rules` executes complete two-player matches with injected basic unit definitions. All
setup, alternating actions, resources, combat, fatigue, victory/draws, private views/events,
legal discovery, versioned snapshots, validated restore, and deterministic replay are implemented.
The client remains separate; no Milestone 1B or later mechanics/content are introduced.

| Specification criteria | Evidence                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------------------------------- |
| F01–F06                | `rules` setup/configuration/buffered-decision/payment/regroup cases.                                          |
| F07–F11                | `rules` passing/initiative/payment/readiness/zero-cost/duplicate-unit cases.                                  |
| F12–F17                | `rules` combat/fatigue/outcome/boundary cases and `matches`.                                                  |
| F18                    | `privacy` projections, redacted events, pending-decision and error checks.                                    |
| F19                    | `rules` rejection helper compares full snapshots; overflow and corruption cases.                              |
| F20, T09               | Three complete scripted matches in `cases/matches.rs`.                                                        |
| T01                    | Rules dependency tree contains only Serde/JSON and their dependencies; structure gate restricts direct edges. |
| T02–T05                | Golden RNG, canonical decision/event order, validation plans, and privacy/discovery cases.                    |
| T06–T08                | Snapshot/continuation/replay cases; 32 invariant walks restore every step.                                    |
| T10–T14                | Focused and workspace Clippy/tests/docs plus native CI matrix and WASM gate.                                  |
| T15                    | Architecture, development/testing, root/rules agent instructions, local quality skill.                        |

## Interpretations and limits

No gameplay decisions changed. Buffered submissions emit no events until both players submit,
preventing private decision leakage and command-arrival-dependent resolution events. Resource
selection vectors normalize to ascending IDs. Exact canonical catalog contents provide snapshot
compatibility instead of a potentially colliding fingerprint. Invalid player IDs are unrepresentable
in the two-variant `PlayerId`; deserialization rejects invalid identifiers.

Printed statistics use `u32`, accumulated damage `u64`, and IDs/rounds checked `u64`. The arithmetic
counter boundary returns an atomic error, not a gameplay duration rule. The snapshot format is
version 1 with no migration. Authoritative snapshots are privileged; logical player authentication
and identity binding remain caller responsibilities. No storage, networking, AI, commander/ability,
action/upgrade, affinity/faction, card library, or gameplay presentation system exists.

## Verification record

Environment: Linux x86_64, pinned Rust/Cargo 1.98.0, Node 24.19.0, npm 11.9.0, Just 1.43.0.
Local checks are recorded below. PR CI results will be appended after inspection. No unexecuted
check is considered passing; WASM compilation does not establish runtime/browser behavior.

| Check                                             | Actual result                                                                                                      |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Focused rules tests, including three full matches | Passed; 28 tests and one API doctest.                                                                              |
| Rules strict all-targets Clippy                   | Passed with warnings denied.                                                                                       |
| Rules WASM compilation                            | Passed on the pinned toolchain.                                                                                    |
| Rules rustdoc                                     | Passed with warnings denied.                                                                                       |
| Host/WASM rules dependency trees                  | Passed; no direct or indirect Bevy dependency.                                                                     |
| Host tooling tests                                | Passed; 10 xtask tests.                                                                                            |
| `just check`                                      | Blocked locally at existing client `wayland-sys`: no pkg-config/system libraries. Formatting and structure passed. |
| `just check-web`                                  | Passed; compile/strict lint for application/rules, production and test-support configurations.                     |
| Native CI matrix (Linux/macOS/Windows)            | Pending PR.                                                                                                        |
| CI quality/WASM/browser/native-render             | Pending PR.                                                                                                        |

The local container cannot provision the native prerequisites: APT downloads hit filesystem/cache
permissions and blocked-by-allowlist responses. This does not pass the native gate; it remains
required in the unchanged CI Linux/macOS/Windows matrix and quality job, where prerequisites are
already provisioned. Local rendering checks were not run; client rendering code was not changed.

Milestone status: implementation complete; CI verification pending.
