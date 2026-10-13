# Architecture

The workspace has five packages. `apps/game` depends on `crates/client`; the client depends on
Bevy, `crates/battle`, `crates/rules`, and narrowly required platform bindings. Battle depends on
rules, RON, Serde, and JSON. Neither battle nor rules has a direct or indirect Bevy dependency.
`tools/xtask` orchestrates host processes and does not depend on the client or renderer. Browser
tests live outside the Rust package graph.

| Location                      | Owned behavior                                                               |
| ----------------------------- | ---------------------------------------------------------------------------- |
| `apps/game/src/main.rs`       | Calls the client's module-qualified bootstrap API.                           |
| `crates/client/src/app.rs`    | Application composition and renderer/window setup.                           |
| `crates/client/src/screens/`  | Title, About, Mulligan, Battle, Results, and state-scoped cleanup.           |
| `crates/client/src/ui/`       | Shared style, input/focus, and navigation activation.                        |
| `crates/client/src/platform/` | Native/browser launch, activity, entropy, and local storage edges.           |
| `crates/client/src/gameplay/` | Session ownership, command dispatch, scheduling, and presentation.           |
| `crates/battle/`              | Embedded content, deterministic AI, and authoritative session/save boundary. |
| `crates/rules/`               | Complete authoritative two-player rules, safe views, snapshots/replay.       |
| `tools/xtask/`                | Root/path resolution, subprocesses, packaging, checks, and evidence.         |
| `tools/web-tests/`            | Real browser input and inspection of generated WASM distributions.           |
| `web/`                        | Canvas host and WASM startup/error loader.                                   |

## Application composition and ownership

The executable delegates to `low_kingdoms_client::app::run()`. Client composition installs the
screen, gameplay, and shared UI interaction plugins. Native and WASM use these same plugins and screen
construction functions. Renderer/window setup is separable so headless tests can compose the
actual application behavior without initializing a GPU.

The persistent application camera is created once. Bevy application state owns Title, About,
Mulligan, Battle, and Results screens. Each screen has one root with `DespawnOnExit`; descendant ownership removes its controls
and text on exit. Repeated navigation, match restarts, and save restoration must not accumulate roots, cameras,
handlers, or observers. Meaningful `Name` values and small marker components give tests entity identity without
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
network protocol, or independently modeled gameplay state in JavaScript. Browser storage holds
opaque privileged save text; all parsing, validation, and decisions stay in Rust.

The normal `low-kingdoms-ready` event dismisses the loading overlay after live UI readiness. It is
not proof of pixel correctness. The non-default `test-support` feature supplies native screenshot
support and a read-only browser bridge named `window.__LOW_KINGDOMS_TEST__`. It derives observations
from the live world and permits no state mutation. Gameplay observations contain only the human
view, permitted legal actions, and presentation data. They exclude seeds, raw snapshots, replay
logs, AI private hands/resources, and deck order. Production packaging excludes that bridge.

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
dependencies are Serde and JSON. The client and battle library consume this API; neither changes
its ownership. Rendering entities must never become rules identifiers.

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
and ownership/statistic invariants before accepting commands. The rules library contains no
storage or migration layer; Milestone 1B adds local storage outside it.
Replay reconstructs initialization plus a caller's ordered command log; events need not rebuild
the authoritative current state. Configuration/catalog order is normalized where appropriate;
player deck input order remains an explicit initialization input.

Cost/Power/Health use `u32`; accumulated damage uses `u64` so lethal overkill cannot overflow.
Round numbers and card IDs use checked `u64` counters. A round-ending command rejects representational
overflow before mutation; this is an arithmetic guard, not a gameplay round limit. Snapshot restore
rejects damage or statistics that could invalidate these arithmetic guarantees.

## Prototype battle boundary (Milestone 1B)

`crates/battle` is the `low-kingdoms-battle` library. It introduces the minimum content, AI, and
session boundary needed to play the approved 1A rules without rendering dependencies.

| Module    | Public API and responsibility                                                                                                                                 |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `content` | `Content::load`/`parse`, `cards`, `config`, `signature`, and `CardDefinition::unit_definition`; validated embedded RON and exact compatibility identity.      |
| `ai`      | `choose(view, legal, catalog)`; a legal, deterministic command using only that player's permitted information.                                                |
| `session` | `Session::new`/`restore`, `view`, `legal_commands`, `needs_ai`, `apply_human`, `step_ai`, `history`, `revision`, `is_finished`, and privileged `save`/`seed`. |

Content embeds six vanilla named units and exactly two twenty-card decks. Definition IDs are
canonicalized; deck order remains a meaningful engine input. The signature is exact canonical
JSON of names, statistics, deck references/order, and stronghold configuration, rather than a
potentially colliding hash. Names are presentation data; printed statistics go unchanged to the
rules engine. RON is parsed once during client initialization, not on each action or frame.

The AI receives its own `PlayerView`, the engine's legal commands, and public definitions. It never
receives `Game`, an authoritative snapshot, the human view, or deck lists. Explicit priorities
favor winning attacks, favorable trades, useful deployments, stronghold attacks, then claiming
initiative or passing when no worthwhile action remains. Setup preserves useful inexpensive
units and resources expensive or redundant cards. Stable command/instance-ID tie-breaking is
independent of legal-list enumeration order. The profile is stateless; timing does not affect
strategy, and no random source, search tree, network call, or generic effect interpreter is involved.

`Session` keeps `Game` private and fixes human player zero/opponent player one. Public presentation
methods expose only the human's permitted view, legal commands, and filtered events. Each accepted
command increments a revision, including buffered setup decisions that emit no public events.
Validation failures preserve game state, revision, history, and log. AI selection or engine errors
are returned directly; an unrelated fallback action never hides a defect.

## Client command and synchronization lifecycle

A control constructs a typed engine command from permitted card IDs and the current legal list.
The session validates and applies it, returns human-filtered events, and supplies a fresh human
view. Events drive notifications and chronological history; the view supplies displayed zones,
readiness, damage, strongholds, resource counters, and phase. UI code does not calculate combat,
resource payment, eligibility, pass sequences, or initiative transitions.

The presentation model owns only transient selection/target mode, scroll positions, panels,
notifications, and speed preference. Changed views reconcile affected card entities and text.
Starting/restoring a match performs a full reconciliation; inconsistency recovery may also do so.
Ordinary damage/readiness updates preserve row scroll and attacker selection. Screen-owned roots
clean up on transitions; the persistent camera and gameplay composition stay singular.

Desktop uses opposing fields with an adaptive grid and log sidebar. Compact viewports use
horizontal unit rows, a collapsible hand tray, and dismissible previews/log/resource panels.
Pass remains outside scroll areas; Claim Initiative is a secondary action. Input distinguishes
inspection from contextual Play/Attack and suppresses activation while scrolling. Legal commands
are the source for disabled actions and highlighted targets. Automatic layout uses logical
viewport dimensions with a 375×667 CSS-pixel minimum, safe-area handling, and no manual scale knob.

AI scheduling executes at most one action at a time, allowing presentation to update before the
next action. Normal waits 750 ms, Fast 150 ms, and Instant adds no intentional delay. Inactive
browser documents/native windows pause scheduling without a backlog; activation starts a fresh
schedule. UI timing, selection, and open panels do not enter rules state.

Unexpected command/AI/presentation failures stop the affected path, preserve the last valid session,
and offer recovery. A save failure after a successful command retries persistence of that current
session; it must not replay the command. Technical diagnostics describe the operation without
printing snapshots, logs, or hidden cards.

## Local save and replay boundary

The platform `Store` abstraction reads, replaces, or clears one opaque active slot. Browser
storage uses `localStorage`; native storage uses an absolute per-user application data directory
and a sibling temporary file followed by replacement. Fresh match seeds come from browser Crypto
or native operating-system entropy at this edge. Rules and battle remain explicit-seed deterministic
libraries. Play Again obtains a fresh seed rather than reusing the previous match.

The session save envelope is version 1 and contains exact content identity, fixed roles, AI profile
version/state, privileged seed, revision, current phase, full version-1 engine snapshot, ordered
successful command log, and human-filtered event history. The AI state is explicitly stateless.
Timers/queued actions and transient selection are excluded; after restore, `needs_ai` derives
eligibility from the authoritative pending state, so already completed decisions do not repeat.
The client may wrap session text with versioned presentation preferences such as AI speed.

Restore validates compatibility and the engine snapshot, replays the ordered commands, checks AI
commands against the saved profile, then compares exact resulting snapshot, revision, phase, and
filtered history. There is no format/content migration. The presentation history retains the last
256 events; the full command log remains intact for replay. Saves are limited to 2 MiB and 4096
successful commands as protective implementation bounds, with atomic rejection before advancing
beyond the command bound. These are not new gameplay rules.

Snapshots/logs contain hidden information and remain privileged local persistence data. They are
never a UI/test-bridge projection or diagnostic dump. This prototype does not protect local files
from their owner and does not define a networking/authentication protocol.

Compatible unfinished saves enable Resume Battle. Replacement requires confirmation; returning to
Main Menu preserves an unfinished slot. Finished saves can validate and restore internally for
cleanup, but are never resumable. Completion clears the active slot; cleanup failure is surfaced
and may preserve a finished record that cannot reopen gameplay. Missing, corrupt, incompatible,
unavailable, and quota-limited storage have explicit recovery paths. No database, cloud service,
save-slot browser, or account system is introduced.
