# Architecture

The workspace has four implemented packages. `apps/game` depends on `crates/client`; the client
depends on Bevy and narrowly required browser bindings. `crates/rules` is an independent Serde/JSON
library. `tools/xtask` orchestrates host processes and does not depend on the client or renderer.
Browser tests live outside the Rust package graph.

| Location                      | Owned behavior                                                         |
| ----------------------------- | ---------------------------------------------------------------------- |
| `apps/game/src/main.rs`       | Calls the client's module-qualified bootstrap API.                     |
| `crates/client/src/app.rs`    | Application composition and renderer/window setup.                     |
| `crates/client/src/screens/`  | Title/About state, entry construction, and state-scoped cleanup.       |
| `crates/client/src/ui/`       | Shared style, input/focus, and navigation activation.                  |
| `crates/client/src/platform/` | Native/browser launch edges and capability differences.                |
| `crates/rules/`               | Complete authoritative two-player rules, safe views, snapshots/replay. |
| `tools/xtask/`                | Root/path resolution, subprocesses, packaging, checks, and evidence.   |
| `tools/web-tests/`            | Real browser input and inspection of generated WASM distributions.     |
| `web/`                        | Canvas host and WASM startup/error loader.                             |

## Application composition and ownership

The executable delegates to `low_kingdoms_client::app::run()`. Client composition installs the
screen and shared UI interaction plugins. Native and WASM use these same plugins and screen
construction functions. Renderer/window setup is separable so headless tests can compose the
actual application behavior without initializing a GPU.

The persistent application camera is created once. Bevy application state owns the title and About
screens. Each screen has one root with `DespawnOnExit`; descendant ownership removes its controls
and text on exit. Repeated About/Back transitions must not accumulate roots, cameras, handlers, or
observers. Meaningful `Name` values and small marker components give tests entity identity without
making internal presentation entities public application APIs.

Input activation reaches the production navigation path. Screen state is authoritative in Bevy.
The browser does not maintain its own screen model or replace the rendered menu with DOM controls.
Centralized style values and logical viewport dimensions control layout; physical framebuffer
dimensions and device pixel ratio remain separate from input/layout coordinates.

## Platform boundaries

The native edge creates a resizable window titled Low Kingdoms. Native Quit requests Bevy's normal
application exit. Browser configuration uses `#game-canvas`, hides Quit, and communicates startup
readiness to the loader. Escape from About returns to the title on both platforms.

Browser output uses WebGL2 with no WebGPU or threaded WASM requirement. HTML/CSS owns the canvas
parent's available size, while Bevy renders the product UI. Relative runtime URLs allow the same
static output at a root URL or a subpath. There is no runtime Node service, CDN, service worker,
network protocol, or gameplay state in JavaScript.

The normal `low-kingdoms-ready` event dismisses the loading overlay after live UI readiness. It is
not proof of pixel correctness. The non-default `test-support` feature supplies native screenshot
support and a read-only browser bridge named `window.__LOW_KINGDOMS_TEST__`. It derives observations
from the live world and permits no state mutation. Production packaging excludes that bridge.

## Tooling and distributions

`Justfile` uses pinned Node as its portable recipe interpreter and delegates substantial work to
host-only `xtask`. It passes arguments through `spawnSync` arrays without shell interpolation.
Direct `cargo xtask` remains the Rust-only command path; CI calls the same maintained commands.
Only application packages are built for `wasm32-unknown-unknown`; target-building the whole workspace
would incorrectly include host tools. Cargo profiles separate native release from size-oriented
`wasm-release`, which uses size optimization, one codegen unit, and disables LTO. The LTO choice
limits cold build/link cost and avoids the unstable linking path observed on constrained hosts;
bundle size remains a measured property. Native dynamic linking is not required.

Production output belongs in `dist/native/` and `dist/web/`. Browser test support uses
`dist/web-test/`. Screenshots, logs, traces, and measurements belong in ignored `evidence/`.
Generated output is never a new source of application truth.

Development static serving calls the pinned `http-server` API through `tools/web-tests/serve.mjs`.
The wrapper binds to loopback directly, avoiding unnecessary CLI network-interface enumeration.
It does not become part of the shipped static application.

## Headless rules boundary (Milestone 1A)

`crates/rules` is the `low-kingdoms-rules` library. It owns complete two-player matches without
Bevy, the client, host tools, operating-system entropy, time, or global mutable state. Its only
dependencies are the already locked Serde and JSON packages. The client does not depend on it yet;
Milestone 1B will add that edge. Rendering entities must never become rules identifiers.

The module-qualified public API is:

| Module    | Responsibility                                                                           |
| --------- | ---------------------------------------------------------------------------------------- |
| `model`   | Immutable unit definitions, stable player/card IDs, configuration, phases, outcomes.     |
| `command` | Explicit typed decisions, attack targets, caller-maintained replay entries.              |
| `engine`  | `Game::new`, `apply`, `view_for`, `legal_commands_for`, `snapshot`, `restore`, `replay`. |
| `event`   | Serializable filtered events, simultaneous damage assignments, private event batches.    |
| `view`    | Public battlefield/discards/counts plus only the viewing player's hand/resources.        |
| `error`   | Structured configuration, command, and privileged snapshot errors.                       |

Private `state` contains match/player zones, damage/readiness, active player, round, initiative,
consecutive passes, claimant, buffered decisions, PRNG state, and the next instance ID. The engine
validates each command into an application plan before mutating anything; application is infallible
under the validated state invariants. Failed commands emit no events and preserve the entire state.
Legal discovery uses the same validator and performs no state mutation.

The lifecycle is Mulligan → InitialResourcing → Action, then repeated Regroup → Action. A later
round's mandatory draw batch runs automatically before the engine accepts regroup resourcing.
Both players independently submit setup/regroup decisions. Submissions emit no events or public
decision flags until both arrive, then resolve in player order. Passing is temporary; claiming
initiative permanently removes the claimant from this round's eligible actors. Finished matches
accept no commands. Damage is persistent, and combat assignments precede all unit destructions.

`Game::new` returns `(Game, EventBatch)` for initialization. `apply(player, &command)` returns the
ordered transition batch. Use `batch.for_player(player)` before delivering events. Opposing hand
and resource identities are omitted entirely, including resource-payment IDs. Deck order is never
projected to either player. `Game` exposes no unrestricted state getter or Debug dump; event-batch
Debug reports only the event count. The caller is trusted to supply the logical player identity.

Randomness uses the explicitly implemented SplitMix64 integer stream and descending Fisher–Yates
shuffle with rejection sampling to remove modulo bias. The serialized `u64` state advances by
`0x9e3779b97f4a7c15`; mixing uses fixed wrapping multiplication constants. IDs start at one,
allocated in player/deck-input order. Opening initiative consumes the first bounded random value
unless overridden, followed by shuffling player 0 then 1; opening draws follow in that order.
Mulligan shuffles resolve in player order only after both submissions. The deck top is its final
element. No other gameplay consumes randomness. Fixed-width integer operations, explicit ordering,
and no hash-map iteration yield the same stream/transitions across supported native/WASM targets
for the same engine version. Golden stream/shuffle tests guard algorithm changes.

Resources are maintained in ascending instance-ID order; payment exhausts exactly the printed
cost. Simultaneous unit damage is one event with assignments sorted by recipient ID; destruction
events then sort by instance ID across both players. Regroup events run player 0's draw/fatigue,
then player 1's draw/fatigue, before determining either victory or a simultaneous draw. Deployment
events describe payment, the exhausted unit, and the next eligible actor. Automatic transitions
emit phase/round/readiness/next-actor events in their execution order.

Authoritative snapshots are privileged version-1 JSON, containing the canonical catalog, initial
configuration, all zones, PRNG/allocation state, and pending decisions. Restore compares exact
catalog contents, avoiding fingerprint collisions, and validates phase/outcome/card conservation
and ownership/statistic invariants before accepting commands. No storage or migration layer exists.
Replay reconstructs initialization plus a caller's ordered command log; events need not rebuild
the authoritative current state. Configuration/catalog order is normalized where appropriate;
player deck input order remains an explicit initialization input.

Cost/Power/Health use `u32`; accumulated damage uses `u64` so lethal overkill cannot overflow.
Round numbers and card IDs use checked `u64` counters. A round-ending command rejects representational
overflow before mutation; this is an arithmetic guard, not a gameplay round limit. Snapshot restore
rejects damage or statistics that could invalidate these arithmetic guarantees.
