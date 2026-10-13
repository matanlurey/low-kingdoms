# Milestone 1B implementation acceptance

The authoritative approved specification is [milestone-1b.md](milestone-1b.md), containing all
33 confirmed decisions from the supplied worker instructions. The specification and implementation
belong to the same feature branch and one implementation PR; no separate planning PR is required.
The original [1A gameplay specification](milestone-1a.md) remains unchanged.

## Status and evidence policy

Milestone status: **incomplete while implementation and verification are in progress**.
Every criterion below is **pending verification** until its cited checks execute and actual
rendered gameplay is inspected. The evidence column names implementation/test responsibilities;
it does not itself claim those checks passed. Compilation, headless execution, browser interaction,
and visual inspection are separate claims. Update this record with actual results and exact run
links/evidence paths before declaring completion.

## Criterion mapping

### Gameplay

| ID  | Requirement                  | Implementation evidence                                           | Verification evidence                                                      | Status  |
| --- | ---------------------------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------- | ------- |
| G01 | Start from Title             | Client gameplay start command and Title Play control.             | Client lifecycle tests; browser battle start.                              | Pending |
| G02 | Use the actual 1A engine     | `battle::session::Session` privately owns `rules::engine::Game`.  | Session atomicity/continuation tests; client production command tests.     | Pending |
| G03 | Load two valid RON decks     | `assets/content/cards.ron`, `decks.ron`; `battle::content`.       | `tests/content_ai.rs` loading and validation.                              | Pending |
| G04 | Human mulligan               | Mulligan screen; `Session::apply_human` typed mulligan command.   | Client setup test; browser mulligan flow.                                  | Pending |
| G05 | AI mulligan                  | `battle::ai::choose` setup heuristic; `Session::step_ai`.         | AI mulligan cases; session pending-setup cases.                            | Pending |
| G06 | Human resources              | Inline legal selection and Confirm Resources.                     | Client zero/one/two selection tests; browser setup/regroup.                | Pending |
| G07 | AI resources                 | Deterministic expensive/redundant-card setup decisions.           | AI initial/subsequent resource cases; full simulations.                    | Pending |
| G08 | Human deploys                | Hand preview contextual Play submits a legal deploy command.      | Client deploy/reconciliation test; real browser play.                      | Pending |
| G09 | AI deploys                   | AI selects affordable validated deploy commands.                  | AI deployment cases; full matches; browser observed action.                | Pending |
| G10 | Human attacks units          | Friendly preview Attack and highlighted legal enemy target.       | Client combat/targeting test; real browser unit attack.                    | Pending |
| G11 | Human attacks strongholds    | Legal stronghold targeting through the same command path.         | Client stronghold/results test; browser stronghold attack.                 | Pending |
| G12 | AI attacks both target types | Explicit winning/trade/stronghold priorities.                     | AI combat cases and complete AI matches.                                   | Pending |
| G13 | Passing                      | Persistent control submits `Command::Pass`.                       | Existing 1A pass cases; client/browser action tests.                       | Pending |
| G14 | Claim initiative             | Secondary menu submits `Command::ClaimInitiative`.                | 1A claim cases; AI/client/browser claim cases.                             | Pending |
| G15 | Regroup                      | Engine phase/draw batch; inline one-card decision.                | 1A progression cases; client/browser round transitions.                    | Pending |
| G16 | Correct damage display       | Human view statistics drive Power/remaining Health and damage.    | Client reconciliation test; inspected damaged-unit/stronghold capture.     | Pending |
| G17 | Correct exhaustion display   | Human view readiness drives text and dimming.                     | Client ready/exhausted test; inspected deployed/attacking cards.           | Pending |
| G18 | Victory/defeat               | Engine outcome selects Results; Draw is also represented.         | Client outcome cases; session full matches; browser results.               | Pending |
| G19 | Complete battles finish      | Engine session and deterministic opponent use legal actions only. | Bounded AI-versus-AI simulations; complete desktop/mobile real-UI battles. | Pending |
| G20 | Fresh Play Again             | Platform entropy, new Session, same content, seeded initiative.   | Client fresh-seed/restart/cleanup test; browser second battle.             | Pending |

### Presentation

| ID  | Requirement                  | Implementation evidence                                          | Verification evidence                                                                   | Status  |
| --- | ---------------------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------- |
| P01 | Desktop battlefield          | Opposing fields, adaptive grid, hand, and log sidebar.           | 1280×800 and 1280×720 browser interactions and inspected captures.                      | Pending |
| P02 | Mobile portrait battlefield  | Compact rows, collapsible hand, dismissible panels.              | 390×844 touch play and inspected captures.                                              | Pending |
| P03 | Mobile landscape battlefield | Responsive compact short-height layout.                          | 844×390 touch play/orientation and inspected capture.                                   | Pending |
| P04 | Responsive layout            | Logical dimensions, automatic scaling, safe-area handling.       | Resize/orientation/DPR browser tests; client layout tests.                              | Pending |
| P05 | Click/tap selection          | Production Bevy input separates activation from scroll gestures. | Desktop pointer and mobile touch interactions.                                          | Pending |
| P06 | Card inspection              | Permitted card preview with dismiss/cancel.                      | Client cancellation atomicity; browser inspect/Escape/close.                            | Pending |
| P07 | Contextual Play/Attack       | Separate preview commands, disabled by legal oracle.             | Client legality tests; browser card preview actions.                                    | Pending |
| P08 | Legal target highlighting    | Current human legal attack commands drive eligible target style. | Client targeting test; inspected targeting screenshot and browser target click.         | Pending |
| P09 | Scrollable crowded fields    | Adaptive grid/rows preserve attacker and scroll positions.       | Crowded-field interaction, touch swipe to offscreen target, inspected overflow capture. | Pending |
| P10 | Persistent Pass              | Action footer outside field/hand scrolling.                      | Browser minimum viewport and crowded-field layout assertions.                           | Pending |
| P11 | Secondary Claim Initiative   | Discoverable secondary menu with legal enabled state.            | Client menu legality; desktop/mobile claim interaction.                                 | Pending |
| P12 | Resource counters            | Permitted ready/exhausted/total counts.                          | Client view reconciliation; browser/capture counter checks.                             | Pending |
| P13 | Resource inspection          | Own identity/readiness; opponent counts only.                    | Privacy/client panel tests; browser resources panel.                                    | Pending |
| P14 | Contextual notifications     | Human-filtered structured events supply feedback.                | Client event consumption; browser observed AI/round feedback.                           | Pending |
| P15 | Persistent event log         | Chronological last 256 human-filtered events.                    | Session bounded-history/privacy tests; desktop/mobile log interactions/capture.         | Pending |
| P16 | Active-player indicator      | Human view phase/actor/legality supplies persistent state.       | Client phase/turn test; browser AI/setup/action transitions.                            | Pending |
| P17 | Normal/Fast/Instant          | 750/150/0 ms presentation scheduling, default Normal.            | Controlled-time client scheduling/pause tests; browser speed menu.                      | Pending |
| P18 | Usable 375×667               | Compact layout keeps all game actions reachable.                 | Minimum-phone full touch flow and visual inspection.                                    | Pending |

### Persistence

| ID  | Requirement             | Implementation evidence                                                         | Verification evidence                                                                                | Status  |
| --- | ----------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------- |
| S01 | Automatic local saves   | Accepted transition persists current session through client store.              | Client injected-store save/retry tests; browser saved progression.                                   | Pending |
| S02 | Browser saves           | `platform/storage/browser.rs`; `low-kingdoms.active.v1` localStorage.           | Real Chromium refresh/resume; browser storage-failure cases.                                         | Pending |
| S03 | Native saves            | `platform/storage/native.rs`; absolute per-user slot and temporary replacement. | Native storage path/round-trip/replacement/failure tests.                                            | Pending |
| S04 | Resume Battle           | Compatible unfinished Session enables Title Resume.                             | Client restore/lifecycle; browser refresh/resume.                                                    | Pending |
| S05 | One active save         | One Store slot; successful replacement updates that slot.                       | Native replacement test; client slot lifecycle.                                                      | Pending |
| S06 | Confirm replacement     | New Battle confirms before replacing unfinished progress.                       | Client confirm/cancel tests; browser confirmation.                                                   | Pending |
| S07 | Clear finished saves    | Completion cleanup; finished restored records are refused for resume.           | Session finished-state restore; client cleanup failure/retry cases.                                  | Pending |
| S08 | Graceful corrupt saves  | Session compatibility/invariant validation plus visible recovery.               | Corrupt/version/content/roles/profile tests; client/browser failure recovery.                        | Pending |
| S09 | Exact continuation      | Snapshot plus command replay verifies exact state/history.                      | `restored_complete_matches_have_identical_commands_events_states_and_outcomes`; browser resume.      | Pending |
| S10 | No duplicate resumed AI | No persisted queue; authoritative `needs_ai` derives pending action.            | `both_pending_setup_orders_restore_without_repeating_opponent_decisions`; client pending-turn tests. | Pending |

### Architecture

| ID  | Requirement                    | Implementation evidence                                                   | Verification evidence                                                  | Status  |
| --- | ------------------------------ | ------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------- |
| A01 | Bevy-independent rules         | Rules depends only on Serde/JSON; battle also excludes Bevy.              | Structure/dependency trees; native/WASM compilation and strict gates.  | Pending |
| A02 | No UI gameplay rules           | Client commands/legal discovery delegate to Session/Game.                 | Production client legality/command tests; source review.               | Pending |
| A03 | Validated AI commands          | AI selects supplied legal command; Session applies via Game.              | AI legal-selection tests and session atomicity.                        | Pending |
| A04 | Deterministic AI               | Explicit priority/tie key; stateless profile version 1.                   | Reordered-legal-list cases; repeated seeded full matches.              | Pending |
| A05 | Structured events              | Session filters EventBatch for human before history/feedback.             | Session private-event tests; client event consumption.                 | Pending |
| A06 | Player-visible state           | `Session::view` always human player zero.                                 | Session view/privacy tests; client reconciliation.                     | Pending |
| A07 | Hidden information             | No AI private cards/order/seed/snapshot/log in UI or bridge.              | Rules/session privacy tests; browser bridge inspection; source review. | Pending |
| A08 | Shared native/WASM gameplay    | Same client plugins and battle library; isolated platform I/O.            | Native matrix; WASM checks; production headless integration.           | Pending |
| A09 | Validated RON                  | Reject unknown fields/refs, duplicate IDs, invalid sizes/Health.          | Content validation and boundary cases.                                 | Pending |
| A10 | Preserve valid state on errors | Atomic Session commands; client pauses and retries failed operation only. | Session invalid-command tests; injected store/AI failure client cases. | Pending |
| A11 | Clean screen transitions       | State-owned roots; singular persistent camera/plugin composition.         | Repeated About/battle/results/resume/restart entity lifecycle tests.   | Pending |
| A12 | Small concrete architecture    | One portable battle crate, private gameplay modules, one Store boundary.  | Dependency/structure gates and source review against 1B scope.         | Pending |

### Verification

| ID  | Requirement                          | Implementation evidence                                                     | Verification evidence                                                                   | Status  |
| --- | ------------------------------------ | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------- |
| T01 | Existing 1A tests pass               | Unchanged complete rules suite and doctest.                                 | Focused rules check and maintained workspace/native CI gates.                           | Pending |
| T02 | AI unit tests pass                   | `crates/battle/tests/content_ai.rs`.                                        | Focused battle and workspace tests.                                                     | Pending |
| T03 | Complete AI tests pass               | Seeded bounded headless AI-versus-AI matches.                               | Focused battle full-match tests.                                                        | Pending |
| T04 | Bevy tests pass                      | Actual production client systems in headless App.                           | `cargo test --locked -p low-kingdoms-client`; workspace/test-support gates.             | Pending |
| T05 | Persistence tests pass               | Session exact restore plus injected/platform storage cases.                 | Focused battle/client tests and browser refresh.                                        | Pending |
| T06 | Desktop browser input passes         | Actual generated WASM with pointer/keyboard gameplay.                       | Playwright desktop 1280×800 and 1280×720.                                               | Pending |
| T07 | Mobile touch passes                  | Actual generated WASM with touch/swipe/orientation gameplay.                | Playwright 390×844, 375×667, and 844×390.                                               | Pending |
| T08 | Refresh/resume passes                | Browser slot survives refresh; real Resume control.                         | Playwright pending-turn and continued-game scenarios.                                   | Pending |
| T09 | Native build passes                  | Pinned native application and libraries.                                    | Linux/macOS/Windows maintained CI matrix; local result separately recorded.             | Pending |
| T10 | WASM build passes                    | Production/test-support application, battle, and rules target gates.        | `just check-web` and CI WASM gate.                                                      | Pending |
| T11 | Strict linting passes                | Workspace all/pedantic/nursery and rustdoc warnings denied.                 | `just check`, explicit supported WASM/feature gates.                                    | Pending |
| T12 | Formatting passes                    | Existing rustfmt and pinned Prettier settings.                              | `just fmt-check`; CI formatting.                                                        | Pending |
| T13 | Native render passes where supported | Real Bevy screenshot completion with bounded failure.                       | `just smoke-native`; CI native-render artifact and inspection.                          | Pending |
| T14 | Browser pixels inspected             | Actual captures of battlefield/cards/panels/targets/log/results.            | Documented manual screenshot review with OS/browser/DPR/evidence paths.                 | Pending |
| T15 | Full CI passes                       | All native, quality, WASM, browser, and native-render jobs at PR head.      | Actual GitHub Actions/check-run URLs and conclusions; deployment remains skipped on PR. | Pending |
| T16 | Complete documentation               | Specification, architecture/development/testing, instructions, this record. | Formatting/structure gate plus complete 76-row evidence/outcome review.                 | Pending |

## Architectural decisions and interpretations

- `low-kingdoms-battle` separates validated content, view-only AI, and sessions from Bevy. The
  client owns rendering/timing/platform I/O; `low-kingdoms-rules` remains the sole match authority.
- Content is compile-time embedded RON, parsed/validated once. Six vanilla named units and two
  twenty-card decks with stronghold Health 30 exercise the approved mechanics without new rules.
- Human identity is player zero, AI player one; starting initiative remains seed-selected. AI
  receives only its own permitted view, legal commands, and public definitions. Explicit tie keys
  avoid accidental command-enumeration dependence; the AI profile is stateless and versioned.
- The view drives hybrid entity reconciliation; filtered structured events drive notifications/log.
  The presentation log retains the last 256 events. The privileged full command log is retained for
  exact replay/restore, so bounded presentation history does not discard authoritative state.
- Version-1 saves carry exact content identity, roles, profile/state, rules snapshot, seed, phase,
  revision, log, and filtered history. Restore replays/validates exact continuation. Finished records
  validate for cleanup but are not resumable. There is no migration or cloud/database layer.
- Protective save bounds are 2 MiB and 4096 successful commands. They fail explicitly rather than
  corrupting a save; the command limit validates before mutation and is not an added gameplay rule.
- Browser storage is origin-local `low-kingdoms.active.v1`; native storage is per-user
  `low-kingdoms/active-v1.json`. A successful gameplay command is never replayed to retry failed I/O.
- Fresh entropy belongs at the client platform edge. Seeds, raw save/snapshot/log data, and opposing
  hidden cards are excluded from player-facing UI, technical diagnostics, and browser snapshots.
- Normal/Fast/Instant delays are 750/150/0 ms. Pause while inactive, execute one AI action at a time,
  and begin fresh scheduling on return without background catch-up.

No Milestone 1A rule interpretation changes are intended. Any actual integration defect and focused
regression must be recorded here rather than silently changing the 17 established decisions.

## Verification record

The pinned compiler is Rust 1.98.0. Local missing capabilities remain blocked even if a provisioned CI runner later passes the corresponding maintained gate. CI and visual results below are pending first branch execution.

| Check                                         | Actual local result                        | Evidence or limitation                                                                                                                                   |
| --------------------------------------------- | ------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Focused rules tests                           | Passed: 28 tests and one doctest           | Existing complete combat, deck-exhaustion, and initiative/passing matches remain unchanged.                                                              |
| Focused battle tests                          | Passed: 24 tests                           | 15 content/AI cases, including 32 complete seeded AI matches; nine session/save cases, including 16 continuation seeds.                                  |
| Rules/battle strict all-target Clippy         | Passed                                     | Clean isolated target directory avoids stale diagnostic replay in the shared cache.                                                                      |
| Battle/rules WASM and rustdoc                 | Passed                                     | Portable modules remain Bevy-independent.                                                                                                                |
| Client production-source headless harness     | Passed: 27 default / 31 test-support tests | Unmodified production source and strict lints; only the scratch harness omits native window backends. This does not replace the maintained native gate.  |
| `just fmt-check` and structure/skill metadata | Passed                                     | Existing rustfmt/Prettier and xtask validation.                                                                                                          |
| `just check-web`                              | Passed                                     | Production and test-support checks plus strict Clippy.                                                                                                   |
| Client WASM tests type-check                  | Passed                                     | `cargo check --locked -p low-kingdoms-client --target wasm32-unknown-unknown --tests --features test-support`.                                           |
| `just build-web`                              | Passed                                     | Production WASM: 62,316,103 bytes; test-support smoke build: 62,443,816 bytes.                                                                           |
| `just check` / `just check-all`               | Blocked locally                            | Missing pkg-config and Wayland development prerequisites; formatting and structure passed before the native build stopped.                               |
| `just smoke-native`                           | Blocked locally                            | Same native prerequisites plus no display/GPU adapter.                                                                                                   |
| `just smoke-web`                              | Blocked locally                            | Both bundles built; all browser launches failed because the pinned Chromium executable is absent. Official CDN installation returned an invalid archive. |
| Maintained Linux/macOS/Windows CI             | Pending                                    | Branch CI will run the original native features and full maintained suite.                                                                               |
| CI browser/native render                      | Pending                                    | Actual screenshots and inspection are required.                                                                                                          |
| Manual interactive local play                 | Blocked locally                            | The cloud browser cannot reach the workspace loopback server; no manual full-game completion is claimed.                                                 |

## Visual evidence and manual review

Record actual native and browser screenshot paths or artifact links here, including mobile portrait.
For each inspected capture, name OS/browser/GPU, viewport/DPR, and the observed readability,
clipping, control accessibility, and any correction. Required views include battlefield, hand,
strongholds, resources, selected/target cards, overflow, preview, log, and results. A successful
capture is not an inspection. Root/subpath and production-without-test-bridge coverage remain required.

Desktop complete playable review: pending. Mobile offscreen-target scrolling, orientation change,
and refresh/resume review: pending. No unexecuted browser, physical phone, or operating-system
runtime is claimed.

## Known limitations and exclusions

Intentional exclusions: card artwork/animation/audio, abilities/keywords, commanders, action/upgrade
cards, factions/affinities, advanced AI/difficulties, deck building/collections, campaigns, networking,
accounts, cloud/database saves, PWA/service worker, and additional gameplay systems. Six vanilla
units and fixed decks are prototype fixtures. The AI is deterministic and basic rather than
competitive. Saves require exact content/profile/version compatibility and have no migration.
Chromium touch emulation does not certify Safari/iOS hardware or physical phone performance.

Incomplete requirements/blockers: maintained CI verification, visual evidence, and complete interactive playable-session review remain pending. Update with concrete remaining defects or
blocked checks instead of treating an intended exclusion as an incomplete requirement.

### Local execution so far

- `cargo test --locked -p low-kingdoms-rules -p low-kingdoms-battle`: passed (28 rules tests, one rules doctest, 24 battle tests).
- Strict all-target Clippy for rules/battle: passed in a clean target directory, avoiding stale diagnostic output in the shared build cache.
- `just check-web`: passed for production and test-support configurations.
- `cargo check --locked -p low-kingdoms-client --target wasm32-unknown-unknown --tests --features test-support`: passed.
- `just fmt-check`: passed; repository structure and local skill metadata passed.
- `just build-web`: passed (62,316,103 WASM bytes before any final presentation refinements).
- `just check` and `just smoke-native`: locally blocked by missing pkg-config/Wayland development prerequisites. No host or native render pass is claimed.
- Local Chromium installation failed on an invalid CDN download. The separate cloud browser could not connect to the workspace loopback server, so manual interactive local review is blocked. CI render artifacts still need execution and visual inspection.
