# Milestone 1B implementation acceptance

The authoritative approved specification is [milestone-1b.md](milestone-1b.md), containing all
33 confirmed decisions from the supplied worker instructions. The specification and implementation
belong to the same feature branch and one implementation PR; no separate planning PR is required.
The original [1A gameplay specification](milestone-1a.md) remains unchanged.

## Status and evidence policy

Milestone status: **incomplete; browser scrolling correction and final verification required**.
Executed headless/architecture/native criteria are marked Passed below. The latest tested code
checkpoint is `a6eddfc5d6193895e09dd0e8fd1485982564b8f7`: all six non-browser CI gates passed.
The inspected maintained browser run completed with **24 passed, one failed, and 30 not run**
out of 55. Full CI is Failed at this checkpoint. The failure revealed a production scroll-bound
defect; its correction/regressions and another complete maintained run are required. Independent desktop
play completed; final minimum-phone, portrait, landscape, and resume interactions were inspected.
These observations supplement rather than replace complete touch matches, offscreen-target input,
and actual background-tab verification in the remaining mobile viewport cases. Compilation, headless execution,
real input, and inspected pixels remain separate evidence claims.

The following focused corrections are verified locally and await the next complete CI run:
scroll drag/wheel input now clamps to Bevy's rendered content bounds before every delta;
real-input regressions cover immediate reversal at DPR 1 and DPR 2; resource cards identify their
selection action correctly; and command failures log static operation/error categories without
private payloads. The production-source headless client harness passed 41 default and 45
`test-support` tests, strict Clippy passed in both modes, and maintained `just check-web` passed.
The browser driver now waits for the human's own submitted resource decision, compares the full
human view when checking presentation-only actions, and searches for offscreen cards in their
actual direction. These corrections do not establish a full browser pass until the rerun succeeds.

## Criterion mapping

### Gameplay

| ID  | Requirement                  | Implementation evidence                                           | Verification evidence                                                                                                                                                     | Status  |
| --- | ---------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| G01 | Start from Title             | Client gameplay start command and Title Play control.             | Production lifecycle test and independent manual desktop Title → Play; final viewport matrix pending.                                                                     | Pending |
| G02 | Use the actual 1A engine     | `battle::session::Session` privately owns `rules::engine::Game`.  | Session atomicity/continuation tests; client production command tests.                                                                                                    | Passed  |
| G03 | Load two valid RON decks     | `assets/content/cards.ron`, `decks.ron`; `battle::content`.       | `tests/content_ai.rs` loading and validation.                                                                                                                             | Passed  |
| G04 | Human mulligan               | Mulligan screen; `Session::apply_human` typed mulligan command.   | `gameplay/tests.rs::play_initializes_real_mulligan_and_saves_without_frame_write_spam`; browser mulligan flow.                                                            | Pending |
| G05 | AI mulligan                  | `battle::ai::choose` setup heuristic; `Session::step_ai`.         | AI mulligan cases; session pending-setup cases.                                                                                                                           | Passed  |
| G06 | Human resources              | Inline legal selection and Confirm Resources.                     | 1A zero/one/two cases, client selection test, and manual desktop resource confirmation; mobile matrix pending.                                                            | Pending |
| G07 | AI resources                 | Deterministic expensive/redundant-card setup decisions.           | AI initial/subsequent resource cases; full simulations.                                                                                                                   | Passed  |
| G08 | Human deploys                | Hand preview contextual Play submits a legal deploy command.      | Production full-match controls and manual desktop deployed/exhausted unit; mobile matrix pending.                                                                         | Pending |
| G09 | AI deploys                   | AI selects affordable validated deploy commands.                  | AI deployment cases; full matches; browser observed action.                                                                                                               | Pending |
| G10 | Human attacks units          | Friendly preview Attack and highlighted legal enemy target.       | Client targeting/combat tests and manual desktop unit combat, including mutual destruction; mobile matrix pending.                                                        | Pending |
| G11 | Human attacks strongholds    | Legal stronghold targeting through the same command path.         | Client stronghold command path and manual desktop direct stronghold damage; mobile matrix pending.                                                                        | Pending |
| G12 | AI attacks both target types | Explicit winning/trade/stronghold priorities.                     | AI combat cases and complete AI matches.                                                                                                                                  | Passed  |
| G13 | Passing                      | Persistent control submits `Command::Pass`.                       | 1A pass cases, production client match, and independent manual desktop Pass; mobile matrix pending.                                                                       | Pending |
| G14 | Claim initiative             | Secondary menu submits `Command::ClaimInitiative`.                | 1A/client cases and two manual desktop claims with continued AI and next initiative zero; mobile matrix pending.                                                          | Pending |
| G15 | Regroup                      | Engine phase/draw batch; inline one-card decision.                | 1A progression cases; client/browser round transitions.                                                                                                                   | Pending |
| G16 | Correct damage display       | Human view statistics drive Power/remaining Health and damage.    | Client damage reconciliation and reviewed manual `desktop-persistent-damage.png` / `desktop-stronghold-damage.png`; mobile review pending.                                | Pending |
| G17 | Correct exhaustion display   | Human view readiness drives text and dimming.                     | Client ready/exhausted tests and inspected `desktop-first-deploy.png`; mobile review pending.                                                                             | Pending |
| G18 | Victory/defeat               | Engine outcome selects Results; Draw is also represented.         | `gameplay/tests.rs::real_rules_outcomes_render_victory_defeat_and_draw_and_clear_the_active_slot` passed; independent manual desktop Defeat and inspected native Results. | Passed  |
| G19 | Complete battles finish      | Engine session and deterministic opponent use legal actions only. | 32 complete AI matches, production controller matches, targeted desktop browser pass, and independent manual desktop completion; mobile completion pending.               | Pending |
| G20 | Fresh Play Again             | Platform entropy, new Session, same content, seeded initiative.   | Production fresh-seed/restart regression passed; manual Play Again produced a fresh hand and seeded initiative zero, then Main Menu exposed Resume.                       | Passed  |

### Presentation

| ID  | Requirement                  | Implementation evidence                                          | Verification evidence                                                                                                                                                                                             | Status  |
| --- | ---------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| P01 | Desktop battlefield          | Opposing fields, adaptive grid, hand, and log sidebar.           | 1280×800 and 1280×720 browser interactions and inspected captures.                                                                                                                                                | Pending |
| P02 | Mobile portrait battlefield  | Compact rows, collapsible hand, dismissible panels.              | 390×844 touch play and inspected captures.                                                                                                                                                                        | Pending |
| P03 | Mobile landscape battlefield | Responsive compact short-height layout.                          | 844×390 touch play/orientation and inspected capture.                                                                                                                                                             | Pending |
| P04 | Responsive layout            | Logical dimensions, automatic scaling, safe-area handling.       | Resize/orientation/DPR browser tests; client layout tests.                                                                                                                                                        | Pending |
| P05 | Click/tap selection          | Production Bevy input separates activation from scroll gestures. | Desktop pointer and mobile touch interactions.                                                                                                                                                                    | Pending |
| P06 | Card inspection              | Permitted card preview with dismiss/cancel.                      | Production cancellation/inspection tests and independent desktop/minimum-phone touch previews inspected; final matrix still verifies repeated flows.                                                              | Passed  |
| P07 | Contextual Play/Attack       | Separate preview commands, disabled by legal oracle.             | Client legality tests; browser card preview actions.                                                                                                                                                              | Pending |
| P08 | Legal target highlighting    | Current human legal attack commands drive eligible target style. | Client targeting test; inspected targeting screenshot and browser target click.                                                                                                                                   | Pending |
| P09 | Scrollable crowded fields    | Adaptive grid/rows preserve attacker and scroll positions.       | Shared raw-scroll bounds defect discovered during portrait resource scrolling; production bounds/regressions are being corrected. Desktop target flow passed, mobile offscreen-target completion remains pending. | Pending |
| P10 | Persistent Pass              | Action footer outside field/hand scrolling.                      | Browser minimum viewport and crowded-field layout assertions.                                                                                                                                                     | Pending |
| P11 | Secondary Claim Initiative   | Discoverable secondary menu with legal enabled state.            | Client menu legality; desktop/mobile claim interaction.                                                                                                                                                           | Pending |
| P12 | Resource counters            | Permitted ready/exhausted/total counts.                          | Production human projection and `gameplay/presentation/`; inspected native counters; browser counter checks pending.                                                                                              | Pending |
| P13 | Resource inspection          | Own identity/readiness; opponent counts only.                    | Landscape manual panel was readable, but maintained 390×844 full-match test failed reaching a resource card through scrolling. Correction and repeated mobile verification required.                              | Failed  |
| P14 | Contextual notifications     | Human-filtered structured events supply feedback.                | Client event consumption; browser observed AI/round feedback.                                                                                                                                                     | Pending |
| P15 | Persistent event log         | Chronological last 256 human-filtered events.                    | Session bounded-history/privacy tests; desktop/mobile log interactions/capture.                                                                                                                                   | Pending |
| P16 | Active-player indicator      | Human view phase/actor/legality supplies persistent state.       | Client phase/turn test; browser AI/setup/action transitions.                                                                                                                                                      | Pending |
| P17 | Normal/Fast/Instant          | 750/150/0 ms presentation scheduling, default Normal.            | Controlled-time client scheduling/pause tests; browser speed menu.                                                                                                                                                | Pending |
| P18 | Usable 375×667               | Compact layout keeps all game actions reachable.                 | Minimum-phone full touch flow and visual inspection.                                                                                                                                                              | Pending |

### Persistence

| ID  | Requirement             | Implementation evidence                                                         | Verification evidence                                                                                                                                                | Status  |
| --- | ----------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| S01 | Automatic local saves   | Accepted transition persists current session through client store.              | Client injected-store save/retry tests; browser saved progression.                                                                                                   | Passed  |
| S02 | Browser saves           | `platform/storage/browser.rs`; `low-kingdoms.active.v1` localStorage.           | Twenty real-WASM storage supplement cases passed across all five viewports; manual desktop setup and portrait regroup refresh/resume succeeded. Full matrix pending. | Passed  |
| S03 | Native saves            | `platform/storage/native.rs`; absolute per-user slot and temporary replacement. | Native storage path/round-trip/replacement/failure tests.                                                                                                            | Passed  |
| S04 | Resume Battle           | Compatible unfinished Session enables Title Resume.                             | Client pending-turn restore passed; real portrait Resume restored round 2/revision 10 Regroup exactly after refresh.                                                 | Passed  |
| S05 | One active save         | One Store slot; successful replacement updates that slot.                       | Native replacement test; client slot lifecycle.                                                                                                                      | Passed  |
| S06 | Confirm replacement     | New Battle confirms before replacing unfinished progress.                       | Client confirm/cancel tests; browser confirmation.                                                                                                                   | Pending |
| S07 | Clear finished saves    | Completion cleanup; finished restored records are refused for resume.           | Session finished-state restore; client cleanup failure/retry cases.                                                                                                  | Passed  |
| S08 | Graceful corrupt saves  | Session compatibility/invariant validation plus visible recovery.               | Native invalid-UTF-8 confirmation/replacement, quota/read/write/clear client regressions, session corruption checks, and 20 browser storage supplement cases passed. | Passed  |
| S09 | Exact continuation      | Snapshot plus command replay verifies exact state/history.                      | Session exact continuation tests passed; manually refreshed portrait Regroup restored exact round 2/revision 10.                                                     | Passed  |
| S10 | No duplicate resumed AI | No persisted queue; authoritative `needs_ai` derives pending action.            | `pending_opponent_action_and_regroup_decisions_resume_exactly_once`; client `refresh_and_resume_preserve_pending_ai_and_do_not_duplicate_accepted_decisions`.        | Passed  |

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

| ID  | Requirement                          | Implementation evidence                                                     | Verification evidence                                                                                                                                           | Status  |
| --- | ------------------------------------ | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| T01 | Existing 1A tests pass               | Unchanged complete rules suite and doctest.                                 | Focused rules check and maintained workspace/native CI gates.                                                                                                   | Passed  |
| T02 | AI unit tests pass                   | `crates/battle/tests/content_ai.rs`.                                        | Focused battle and workspace tests.                                                                                                                             | Passed  |
| T03 | Complete AI tests pass               | Seeded bounded headless AI-versus-AI matches.                               | `content_ai::complete_ai_matches_are_legal_deterministic_and_terminate_across_seeds`: 32 seeded matches.                                                        | Passed  |
| T04 | Bevy tests pass                      | Actual production client systems in headless App.                           | Latest maintained CI: 37 client default tests and 41 test-support tests; production controller, input, presentation, storage, and lifecycle regressions.        | Passed  |
| T05 | Persistence tests pass               | Session exact restore plus injected/platform storage cases.                 | Nine session/save tests, native/client failure recovery, and 20 browser storage supplement cases passed. Full refresh matrix remains T08.                       | Passed  |
| T06 | Desktop browser input passes         | Actual generated WASM with pointer/keyboard gameplay.                       | Both maintained desktop viewport full matches and real background/reload/resume scenarios passed at a6ed; independent manual desktop lifecycle completed.       | Passed  |
| T07 | Mobile touch passes                  | Actual generated WASM with touch/swipe/orientation gameplay.                | Maintained portrait full-match case failed on resource scrolling; minimum-phone/landscape full flows were not run after fail-fast. Fix and full rerun required. | Failed  |
| T08 | Refresh/resume passes                | Browser slot survives refresh; real Resume control.                         | Desktop real pending-AI refresh/resume passed; manual portrait Regroup restore matched round 2/revision 10. Remaining mobile matrix cases not run.              | Pending |
| T09 | Native build passes                  | Pinned native application and libraries.                                    | Linux/macOS/Windows maintained CI matrix; local result separately recorded.                                                                                     | Passed  |
| T10 | WASM build passes                    | Production/test-support application, battle, and rules target gates.        | `just check-web` and CI WASM gate.                                                                                                                              | Passed  |
| T11 | Strict linting passes                | Workspace all/pedantic/nursery and rustdoc warnings denied.                 | `just check`, explicit supported WASM/feature gates.                                                                                                            | Passed  |
| T12 | Formatting passes                    | Existing rustfmt and pinned Prettier settings.                              | `just fmt-check`; CI formatting.                                                                                                                                | Passed  |
| T13 | Native render passes where supported | Real Bevy screenshot completion with bounded failure.                       | `just smoke-native`; CI native-render artifact and inspection.                                                                                                  | Passed  |
| T14 | Browser pixels inspected             | Actual captures of battlefield/cards/panels/targets/log/results.            | Latest four native PNGs plus desktop, minimum-phone, portrait, and landscape captures inspected. Final full-match/targeting matrix captures pending.            | Pending |
| T15 | Full CI passes                       | All native, quality, WASM, browser, and native-render jobs at PR head.      | All six non-browser gates passed; inspected browser matrix ended 24 passed, one failed, 30 not run. Production correction/full rerun pending.                   | Failed  |
| T16 | Complete documentation               | Specification, architecture/development/testing, instructions, this record. | Formatting/structure gate plus complete 76-row evidence/outcome review.                                                                                         | Pending |

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

The latest inspected code checkpoint is `a6eddfc5d6193895e09dd0e8fd1485982564b8f7`:
[PR Actions run 38360813050](https://github.com/matanlurey/low-kingdoms/actions/runs/38360813050)
and [push run 38360807991](https://github.com/matanlurey/low-kingdoms/actions/runs/38360807991).
All six non-browser gates are green: Linux/macOS/Windows native, quality, WASM, and native renderer.
Client tests executed 37 default and 41 test-support cases. Linux/macOS executed 99 workspace tests
plus one doctest; Windows executed 97 plus one doctest (the Unix-only process cases are platform scoped).
The inspected browser run completed with 24 passing cases, one failed portrait resource-scroll
case, and 30 not run after fail-fast. Both desktop full matches and their real background/resume
cases passed. Full CI is Failed at this checkpoint; corrected production input and a full rerun
remain required. Thirty not-run cases are not passing or skipped evidence.

Pinned tools are Rust 1.98.0 and official Chromium 153.0.8010.12. Local native gates remain blocked
by missing host prerequisites. The full Chromium executable is verified, but local maintained
55-case launch is blocked by this environment's UNIX-socket policy: ProcessSingleton terminates
with socket `EPERM`. Headless-shell supplementary runs do not establish real tab-focus behavior.
Twenty storage cases passed across all five viewports under the shell; a 50-case non-background
subset is in progress. Only the maintained full Chromium matrix can establish all 55 cases,
including the five real background-tab cases. No partial suite is labeled a full smoke pass.

| Check                                         | Actual result                                                                                   | Evidence or limitation                                                                                                                |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Focused rules tests                           | Passed: 28 tests and one doctest                                                                | Complete combat, deck-exhaustion, and initiative/passing matches unchanged.                                                           |
| Focused battle tests                          | Passed: 24 tests                                                                                | 15 content/AI cases include 32 seeded full AI matches; nine session/save cases include 16 continuation seeds.                         |
| Rules/battle strict all-target Clippy         | Passed locally and in maintained CI                                                             | Portable modules remain Bevy-independent.                                                                                             |
| Battle/rules WASM and rustdoc                 | Passed locally and in maintained CI                                                             | Production and supported feature gates.                                                                                               |
| Maintained client tests                       | Passed at latest CI checkpoint: 37 default / 41 test-support                                    | Actual production plugins and native features; includes outcome, quota, input/tray, and encoding recovery regressions.                |
| Native workspace tests                        | Passed: Linux/macOS 99 + one doctest; Windows 97 + one doctest                                  | Platform-scoped Unix subprocess tests explain the two-case difference.                                                                |
| `just fmt-check` and structure/skill metadata | Passed at latest checkpoint                                                                     | Existing rustfmt/Prettier and xtask validation; final documentation edits require another check.                                      |
| `just check-web`                              | Passed locally and in latest CI                                                                 | Production/test-support compile and strict Clippy.                                                                                    |
| Client WASM tests type-check                  | Passed locally and in CI verification                                                           | Supported test-support target build.                                                                                                  |
| `just build-web`                              | Passed; measured earlier production WASM 62,316,103 bytes                                       | The measurement precedes final refinements and is not a final-size claim.                                                             |
| `just check`                                  | Blocked locally; latest CI quality passed                                                       | Missing local pkg-config/Wayland prerequisites; maintained CI ran workspace lints/tests/docs.                                         |
| `just smoke-native`                           | Blocked locally; latest CI passed, all four captures inspected                                  | Latest artifact 11794975945; evidence below.                                                                                          |
| `just smoke-web`                              | Blocked locally; inspected maintained CI failed                                                 | Full Chromium local ProcessSingleton policy blocks launch. CI reached 24 passes, then portrait scroll failure; not a full smoke pass. |
| Supplementary browser storage suite           | Passed: 20 cases, five viewports, no failures/skips                                             | `evidence/storage-browser-debug/report.json`; official headless shell, outside the maintained full-channel gate.                      |
| Supplementary non-background browser subset   | Pending: 50 selected cases                                                                      | Does not include the five real background-tab cases and cannot pass the full gate.                                                    |
| `just check-all`                              | Blocked locally                                                                                 | Mandatory native and full browser prerequisites cannot execute here.                                                                  |
| Native CI matrix                              | Passed: Linux/macOS/Windows                                                                     | Actual builds and workspace tests; no interactive macOS/Windows runtime claim.                                                        |
| CI quality/WASM/native-render                 | Passed at latest checkpoint                                                                     | Run links above; latest native evidence inspected.                                                                                    |
| CI browser / full CI                          | Failed at a6ed checkpoint: 24 passed, one failed, 30 not run                                    | Portrait resource-card scrolling exposed raw scroll bounds defect; correction and full 55-case rerun pending.                         |
| Desktop interactive playable review           | Passed: complete manual match, Play Again, Main Menu/Resume                                     | Round-six Defeat, fresh hand/initiative, reviewed desktop captures.                                                                   |
| Mobile interactive review                     | Passed for minimum touch deployment/Pass, orientation, panels, and exact regroup refresh/resume | Full touch-match completion and offscreen target/background cases remain pending in maintained matrix.                                |

## Visual evidence and manual review

Latest native captures from [artifact 11794975945](https://github.com/matanlurey/low-kingdoms/actions/runs/38360813050/artifacts/11794975945)
were downloaded under `evidence/ci/native-final/`. The root worker re-inspected all four PNGs for
code checkpoint `a6eddfc5d6193895e09dd0e8fd1485982564b8f7`. Environment: Ubuntu 24.04,
Xvfb/Openbox, Mesa 25.2.8, llvmpipe LLVM 20.1.2 Vulkan CPU, 1280×720/DPR 1. Text and controls
were readable with no obvious clipping in the reviewed scenes.

| Inspected latest native capture | Observation                                                               |
| ------------------------------- | ------------------------------------------------------------------------- |
| `title.png`                     | Title navigation and visible controls.                                    |
| `mulligan.png`                  | Six opening cards and keep/mulligan controls.                             |
| `battle.png`                    | Deployed battlefield, hand, turn/actions, strongholds/resources, and log. |
| `results.png`                   | Terminal result and restart/menu controls.                                |

Native rendering is established at this checkpoint. Earlier missing Xvfb focus and unsupported
separator glyphs were corrected, with the real inactive-window AI policy preserved. These native
captures do not establish an interactive runtime on macOS/Windows or physical mobile hardware.

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

Play Again was then activated through the real control: the new hand was fresh and seeded opening
initiative was player zero. Main Menu returned to Title and offered Resume for the unfinished new
match. `desktop-play-again.png` and `desktop-returned-title.png` were inspected. These complete
the independent desktop lifecycle review; the final maintained matrix/full-CI conclusion is separate.
Older desktop captures document the match review, while subsequent notification/status fixes are
represented by the latest native and mobile captures rather than obsolete status references.

### Final mobile touch, rotation, and restore review

The root worker used a fresh real touch session at 375×667/DPR 1 in official Chromium's headless
shell with ANGLE/SwiftShader on Ubuntu 24.04.3. Hand opening, card inspection, contextual Play, and
Pass from the open hand tray were accepted through visible controls. Rotation to 844×390/DPR 1
preserved usable battle/panel layout. Resource inspection showed the human's identities/readiness
and both players' public counts; log scroll was intentional and controls fit the viewport.
Resizing to 390×844 was inspected, then refresh and actual Title Resume restored Regroup at exact
round 2/revision 10. All captures below were inspected by the root worker.

| Final manual mobile capture under `evidence/manual/` | Observation                                                     |
| ---------------------------------------------------- | --------------------------------------------------------------- |
| `minimum-final-hand-tray.png`                        | Collapsible bottom hand remains usable at the minimum viewport. |
| `minimum-final-preview.png`                          | Touch-selected readable card preview and contextual Play.       |
| `minimum-final-deployment.png`                       | Touch Play applied a real engine deployment.                    |
| `minimum-pass-from-hand.png`                         | Persistent Pass accepted while the hand was open.               |
| `landscape-final-battle.png`                         | Rotated short-height battle controls fit.                       |
| `landscape-final-resources.png`                      | Own resource identities and both public resource counts.        |
| `landscape-final-log.png`                            | Readable intentional scrolling event history.                   |
| `portrait-final-battle.png`                          | Responsive 390×844 portrait layout.                             |
| `portrait-final-restored-regroup.png`                | Real refresh/Resume restored exact pending regroup state.       |

The full matrix still needs completed results, offscreen target scrolling, high-DPI changes, and
real background-tab pause/reload across the five configured viewports. Root/subpath and production
without the test bridge remain mandatory. No physical phone, Safari/iOS, or final full-CI pass is
claimed from these supplementary shell interactions.

## Known limitations and exclusions

Intentional exclusions: card artwork/animation/audio, abilities/keywords, commanders, action/upgrade
cards, factions/affinities, advanced AI/difficulties, deck building/collections, campaigns, networking,
accounts, cloud/database saves, PWA/service worker, and additional gameplay systems. Six vanilla
units and fixed decks are prototype fixtures. The AI is deterministic and basic rather than
competitive. Saves require exact content/profile/version compatibility and have no migration.
Chromium touch emulation does not certify Safari/iOS hardware or physical phone performance.

Incomplete implementation/verification: portrait resource-card scrolling failed in the maintained
browser run. The raw-scroll bounds correction, focused regressions, and complete 55-case rerun
are still required. Full CI is Failed at the a6ed checkpoint. Six
non-browser gates and latest native inspection passed; independent desktop completion/restart and
manual mobile interaction/rotation/resume are established. Local native checks are blocked by host
prerequisites and local full Chromium by ProcessSingleton UNIX-socket policy. These local blockers
are not successful skips and do not remove the maintained CI requirements.

### First CI correction

PR run `38354248534` rendered title and mulligan with llvmpipe/Mesa Vulkan but the battle fixture timed out at a saved human mulligan decision. Xvfb alone provided no window manager/focus, so the production AI correctly stayed paused. The native render job now starts Openbox inside Xvfb rather than bypassing the inactive-window policy. Inspection also found unsupported middle-dot glyphs in the embedded font; UI separators now use ASCII pipes. These are presentation/test-environment corrections, with no 1A rule changes.

The startup schedule inspection also found that initial state entry precedes Startup save loading. Title now reconciles its Resume control when availability changes, with a production-plugin regression that requires the actual rendered control rather than sending a resume message directly.

Read-only browser-driver review found that landscape mulligan layout changed horizontal overflow without updating the scroll-axis marker. The marker now follows the actual pane axis, with rotated minimum-viewport touch coverage. The compact hand inspection surface is anchored as a collapsible bottom tray to preserve battlefield context, as specified.

### Browser verification corrections

The software-rendered complete desktop scenario took approximately 270 seconds. The previous
300-second scenario deadline and 900-second aggregate budget did not fit the expanded 55-case
suite. The maintained wrapper now allows 3600 seconds, the browser CI job 90 minutes, and complete
match scenarios 600 seconds; ordinary tests retain their bounded waits. This changes execution
budgets without removing gameplay assertions, viewport coverage, or error checks. The full matrix
still needs an actual completed pass.

Same-frame pointer/hand-tray edge cases and native invalid-UTF-8 recovery were corrected in
production and verified by focused regressions in the latest maintained native/quality gates.
Additional explicit Victory/Defeat/Draw and quota-retry tests passed. The latest touch review
confirmed real tray/Play/Pass input. These are input/storage/presentation fixes, with no 1A rule change.

The browser configuration uses supported `channel: "chromium"` full Chromium. For the real
background-tab fixture, Playwright's focus emulation is disabled through CDP on each tab; actual
visibility/focus then controls the production pause policy. No document visibility or game field is
mocked. The two desktop background cases passed in full Chromium at the inspected checkpoint; the
remaining three mobile background cases were not run after the portrait failure. All five must pass
in the complete maintained matrix. Headless-shell supplementary cases cannot establish that behavior.

### Portrait resource-scroll correction

The inspected a6ed browser run passed both desktop complete matches, background pause/restore,
and desktop storage/input cases before failing the 390×844 portrait full-match scenario while
revealing a resource card. Bevy clamps its computed scroll offset for rendering, but the client's raw
`ScrollPosition` could accumulate beyond the actual range. Reversing direction then consumed the
invisible excess rather than immediately moving the visible cards. This is a production input defect,
not a rules-engine rejection or a permitted test skip.

The presentation fix bounds raw scroll positions to the actual content/viewport range and adds
reversal/overflow regressions. The driver also uses bounded directional search and acknowledges
setup commands using the human view's submitted decision/phase, rather than a revision that an
independent AI decision might also advance. These changes preserve real input and existing gameplay
assertions. The corrected branch still needs focused checks and a complete maintained browser run;
24 earlier passes do not establish the 30 cases that did not execute.
