# Milestone 1B implementation acceptance

The authoritative approved specification is [milestone-1b.md](milestone-1b.md), containing all
33 confirmed decisions from the supplied worker instructions. The specification and implementation
belong to the same feature branch and one implementation PR; no separate planning PR is required.
The original [1A gameplay specification](milestone-1a.md) remains unchanged.

## Status and evidence policy

Milestone status: **incomplete while implementation and verification are in progress**.
Headless/architecture criteria with executed evidence are marked Passed below. Player-facing
browser matrix/mobile criteria remain Pending until the maintained suite and required review
succeed. An independently controlled desktop match has now completed and its captures were
inspected; that evidence does not replace the remaining matrix/mobile/CI gates. Evidence
columns identify concrete implementation and test responsibilities; a test existing does not mean
it passed. Compilation, headless execution, browser interaction, and visual inspection are separate
claims. The CI checkpoint below is commit `2fe9bcedee3c9aec36524206bf51aee7cd730be7`; later
presentation/browser refinements still require their own verified head. Full CI is currently Failed
because the checkpoint browser jobs failed, with fixes/reverification in progress.

## Criterion mapping

### Gameplay

| ID  | Requirement                  | Implementation evidence                                           | Verification evidence                                                                                                                                       | Status  |
| --- | ---------------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| G01 | Start from Title             | Client gameplay start command and Title Play control.             | Production lifecycle test and independent manual desktop Title → Play; final viewport matrix pending.                                                       | Pending |
| G02 | Use the actual 1A engine     | `battle::session::Session` privately owns `rules::engine::Game`.  | Session atomicity/continuation tests; client production command tests.                                                                                      | Passed  |
| G03 | Load two valid RON decks     | `assets/content/cards.ron`, `decks.ron`; `battle::content`.       | `tests/content_ai.rs` loading and validation.                                                                                                               | Passed  |
| G04 | Human mulligan               | Mulligan screen; `Session::apply_human` typed mulligan command.   | `gameplay/tests.rs::play_initializes_real_mulligan_and_saves_without_frame_write_spam`; browser mulligan flow.                                              | Pending |
| G05 | AI mulligan                  | `battle::ai::choose` setup heuristic; `Session::step_ai`.         | AI mulligan cases; session pending-setup cases.                                                                                                             | Passed  |
| G06 | Human resources              | Inline legal selection and Confirm Resources.                     | 1A zero/one/two cases, client selection test, and manual desktop resource confirmation; mobile matrix pending.                                              | Pending |
| G07 | AI resources                 | Deterministic expensive/redundant-card setup decisions.           | AI initial/subsequent resource cases; full simulations.                                                                                                     | Passed  |
| G08 | Human deploys                | Hand preview contextual Play submits a legal deploy command.      | Production full-match controls and manual desktop deployed/exhausted unit; mobile matrix pending.                                                           | Pending |
| G09 | AI deploys                   | AI selects affordable validated deploy commands.                  | AI deployment cases; full matches; browser observed action.                                                                                                 | Pending |
| G10 | Human attacks units          | Friendly preview Attack and highlighted legal enemy target.       | Client targeting/combat tests and manual desktop unit combat, including mutual destruction; mobile matrix pending.                                          | Pending |
| G11 | Human attacks strongholds    | Legal stronghold targeting through the same command path.         | Client stronghold command path and manual desktop direct stronghold damage; mobile matrix pending.                                                          | Pending |
| G12 | AI attacks both target types | Explicit winning/trade/stronghold priorities.                     | AI combat cases and complete AI matches.                                                                                                                    | Passed  |
| G13 | Passing                      | Persistent control submits `Command::Pass`.                       | 1A pass cases, production client match, and independent manual desktop Pass; mobile matrix pending.                                                         | Pending |
| G14 | Claim initiative             | Secondary menu submits `Command::ClaimInitiative`.                | 1A/client cases and two manual desktop claims with continued AI and next initiative zero; mobile matrix pending.                                            | Pending |
| G15 | Regroup                      | Engine phase/draw batch; inline one-card decision.                | 1A progression cases; client/browser round transitions.                                                                                                     | Pending |
| G16 | Correct damage display       | Human view statistics drive Power/remaining Health and damage.    | Client damage reconciliation and reviewed manual `desktop-persistent-damage.png` / `desktop-stronghold-damage.png`; mobile review pending.                  | Pending |
| G17 | Correct exhaustion display   | Human view readiness drives text and dimming.                     | Client ready/exhausted tests and inspected `desktop-first-deploy.png`; mobile review pending.                                                               | Pending |
| G18 | Victory/defeat               | Engine outcome selects Results; Draw is also represented.         | Production terminal flow and independent manual desktop round-six Defeat; explicit other outcome labels/final matrix pending.                               | Pending |
| G19 | Complete battles finish      | Engine session and deterministic opponent use legal actions only. | 32 complete AI matches, production controller matches, targeted desktop browser pass, and independent manual desktop completion; mobile completion pending. | Pending |
| G20 | Fresh Play Again             | Platform entropy, new Session, same content, seeded initiative.   | Client fresh-seed/restart/cleanup test; browser second battle.                                                                                              | Pending |

### Presentation

| ID  | Requirement                  | Implementation evidence                                          | Verification evidence                                                                                                | Status  |
| --- | ---------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | ------- |
| P01 | Desktop battlefield          | Opposing fields, adaptive grid, hand, and log sidebar.           | 1280×800 and 1280×720 browser interactions and inspected captures.                                                   | Pending |
| P02 | Mobile portrait battlefield  | Compact rows, collapsible hand, dismissible panels.              | 390×844 touch play and inspected captures.                                                                           | Pending |
| P03 | Mobile landscape battlefield | Responsive compact short-height layout.                          | 844×390 touch play/orientation and inspected capture.                                                                | Pending |
| P04 | Responsive layout            | Logical dimensions, automatic scaling, safe-area handling.       | Resize/orientation/DPR browser tests; client layout tests.                                                           | Pending |
| P05 | Click/tap selection          | Production Bevy input separates activation from scroll gestures. | Desktop pointer and mobile touch interactions.                                                                       | Pending |
| P06 | Card inspection              | Permitted card preview with dismiss/cancel.                      | Client cancellation atomicity; browser inspect/Escape/close.                                                         | Pending |
| P07 | Contextual Play/Attack       | Separate preview commands, disabled by legal oracle.             | Client legality tests; browser card preview actions.                                                                 | Pending |
| P08 | Legal target highlighting    | Current human legal attack commands drive eligible target style. | Client targeting test; inspected targeting screenshot and browser target click.                                      | Pending |
| P09 | Scrollable crowded fields    | Adaptive grid/rows preserve attacker and scroll positions.       | Crowded-field interaction, touch swipe to offscreen target, inspected overflow capture.                              | Pending |
| P10 | Persistent Pass              | Action footer outside field/hand scrolling.                      | Browser minimum viewport and crowded-field layout assertions.                                                        | Pending |
| P11 | Secondary Claim Initiative   | Discoverable secondary menu with legal enabled state.            | Client menu legality; desktop/mobile claim interaction.                                                              | Pending |
| P12 | Resource counters            | Permitted ready/exhausted/total counts.                          | Production human projection and `gameplay/presentation/`; inspected native counters; browser counter checks pending. | Pending |
| P13 | Resource inspection          | Own identity/readiness; opponent counts only.                    | Privacy/client panel tests; browser resources panel.                                                                 | Pending |
| P14 | Contextual notifications     | Human-filtered structured events supply feedback.                | Client event consumption; browser observed AI/round feedback.                                                        | Pending |
| P15 | Persistent event log         | Chronological last 256 human-filtered events.                    | Session bounded-history/privacy tests; desktop/mobile log interactions/capture.                                      | Pending |
| P16 | Active-player indicator      | Human view phase/actor/legality supplies persistent state.       | Client phase/turn test; browser AI/setup/action transitions.                                                         | Pending |
| P17 | Normal/Fast/Instant          | 750/150/0 ms presentation scheduling, default Normal.            | Controlled-time client scheduling/pause tests; browser speed menu.                                                   | Pending |
| P18 | Usable 375×667               | Compact layout keeps all game actions reachable.                 | Minimum-phone full touch flow and visual inspection.                                                                 | Pending |

### Persistence

| ID  | Requirement             | Implementation evidence                                                         | Verification evidence                                                                                                                                         | Status  |
| --- | ----------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| S01 | Automatic local saves   | Accepted transition persists current session through client store.              | Client injected-store save/retry tests; browser saved progression.                                                                                            | Passed  |
| S02 | Browser saves           | `platform/storage/browser.rs`; `low-kingdoms.active.v1` localStorage.           | Actual manual Chromium setup refresh/resume succeeded; final browser matrix and error-path verification pending.                                              | Pending |
| S03 | Native saves            | `platform/storage/native.rs`; absolute per-user slot and temporary replacement. | Native storage path/round-trip/replacement/failure tests.                                                                                                     | Passed  |
| S04 | Resume Battle           | Compatible unfinished Session enables Title Resume.                             | Client restore/lifecycle and actual manual desktop Title Resume after setup refresh; mobile matrix pending.                                                   | Pending |
| S05 | One active save         | One Store slot; successful replacement updates that slot.                       | Native replacement test; client slot lifecycle.                                                                                                               | Passed  |
| S06 | Confirm replacement     | New Battle confirms before replacing unfinished progress.                       | Client confirm/cancel tests; browser confirmation.                                                                                                            | Pending |
| S07 | Clear finished saves    | Completion cleanup; finished restored records are refused for resume.           | Session finished-state restore; client cleanup failure/retry cases.                                                                                           | Passed  |
| S08 | Graceful corrupt saves  | Session compatibility/invariant validation plus visible recovery.               | Session corrupt/version/content/roles/profile cases; client corrupt/read/write/cleanup recovery cases. Browser recovery not separately executed.              | Passed  |
| S09 | Exact continuation      | Snapshot plus command replay verifies exact state/history.                      | `restored_complete_matches_have_identical_commands_events_states_and_outcomes`; browser resume.                                                               | Passed  |
| S10 | No duplicate resumed AI | No persisted queue; authoritative `needs_ai` derives pending action.            | `pending_opponent_action_and_regroup_decisions_resume_exactly_once`; client `refresh_and_resume_preserve_pending_ai_and_do_not_duplicate_accepted_decisions`. | Passed  |

### Architecture

| ID  | Requirement                    | Implementation evidence                                                   | Verification evidence                                                                                           | Status  |
| --- | ------------------------------ | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------- |
| A01 | Bevy-independent rules         | Rules depends only on Serde/JSON; battle also excludes Bevy.              | Structure/dependency trees; native/WASM compilation and strict gates.                                           | Passed  |
| A02 | No UI gameplay rules           | Client commands/legal discovery delegate to Session/Game.                 | Production client legality/command tests; source review.                                                        | Passed  |
| A03 | Validated AI commands          | AI selects supplied legal command; Session applies via Game.              | AI legal-selection tests and session atomicity.                                                                 | Passed  |
| A04 | Deterministic AI               | Explicit priority/tie key; stateless profile version 1.                   | Reordered-legal-list cases; repeated seeded full matches.                                                       | Passed  |
| A05 | Structured events              | Session filters EventBatch for human before history/feedback.             | Session private-event tests; client event consumption.                                                          | Passed  |
| A06 | Player-visible state           | `Session::view` always human player zero.                                 | Session view/privacy tests; client reconciliation.                                                              | Passed  |
| A07 | Hidden information             | No AI private cards/order/seed/snapshot/log in UI or bridge.              | Rules/session privacy tests; browser bridge inspection; source review.                                          | Pending |
| A08 | Shared native/WASM gameplay    | Same client plugins and battle library; isolated platform I/O.            | Native matrix; WASM checks; production headless integration.                                                    | Passed  |
| A09 | Validated RON                  | Reject unknown fields/refs, duplicate IDs, invalid sizes/Health.          | Content validation and boundary cases.                                                                          | Passed  |
| A10 | Preserve valid state on errors | Atomic Session commands; client pauses and retries failed operation only. | Session invalid-command/changed-content AI atomicity; client injected read/write/clear failures and safe retry. | Passed  |
| A11 | Clean screen transitions       | State-owned roots; singular persistent camera/plugin composition.         | Repeated About/battle/results/resume/restart entity lifecycle tests.                                            | Passed  |
| A12 | Small concrete architecture    | One portable battle crate, private gameplay modules, one Store boundary.  | Dependency/structure gates and source review against 1B scope.                                                  | Passed  |

### Verification

| ID  | Requirement                          | Implementation evidence                                                     | Verification evidence                                                                                                             | Status                |
| --- | ------------------------------------ | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| T01 | Existing 1A tests pass               | Unchanged complete rules suite and doctest.                                 | Focused rules check and maintained workspace/native CI gates.                                                                     | Passed                |
| T02 | AI unit tests pass                   | `crates/battle/tests/content_ai.rs`.                                        | Focused battle and workspace tests.                                                                                               | Passed                |
| T03 | Complete AI tests pass               | Seeded bounded headless AI-versus-AI matches.                               | `content_ai::complete_ai_matches_are_legal_deterministic_and_terminate_across_seeds`: 32 seeded matches.                          | Passed                |
| T04 | Bevy tests pass                      | Actual production client systems in headless App.                           | Maintained CI workspace and test-support gates; production controller/presentation tests in `src/gameplay/`.                      | Passed                |
| T05 | Persistence tests pass               | Session exact restore plus injected/platform storage cases.                 | Nine session/save tests and native storage/client injected-store tests passed; browser refresh remains T08.                       | Passed                |
| T06 | Desktop browser input passes         | Actual generated WASM with pointer/keyboard gameplay.                       | Targeted complete desktop scenario passed (~4.5 min); independent manual match completed. Maintained full desktop matrix pending. | Pending               |
| T07 | Mobile touch passes                  | Actual generated WASM with touch/swipe/orientation gameplay.                | Playwright 390×844, 375×667, and 844×390.                                                                                         | Pending               |
| T08 | Refresh/resume passes                | Browser slot survives refresh; real Resume control.                         | Playwright pending-turn and continued-game scenarios.                                                                             | Pending               |
| T09 | Native build passes                  | Pinned native application and libraries.                                    | Linux/macOS/Windows maintained CI matrix; local result separately recorded.                                                       | Passed                |
| T10 | WASM build passes                    | Production/test-support application, battle, and rules target gates.        | `just check-web` and CI WASM gate.                                                                                                | Passed                |
| T11 | Strict linting passes                | Workspace all/pedantic/nursery and rustdoc warnings denied.                 | `just check`, explicit supported WASM/feature gates.                                                                              | Passed                |
| T12 | Formatting passes                    | Existing rustfmt and pinned Prettier settings.                              | `just fmt-check`; CI formatting.                                                                                                  | Passed                |
| T13 | Native render passes where supported | Real Bevy screenshot completion with bounded failure.                       | `just smoke-native`; CI native-render artifact and inspection.                                                                    | Passed                |
| T14 | Browser pixels inspected             | Actual captures of battlefield/cards/panels/targets/log/results.            | Four native scenes and eight independent desktop captures inspected; final browser/mobile captures remain pending.                | Pending               |
| T15 | Full CI passes                       | All native, quality, WASM, browser, and native-render jobs at PR head.      | Actual GitHub Actions/check-run URLs and conclusions; deployment remains skipped on PR.                                           | Failed; rerun pending |
| T16 | Complete documentation               | Specification, architecture/development/testing, instructions, this record. | Formatting/structure gate plus complete 76-row evidence/outcome review.                                                           | Pending               |

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

Pinned Rust is 1.98.0. The inspected checkpoint is
[PR Actions run 38355319747](https://github.com/matanlurey/low-kingdoms/actions/runs/38355319747)
and its separate [push run 38355315364](https://github.com/matanlurey/low-kingdoms/actions/runs/38355315364),
both for commit `2fe9bcedee3c9aec36524206bf51aee7cd730be7`. Native matrix, quality, WASM, and
native-render gates passed in the push run. PR Windows build/tests passed, with its final cache
step still pending at this checkpoint. Both browser jobs failed; their artifacts/logs are being
investigated. No full-CI or browser pass is claimed.

Local missing native prerequisites remain blocked even though provisioned CI passed those gates.
The original local browser install failed, but the exact pinned official Chromium is now installed
and starts: version 153.0.8010.12, archive SHA-256
`ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e`.
A targeted complete desktop browser scenario passed in approximately 4.5 minutes using
SwiftShader. The maintained full 55-case smoke matrix and final-head browser CI remain pending;
a targeted pass is not a full-suite pass.

| Check                                         | Actual result                                                                 | Evidence or limitation                                                                                                                          |
| --------------------------------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Focused rules tests                           | Passed locally: 28 tests and one doctest                                      | Complete combat, deck-exhaustion, and initiative/passing matches unchanged.                                                                     |
| Focused battle tests                          | Passed locally: 24 tests                                                      | 15 content/AI cases include 32 seeded full AI matches; nine session/save cases include 16 continuation seeds.                                   |
| Rules/battle strict all-target Clippy         | Passed locally                                                                | Clean isolated target directory; no stale shared-cache diagnostic claims.                                                                       |
| Battle/rules WASM and rustdoc                 | Passed locally                                                                | Bevy-independent portable modules.                                                                                                              |
| Client production-source headless harness     | Passed locally: 27 default / 31 test-support at the recorded local checkpoint | Scratch harness uses unmodified production source, omitting native window backends; maintained CI separately ran real workspace configurations. |
| `just fmt-check` and structure/skill metadata | Passed locally and at CI checkpoint                                           | Existing rustfmt/Prettier and xtask structure validation. Later edits require another formatting check.                                         |
| `just check-web`                              | Passed locally and at CI checkpoint                                           | Production/test-support compile and strict Clippy.                                                                                              |
| Client WASM tests type-check                  | Passed locally                                                                | `cargo check --locked -p low-kingdoms-client --target wasm32-unknown-unknown --tests --features test-support`.                                  |
| `just build-web`                              | Passed locally at the measured checkpoint                                     | Production WASM 62,316,103 bytes; support smoke build 62,443,816 bytes before final presentation refinements.                                   |
| `just check`                                  | Blocked locally; CI quality passed                                            | Missing local pkg-config/Wayland prerequisites. CI ran strict workspace lints/tests/docs in both supported configurations.                      |
| `just smoke-native`                           | Blocked locally; CI passed and captures inspected                             | Four real native scenes in artifact 11792408329; see visual evidence below.                                                                     |
| `just smoke-web`                              | Earlier local attempt blocked; current rerun pending. Checkpoint CI failed    | Earlier launches lacked Chromium; pinned official Chromium now starts. Actual CI browser failures remain under investigation.                   |
| `just check-all`                              | Blocked locally                                                               | Mandatory native gate cannot execute in the local environment; do not substitute a successful WASM/headless check.                              |
| Maintained native CI matrix                   | Passed at push checkpoint: Linux/macOS/Windows                                | Actual builds and workspace tests; no interactive macOS/Windows runtime claim.                                                                  |
| CI quality/WASM/native-render                 | Passed at checkpoint                                                          | Run links above; native evidence downloaded and inspected.                                                                                      |
| CI browser                                    | Failed at checkpoint; correction/rerun pending                                | Both push and PR browser runs failed.                                                                                                           |
| Full CI                                       | Failed at checkpoint                                                          | Browser gates must pass at the final PR head.                                                                                                   |
| Desktop interactive playable review           | Passed through a complete rendered match; restart/menu follow-up pending      | Independent manual mouse choices reached round 6, revision 61, Defeat; reviewed capture paths below.                                            |
| Mobile interactive playable review            | Pending                                                                       | Touch offscreen targeting, orientation, refresh/resume, and full completion still require final review.                                         |

## Visual evidence and manual review

Native captures from [artifact 11792408329](https://github.com/matanlurey/low-kingdoms/actions/runs/38355319747/artifacts/11792408329)
were downloaded under `evidence/ci/native-focused/` and all four PNGs were inspected by the root
worker. Environment: Ubuntu 24.04, Xvfb/Openbox, llvmpipe LLVM 20.1.2, Mesa 25.2.8 Vulkan CPU,
1280×720 logical viewport at DPR 1.

| Inspected native capture | Observation                                                                             |
| ------------------------ | --------------------------------------------------------------------------------------- |
| `title.png`              | Title navigation and visible controls rendered with the embedded font.                  |
| `mulligan.png`           | Six opening cards and keep/mulligan controls rendered.                                  |
| `battle.png`             | Real deployed battlefield, hand, turn/actions, strongholds/resources, and log rendered. |
| `results.png`            | Terminal result and restart/menu controls rendered.                                     |

These captures establish native rendering at the stated checkpoint; they do not establish a
manual full match or browser/mobile usability. The earlier render run exposed missing Xvfb focus
and unsupported separator glyphs; those were corrected and the focused four-scene run passed.

### Independent desktop playable review

The root worker completed a real rendered match with manually chosen mouse interactions at
1280×720/DPR 1 using the official Chromium 153.0.8010.12 headless shell on Ubuntu 24.04.3,
ANGLE/SwiftShader. The match ended in Defeat at round 6, revision 61, with no page or console errors;
the completed slot was not resumable. Actions were chosen independently of the scripted full-match
browser driver and submitted through visible production controls, not a command/mutation hook.

The review exercised initial resourcing, unit deployment/inspection, unit targeting, exhausted
retaliation and mutual destruction, persistent damage, direct stronghold damage, passing, and two
initiative claims. The AI continued after the human claimed, and the following initiative belonged
to player zero. Setup was refreshed and resumed through the real Title Resume control.

| Reviewed desktop capture under `evidence/manual/` | Evidence                                               |
| ------------------------------------------------- | ------------------------------------------------------ |
| `desktop-mulligan.png`                            | Opening cards and mulligan/keep controls.              |
| `desktop-resourcing.png`                          | Inline resource selection and confirmation.            |
| `desktop-unit-preview.png`                        | Card inspection and contextual actions.                |
| `desktop-first-deploy.png`                        | Actual deployment with exhausted readiness text.       |
| `desktop-targeting.png`                           | Chosen attacker and legal target presentation.         |
| `desktop-persistent-damage.png`                   | Remaining Health, accumulated damage, and retaliation. |
| `desktop-stronghold-damage.png`                   | Direct stronghold attack and updated Health.           |
| `desktop-manual-results.png`                      | Round-six terminal Defeat and results controls.        |

Play Again/Main Menu follow-up and the full desktop test matrix still require their recorded
completion. This independent desktop match establishes an actual complete playable session;
it does not claim mobile play or final full-CI success.

### Remaining browser/mobile review

Record final successful 1280×800/1280×720 desktop, 390×844 portrait,
375×667 minimum, and 844×390 landscape capture paths, browser/GPU/DPR, and observed readability,
clipping, and control accessibility. Required reviewed states include battlefield, hand, strongholds,
resources/inspection, selected/target cards, offscreen targeting, preview, log, and results.
A PNG capture is not itself an inspection. Root/subpath and production-without-bridge coverage
remain required.

Desktop complete playable review: passed for the independent round-six match above; restart/menu
follow-up and final matrix remain pending. Mobile offscreen-target scrolling, orientation change,
and refresh/resume review: pending. No unexecuted browser, physical phone, or operating-system
runtime is claimed.

## Known limitations and exclusions

Intentional exclusions: card artwork/animation/audio, abilities/keywords, commanders, action/upgrade
cards, factions/affinities, advanced AI/difficulties, deck building/collections, campaigns, networking,
accounts, cloud/database saves, PWA/service worker, and additional gameplay systems. Six vanilla
units and fixed decks are prototype fixtures. The AI is deterministic and basic rather than
competitive. Saves require exact content/profile/version compatibility and have no migration.
Chromium touch emulation does not certify Safari/iOS hardware or physical phone performance.

Incomplete requirements/blockers: both checkpoint browser jobs failed. Final-head browser/CI
verification, final browser/mobile visual inspection, mobile playable-session review, and desktop
restart/menu follow-up remain pending. Native render evidence is complete for the stated checkpoint. Update with concrete remaining defects or
blocked checks instead of treating an intended exclusion as an incomplete requirement.

### Local execution so far

- `cargo test --locked -p low-kingdoms-rules -p low-kingdoms-battle`: passed (28 rules tests, one rules doctest, 24 battle tests).
- Strict all-target Clippy for rules/battle: passed in a clean target directory, avoiding stale diagnostic output in the shared build cache.
- `just check-web`: passed for production and test-support configurations.
- `cargo check --locked -p low-kingdoms-client --target wasm32-unknown-unknown --tests --features test-support`: passed.
- `just fmt-check`: passed; repository structure and local skill metadata passed.
- `just build-web`: passed (62,316,103 WASM bytes before any final presentation refinements).
- `just check` and `just smoke-native`: locally blocked by missing pkg-config/Wayland development prerequisites. No host or native render pass is claimed.
- The first local Chromium installation failed on an invalid CDN download; the pinned official browser has since been recovered and starts. A targeted full desktop scenario passed in about 4.5 minutes; the maintained 55-case smoke matrix remains pending. Native captures and the independent desktop match captures have been inspected. No cloud-browser, mobile-completion, or final-CI pass is claimed.

### First CI correction

PR run `38354248534` rendered title and mulligan with llvmpipe/Mesa Vulkan but the battle fixture timed out at a saved human mulligan decision. Xvfb alone provided no window manager/focus, so the production AI correctly stayed paused. The native render job now starts Openbox inside Xvfb rather than bypassing the inactive-window policy. Inspection also found unsupported middle-dot glyphs in the embedded font; UI separators now use ASCII pipes. These are presentation/test-environment corrections, with no 1A rule changes.

The startup schedule inspection also found that initial state entry precedes Startup save loading. Title now reconciles its Resume control when availability changes, with a production-plugin regression that requires the actual rendered control rather than sending a resume message directly.

Read-only browser-driver review found that landscape mulligan layout changed horizontal overflow without updating the scroll-axis marker. The marker now follows the actual pane axis, with rotated minimum-viewport touch coverage. The compact hand inspection surface is anchored as a collapsible bottom tray to preserve battlefield context, as specified.

### Browser verification corrections in progress

The software-rendered complete desktop scenario took approximately 270 seconds. The previous
300-second scenario deadline and 900-second aggregate budget did not fit the expanded 55-case
suite. The maintained wrapper now allows 3600 seconds, the browser CI job 90 minutes, and complete
match scenarios 600 seconds; ordinary tests retain their bounded waits. This changes execution
budgets without removing gameplay assertions, viewport coverage, or error checks. The full matrix
still needs an actual completed pass.

Review also found same-frame pointer/hand-tray interaction edge cases and native invalid-UTF-8 save
recovery. Production corrections and focused regressions are in progress. Do not treat those paths
as verified merely because the ordinary manual desktop match completed. No 1A rule change is
required for these input/storage corrections.
