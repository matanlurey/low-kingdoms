# Milestone 1B implementation acceptance

The authoritative approved specification is [milestone-1b.md](milestone-1b.md), containing all
33 confirmed decisions from the supplied worker instructions. The specification and implementation
belong to the same feature branch and one implementation PR; no separate planning PR is required.
The original [1A gameplay specification](milestone-1a.md) remains unchanged.

## Status and evidence policy

Milestone status: **incomplete pending final current-source rendering and full CI**.
Last published source `3cf7d0f050f7966c7ee5edd5d3750a8d47a09f92` passed all six non-browser gates
but its browser attempt ended 33 passed/one failed/21 not run. That failure is historical: the
rotated Mulligan alignment correction now passes actual maintained root/subpath lifecycle routes,
geometry regressions, and local production-source strict gates. Previously failing individual
responsive/inspection criteria are Passed below. The next-head full 55, two real mobile background
cases, and new native captures remain Pending. Current Mulligan wrapping/vertical alignment changed,
so earlier native frames cannot establish its final pixels. No complete-CI result is claimed yet.

## Criterion mapping

### Gameplay

| ID  | Requirement                  | Implementation evidence                                           | Verification evidence                                                                                                                                                     | Status |
| --- | ---------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| G01 | Start from Title             | Client gameplay start command and Title Play control.             | Real Title Play paths executed on desktop and all three touch phone viewports, followed by full matches.                                                                  | Passed |
| G02 | Use the actual 1A engine     | `battle::session::Session` privately owns `rules::engine::Game`.  | Session atomicity/continuation tests; client production command tests.                                                                                                    | Passed |
| G03 | Load two valid RON decks     | `assets/content/cards.ron`, `decks.ron`; `battle::content`.       | `tests/content_ai.rs` loading and validation.                                                                                                                             | Passed |
| G04 | Human mulligan               | Mulligan screen; `Session::apply_human` typed mulligan command.   | Production mulligan regression and maintained desktop/portrait root/subpath mulligan flows passed; dedicated phone opening screens inspected.                             | Passed |
| G05 | AI mulligan                  | `battle::ai::choose` setup heuristic; `Session::step_ai`.         | AI mulligan cases; session pending-setup cases.                                                                                                                           | Passed |
| G06 | Human resources              | Inline legal selection and Confirm Resources.                     | Production zero/one/two/selection cases and all three phone resource confirmation/reversal flows passed.                                                                  | Passed |
| G07 | AI resources                 | Deterministic expensive/redundant-card setup decisions.           | AI initial/subsequent resource cases; full simulations.                                                                                                                   | Passed |
| G08 | Human deploys                | Hand preview contextual Play submits a legal deploy command.      | Production full matches and all three actual touch phone deployments passed, including quick card Play.                                                                   | Passed |
| G09 | AI deploys                   | AI selects affordable validated deploy commands.                  | All completed real-UI matches observed opponent deployments; deterministic AI cases passed.                                                                               | Passed |
| G10 | Human attacks units          | Friendly preview Attack and highlighted legal enemy target.       | Production combat/targeting tests and all three completed touch matches attacked units; portrait/minimum offscreen targeting captures inspected.                          | Passed |
| G11 | Human attacks strongholds    | Legal stronghold targeting through the same command path.         | Completed desktop and all three touch phone matches executed direct stronghold attacks.                                                                                   | Passed |
| G12 | AI attacks both target types | Explicit winning/trade/stronghold priorities.                     | AI combat cases and complete AI matches.                                                                                                                                  | Passed |
| G13 | Passing                      | Persistent control submits `Command::Pass`.                       | Production and full rendered match Pass passed; focused phone quick Pass accepted from the hand tray.                                                                     | Passed |
| G14 | Claim initiative             | Secondary menu submits `Command::ClaimInitiative`.                | Full rendered desktop/phone claim actions passed; manual desktop verified continued AI actions and next-round initiative zero.                                            | Passed |
| G15 | Regroup                      | Engine phase/draw batch; inline one-card decision.                | Production regroup and all completed rendered phone round transitions/confirmation passed.                                                                                | Passed |
| G16 | Correct damage display       | Human view statistics drive Power/remaining Health and damage.    | Production damage reconciliation and root-reviewed phone combat/remaining Health/damage captures.                                                                         | Passed |
| G17 | Correct exhaustion display   | Human view readiness drives text and dimming.                     | Production readiness text plus root-reviewed exhausted deployment/combat captures at phone widths and DPR 2.                                                              | Passed |
| G18 | Victory/defeat               | Engine outcome selects Results; Draw is also represented.         | `gameplay/tests.rs::real_rules_outcomes_render_victory_defeat_and_draw_and_clear_the_active_slot` passed; independent manual desktop Defeat and inspected native Results. | Passed |
| G19 | Complete battles finish      | Engine session and deterministic opponent use legal actions only. | 32 headless AI matches, production controller matches, complete desktop play, and three real-touch phone matches all reached terminal outcomes.                           | Passed |
| G20 | Fresh Play Again             | Platform entropy, new Session, same content, seeded initiative.   | Production fresh-seed/restart regression passed; manual Play Again produced a fresh hand and seeded initiative zero, then Main Menu exposed Resume.                       | Passed |

### Presentation

| ID  | Requirement                  | Implementation evidence                                          | Verification evidence                                                                                                                                             | Status |
| --- | ---------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| P01 | Desktop battlefield          | Opposing fields, adaptive grid, hand, and log sidebar.           | Maintained desktop full flows passed at both sizes; root inspected actual native/desktop captures.                                                                | Passed |
| P02 | Mobile portrait battlefield  | Compact rows, collapsible hand, dismissible panels.              | Completed 390×844 touch match and root-reviewed portrait crowded targeting, damage, and results.                                                                  | Passed |
| P03 | Mobile landscape battlefield | Responsive compact short-height layout.                          | 844×390/DPR 2 complete touch match passed; corrected 667×375 rotated Mulligan first/last inspection passed both maintained root/subpath routes.                   | Passed |
| P04 | Responsive layout            | Logical dimensions, automatic scaling, safe-area handling.       | Completed resize/rotation/DPR flows plus two real Bevy/Taffy geometry regressions for reachable x/y overflow origins and fitting-row centering at DPR 1/2 passed. | Passed |
| P05 | Click/tap selection          | Production Bevy input separates activation from scroll gestures. | Actual mouse and touch input completed matches; focused phone tap/deploy/quick Pass regressions passed.                                                           | Passed |
| P06 | Card inspection              | Permitted card preview with dismiss/cancel.                      | Corrected rotated first/last opening-card inspection passed both routes in 18.3/18.6 s; full human view and own pending decision remained unchanged.              | Passed |
| P07 | Contextual Play/Attack       | Separate preview commands, disabled by legal oracle.             | Real contextual Play/Attack operated through all completed desktop/phone flows; legal-control regressions passed.                                                 | Passed |
| P08 | Legal target highlighting    | Current human legal attack commands drive eligible target style. | Actual targeting and attack acceptance passed; crowded portrait/minimum targeting captures inspected.                                                             | Passed |
| P09 | Scrollable crowded fields    | Adaptive grid/rows preserve attacker and scroll positions.       | Corrected clamped raw scroll and immediate reversal regressions passed; focused resource middle-card/reversal and actual offscreen touch targeting passed.        | Passed |
| P10 | Persistent Pass              | Action footer outside field/hand scrolling.                      | Focused quick Pass from open hand passed in all three phone views; DPR 2 hand/footer reviewed.                                                                    | Passed |
| P11 | Secondary Claim Initiative   | Discoverable secondary menu with legal enabled state.            | Completed phone match drivers accessed Claim Initiative through More; actual controls and results inspected.                                                      | Passed |
| P12 | Resource counters            | Permitted ready/exhausted/total counts.                          | Root-reviewed native/phone counters and resources panels; full-match view/readiness assertions passed.                                                            | Passed |
| P13 | Resource inspection          | Own identity/readiness; opponent counts only.                    | Historical portrait failure fixed; all three focused edge-reversal/middle-card inspections passed, with own-only resource identities and public counts inspected. | Passed |
| P14 | Contextual notifications     | Human-filtered structured events supply feedback.                | Structured notifications/history asserted in completed rendered matches; subsequent corrected status captures inspected.                                          | Passed |
| P15 | Persistent event log         | Chronological last 256 human-filtered events.                    | All completed phone flows opened/closed chronological log; root reviewed landscape log and scroll.                                                                | Passed |
| P16 | Active-player indicator      | Human view phase/actor/legality supplies persistent state.       | Production phase/actor tests and completed phone setup/action/regroup/results flows passed.                                                                       | Passed |
| P17 | Normal/Fast/Instant          | 750/150/0 ms presentation scheduling, default Normal.            | Production controlled-time 750/150/0 ms tests and rendered speed changes passed. Real mobile background-tab checks remain part of pending full CI.                | Passed |
| P18 | Usable 375×667               | Compact layout keeps all game actions reachable.                 | Complete 375×667 portrait match and focused controls passed; the separate 667×375 rotated Mulligan defect is recorded under P03/P04/P06.                          | Passed |

### Persistence

| ID  | Requirement             | Implementation evidence                                                         | Verification evidence                                                                                                                                                | Status |
| --- | ----------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| S01 | Automatic local saves   | Accepted transition persists current session through client store.              | Client injected-store save/retry tests; browser saved progression.                                                                                                   | Passed |
| S02 | Browser saves           | `platform/storage/browser.rs`; `low-kingdoms.active.v1` localStorage.           | Twenty real-WASM storage supplement cases passed across all five viewports; manual desktop setup and portrait regroup refresh/resume succeeded. Full matrix pending. | Passed |
| S03 | Native saves            | `platform/storage/native.rs`; absolute per-user slot and temporary replacement. | Native storage path/round-trip/replacement/failure tests.                                                                                                            | Passed |
| S04 | Resume Battle           | Compatible unfinished Session enables Title Resume.                             | Client pending-turn restore passed; real portrait Resume restored round 2/revision 10 Regroup exactly after refresh.                                                 | Passed |
| S05 | One active save         | One Store slot; successful replacement updates that slot.                       | Native replacement test; client slot lifecycle.                                                                                                                      | Passed |
| S06 | Confirm replacement     | New Battle confirms before replacing unfinished progress.                       | Client confirmation/cancel regressions, corrupt replacement browser cases, and maintained desktop root/subpath replacement flows passed.                             | Passed |
| S07 | Clear finished saves    | Completion cleanup; finished restored records are refused for resume.           | Session finished-state restore; client cleanup failure/retry cases.                                                                                                  | Passed |
| S08 | Graceful corrupt saves  | Session compatibility/invariant validation plus visible recovery.               | Native invalid-UTF-8 confirmation/replacement, quota/read/write/clear client regressions, session corruption checks, and 20 browser storage supplement cases passed. | Passed |
| S09 | Exact continuation      | Snapshot plus command replay verifies exact state/history.                      | Session exact continuation tests passed; manually refreshed portrait Regroup restored exact round 2/revision 10.                                                     | Passed |
| S10 | No duplicate resumed AI | No persisted queue; authoritative `needs_ai` derives pending action.            | `pending_opponent_action_and_regroup_decisions_resume_exactly_once`; client `refresh_and_resume_preserve_pending_ai_and_do_not_duplicate_accepted_decisions`.        | Passed |

### Architecture

| ID  | Requirement                    | Implementation evidence                                                   | Verification evidence                                                                                                                         | Status |
| --- | ------------------------------ | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| A01 | Bevy-independent rules         | Rules depends only on Serde/JSON; battle also excludes Bevy.              | Structure/dependency trees; native/WASM compilation and strict gates.                                                                         | Passed |
| A02 | No UI gameplay rules           | Client commands/legal discovery delegate to Session/Game.                 | Production client legality/command tests; source review.                                                                                      | Passed |
| A03 | Validated AI commands          | AI selects supplied legal command; Session applies via Game.              | AI legal-selection tests and session atomicity.                                                                                               | Passed |
| A04 | Deterministic AI               | Explicit priority/tie key; stateless profile version 1.                   | Reordered-legal-list cases; repeated seeded full matches.                                                                                     | Passed |
| A05 | Structured events              | Session filters EventBatch for human before history/feedback.             | Session private-event tests; client event consumption.                                                                                        | Passed |
| A06 | Player-visible state           | `Session::view` always human player zero.                                 | Session view/privacy tests; client reconciliation.                                                                                            | Passed |
| A07 | Hidden information             | No AI private cards/order/seed/snapshot/log in UI or bridge.              | Rules/session privacy and live browser human-view/frozen-bridge assertions passed in completed phone flows; production hooks remain excluded. | Passed |
| A08 | Shared native/WASM gameplay    | Same client plugins and battle library; isolated platform I/O.            | Native matrix; WASM checks; production headless integration.                                                                                  | Passed |
| A09 | Validated RON                  | Reject unknown fields/refs, duplicate IDs, invalid sizes/Health.          | Content validation and boundary cases.                                                                                                        | Passed |
| A10 | Preserve valid state on errors | Atomic Session commands; client pauses and retries failed operation only. | Session invalid-command/changed-content AI atomicity; client injected read/write/clear failures and safe retry.                               | Passed |
| A11 | Clean screen transitions       | State-owned roots; singular persistent camera/plugin composition.         | Repeated About/battle/results/resume/restart entity lifecycle tests.                                                                          | Passed |
| A12 | Small concrete architecture    | One portable battle crate, private gameplay modules, one Store boundary.  | Dependency/structure gates and source review against 1B scope.                                                                                | Passed |

### Verification

| ID  | Requirement                          | Implementation evidence                                                     | Verification evidence                                                                                                                                                                                          | Status  |
| --- | ------------------------------------ | --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------- |
| T01 | Existing 1A tests pass               | Unchanged complete rules suite and doctest.                                 | Focused rules check and maintained workspace/native CI gates.                                                                                                                                                  | Passed  |
| T02 | AI unit tests pass                   | `crates/battle/tests/content_ai.rs`.                                        | Focused battle and workspace tests.                                                                                                                                                                            | Passed  |
| T03 | Complete AI tests pass               | Seeded bounded headless AI-versus-AI matches.                               | `content_ai::complete_ai_matches_are_legal_deterministic_and_terminate_across_seeds`: 32 seeded matches.                                                                                                       | Passed  |
| T04 | Bevy tests pass                      | Actual production client systems in headless App.                           | Published CI passed 41 default/45 test-support; corrected-source local production harness passed 43/47 with strict native/WASM lints. Next published CI counts pending.                                        | Passed  |
| T05 | Persistence tests pass               | Session exact restore plus injected/platform storage cases.                 | Session/native/client persistence tests and 20 real-WASM storage supplement cases passed.                                                                                                                      | Passed  |
| T06 | Desktop browser input passes         | Actual generated WASM with pointer/keyboard gameplay.                       | Both maintained desktop viewport full matches and real background/reload/resume scenarios passed at a6ed; independent manual desktop lifecycle completed.                                                      | Passed  |
| T07 | Mobile touch passes                  | Actual generated WASM with touch/swipe/orientation gameplay.                | Three complete touch matches/focused cases passed; formerly failing minimum rotated-Mulligan inspection now passes both maintained routes after production alignment correction.                               | Passed  |
| T08 | Refresh/resume passes                | Browser slot survives refresh; real Resume control.                         | Production and supplementary phone continuation passed; full Chromium actual pending-AI background/reload passed desktop720 (29.3 s), desktop800 (32.7 s), portrait (32.0 s). Minimum/DPR2 background not run. | Passed  |
| T09 | Native build passes                  | Pinned native application and libraries.                                    | Linux/macOS/Windows maintained CI matrix; local result separately recorded.                                                                                                                                    | Passed  |
| T10 | WASM build passes                    | Production/test-support application, battle, and rules target gates.        | `just check-web` and CI WASM gate.                                                                                                                                                                             | Passed  |
| T11 | Strict linting passes                | Workspace all/pedantic/nursery and rustdoc warnings denied.                 | `just check`, explicit supported WASM/feature gates.                                                                                                                                                           | Passed  |
| T12 | Formatting passes                    | Existing rustfmt and pinned Prettier settings.                              | `just fmt-check`; CI formatting.                                                                                                                                                                               | Passed  |
| T13 | Native render passes where supported | Real Bevy screenshot completion with bounded failure.                       | Published source rendered four scenes successfully. Corrected-source native Mulligan wrapping/vertical alignment changed; next CI capture/check and own inspection are Pending.                                | Pending |
| T14 | Browser pixels inspected             | Actual captures of battlefield/cards/panels/targets/log/results.            | Prior native/phone frames were reviewed; corrected rotated browser PNG is being reviewed and newly changed native Mulligan must receive its own final capture inspection.                                      | Pending |
| T15 | Full CI passes                       | All native, quality, WASM, browser, and native-render jobs at PR head.      | Verified-source non-browser gates Passed; full Chromium ended with 33 passed, one failed, and 21 not run at rotated Mulligan. Alignment correction passed focused checks; next-head full 55 Pending.           | Pending |
| T16 | Complete documentation               | Specification, architecture/development/testing, instructions, this record. | Formatting/structure gate plus complete 76-row evidence/outcome review.                                                                                                                                        | Pending |

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

Verified source `3cf7d0f050f7966c7ee5edd5d3750a8d47a09f92` has six green non-browser gates in
[PR run 38367364733](https://github.com/matanlurey/low-kingdoms/actions/runs/38367364733) and
[push run 38367360367](https://github.com/matanlurey/low-kingdoms/actions/runs/38367360367).
Client tests executed 41 default/45 test-support cases. Linux/macOS executed 103 workspace tests
plus one doctest; Windows executed 101 plus one doctest because two Unix process cases are scoped
out. The maintained full Chromium browser run at this checkpoint completed in 18.3 minutes with
33 passed, one failed, and 21 not run. The failure was the same minimum-phone rotated Mulligan
at case 34. Both desktop complete matches and the portrait complete match passed. Actual
background/reload/exactly-once cases passed at desktop 720 (29.3 seconds), desktop 800
(32.7 seconds), and portrait (32.0 seconds); minimum and DPR 2 background cases did not execute.
The published checkpoint full-CI result is Failed. The subsequent alignment correction passes
focused checks; next-head complete 55 and final native inspection remain Pending. The supplementary 50-case run independently exposed the same defect
after ten passing cases; no different blocker was found.

Local native checks remain blocked by host prerequisites. Full official Chromium launch is blocked
locally by ProcessSingleton UNIX-socket `EPERM`. Supplementary official headless-shell runs
establish actual WASM input/rendering but cannot establish real background-tab focus. The three
phone full matches all passed, as did three focused resource-reversal/confirmation/deployment/quick
Pass flows. The maintained suite retains all assertions and full Chromium mobile background cases.

| Check                                                       | Actual result                                                                                         | Evidence or limitation                                                                                                                                                        |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rules tests                                                 | Passed: 28 tests and one doctest                                                                      | Complete combat/fatigue/initiative matches unchanged.                                                                                                                         |
| Battle content/AI/session tests                             | Passed: 24                                                                                            | Includes 32 full AI seeds and 16 saved continuation seeds.                                                                                                                    |
| Client tests                                                | Published CI Passed 41 default/45 support; corrected-source local harness Passed 43/47                | Two actual Bevy/Taffy geometry regressions added; final published CI counts pending.                                                                                          |
| Native workspace tests                                      | Passed: Linux/macOS 103 + one doctest; Windows 101 + one doctest                                      | Two Unix-only process cases explain the platform difference.                                                                                                                  |
| Formatting/structure/strict Clippy/rustdoc                  | Passed in both verified-source CI runs                                                                | Final documentation/driver-only edits need another final formatting/CI check.                                                                                                 |
| `just check-web`                                            | Passed locally and in verified-source CI                                                              | Production/test-support compile and strict Clippy.                                                                                                                            |
| `just build-web`                                            | Passed                                                                                                | Earlier measured production WASM 62,316,103 bytes is not a final-size claim.                                                                                                  |
| `just check`                                                | Blocked locally; both CI quality gates passed                                                         | Missing local native host prerequisites.                                                                                                                                      |
| `just smoke-native`                                         | Blocked locally; published CI Passed; corrected-source capture Pending                                | New Mulligan wrapping/vertical alignment requires fresh native evidence and inspection.                                                                                       |
| `just smoke-web` / maintained full 55                       | Blocked locally; final CI Pending                                                                     | Full Chromium socket policy blocks local launch; supplementary subsets do not pass this aggregate gate.                                                                       |
| Supplementary storage suite                                 | Passed: 20 across five viewports                                                                      | `evidence/storage-browser-debug/report.json`; no failures/skips.                                                                                                              |
| Supplementary three phone matches                           | Passed: 3                                                                                             | `evidence/web-shell-phone-matches/report.json`; portrait 171.7 s, minimum 138.2 s, DPR 2 landscape 482.5 s.                                                                   |
| Supplementary non-background 50-case subset                 | Historical failure after ten passes; identified rotated Mulligan correction now Passed focused routes | Corrected-source full subset/maintained full 55 still need their complete conclusions.                                                                                        |
| Focused three phone input cases                             | Passed: 3                                                                                             | `evidence/web-phone-focus-final/report.json`; edge reversal/middle card, confirm, deployment, quick Pass.                                                                     |
| `just check-all`                                            | Blocked locally                                                                                       | Required native/full-channel capabilities unavailable; not a successful skip.                                                                                                 |
| Native CI Linux/macOS/Windows, quality, WASM, native-render | Passed in PR and push runs                                                                            | Exact verified-source run links above.                                                                                                                                        |
| Maintained full Chromium at verified checkpoint             | Failed: 33 passed, one minimum rotated-Mulligan failure, 21 not run; 18.3 minutes                     | Artifact 11798108044; both desktop/portrait complete matches and three actual background cases passed. Alignment correction passed focused checks; next-head full 55 Pending. |
| Independent desktop playable review                         | Passed                                                                                                | Manual complete match, Play Again, Main Menu/Resume, inspected captures.                                                                                                      |
| Phone actual touch matches/render review                    | Passed in supplementary shell                                                                         | All three reached round 6/revision 64 Defeat, restarted fresh, returned to Menu; no browser errors.                                                                           |

## Visual evidence and manual review

Latest [native artifact 11797915158](https://github.com/matanlurey/low-kingdoms/actions/runs/38367364733/artifacts/11797915158)
belongs to PR run 38367364733 and verified source
`3cf7d0f050f7966c7ee5edd5d3750a8d47a09f92`. Its ZIP SHA-256 is
`45aae6465a71d1117c4503956cf72e5e4f3f686b0f30bbb8e86cb82aaa79eccb`.
The root checked all four PNGs against the reviewed scroll-fix native frames: they are byte-identical.
Environment: Ubuntu 24.04, Openbox/Xvfb, Mesa 25.2.8, llvmpipe LLVM 20.1.2 Vulkan CPU,
1280×720/DPR 1. Title, Mulligan, deployed Battle, and Results are readable with no obvious clipping.
Local evidence includes `evidence/ci/native-scroll-fix/{title,mulligan,battle,results}.png`.
No interactive macOS/Windows or physical phone runtime is claimed.

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

### Completed supplementary phone touch matches

All three real-touch complete-match scenarios passed in the official Chromium 153.0.8010.12
headless shell with ANGLE/SwiftShader on Ubuntu 24.04.3. Portrait 390×844 took 171.7 seconds,
minimum 375×667 took 138.2 seconds, and landscape 844×390/DPR 2 took 482.5 seconds. Each reached
round 6/revision 64 Defeat, started a fresh match, and returned to Main Menu with Resume available.
No browser/page/console errors were recorded. Actual controls executed resources, deployment,
unit/stronghold attacks, Pass/Claim, inspection/log/speeds, rotation/DPR changes, refresh/resume,
and results/restart. Portrait/minimum offscreen target swipes retained the chosen attacker.

Evidence is under `evidence/web-shell-phone-matches/results/`, in each
`battle-complete-battle-thr-d8d9d-controls-resume-and-restart-PROJECT/` directory. The root reviewed
portrait/minimum crowded targeting, damage, and results, plus full DPR 2 combat, results, returned
menu, and rotated frame. Focused DPR 2 hand/footer/regroup captures were also inspected. Relevant
captures include `offscreen-attack-target.png`, `unit-combat.png`, `results.png`,
`returned-title.png`, `battle-resized.png`, and `battle-changed-dpi.png` where produced.

The separate focused phone run passed edge reversal, middle-resource-card inspection, resource
confirmation, deployment, and quick Pass on all three phone configurations; its report is
`evidence/web-phone-focus-final/report.json`. This repeats the formerly failing scroll path after
production clamping and verifies that reversal moves visible content immediately.

These establish real touch gameplay and reviewed pixels. The maintained full-channel suite still
must pass all 55 cases and actual mobile background-tab checks; shell runs do not prove tab focus.
Root/subpath and production-without-bridge remain in that maintained suite. No physical phone or
Safari/iOS certification is implied.

## Known limitations and exclusions

Intentional exclusions: card artwork/animation/audio, abilities/keywords, commanders, action/upgrade
cards, factions/affinities, advanced AI/difficulties, deck building/collections, campaigns, networking,
accounts, cloud/database saves, PWA/service worker, and additional gameplay systems. Six vanilla
units and fixed decks are prototype fixtures. The AI is deterministic and basic rather than
competitive. Saves require exact content/profile/version compatibility and have no migration.
Chromium touch emulation does not certify Safari/iOS hardware or physical phone performance.

Remaining verification: corrected-source complete 55-case/full-CI conclusion, two real mobile
background-tab cases, and fresh native pixel capture/inspection. The rotated first/last card and
fitting/overflow geometry corrections pass focused checks; no known gameplay/input blocker remains.
Local host/socket restrictions remain Blocked, not successful skips. Do not use unchanged older
native frames to approve the newly changed Mulligan vertical/wrapping alignment.

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
mocked. The two desktop and portrait background cases passed in full Chromium at the inspected
checkpoint. Minimum and DPR 2 background cases did not run after the minimum rotated-Mulligan
failure. All five must pass
in the complete maintained matrix. Headless-shell supplementary cases cannot establish that behavior.

### Portrait resource-scroll correction

The inspected a6ed browser run passed both desktop complete matches, background pause/restore,
and desktop storage/input cases before failing the 390×844 portrait full-match scenario while
revealing a resource card. Bevy clamps its computed scroll offset for rendering, but the client's raw
`ScrollPosition` could accumulate beyond the actual range. Reversing direction then consumed the
invisible excess rather than immediately moving the visible cards. This is a production input defect,
not a rules-engine rejection or a permitted test skip.

The verified presentation fix bounds raw scroll positions to the actual content/viewport range
and includes passing reversal/overflow regressions. The driver also uses bounded directional search and acknowledges
setup commands using the human view's submitted decision/phase, rather than a revision that an
independent AI decision might also advance. These changes preserve real input and existing gameplay
assertions. Focused checks and all three supplementary phone full matches now pass on corrected source.
The complete maintained browser run is still required; 24 historical passes do not establish the
30 cases that did not execute in that old run.

### Narrow DPR 2 complete-match deadline

Only the `phone-landscape-dpr2` complete-match scenario now has a 900-second deadline; the other
complete matches retain 600 seconds, ordinary cases 120 seconds, and the aggregate suite 3600
seconds. The local DPR 2 complete match measured 482.5 seconds (about 480 seconds). Desktop
software-rendered CI/local timings of 6.1/4.3 minutes give a ratio that projects approximately
681 seconds for that DPR 2 case. This narrowly scoped bounded allowance preserves all gameplay,
input, rendering, error, and resume assertions; it does not retry or drop a test. Final maintained
CI execution remains the completion requirement.

### Verified full Chromium checkpoint outcome

[Browser artifact 11798108044](https://github.com/matanlurey/low-kingdoms/actions/runs/38367364733/artifacts/11798108044)
from PR run 38367364733 records the actual full Chromium attempt on source
`3cf7d0f050f7966c7ee5edd5d3750a8d47a09f92`. Archive SHA-256:
`f4c3094b7a2bc8dd6f210aa028dc9e6023feceb82d631d3bbb34174f67a84935`.
The environment worker checked both report and logs: 33 passed, one failed, 21 not run in
18.3 minutes. The failure is the already identified first opening card outside scroll reach at
667×375, not a new blocker. Desktop 720/800 and portrait complete matches passed. Real background
pause/reload/exactly-once passed in those three viewports at 29.3/32.7/32.0 seconds respectively.
The minimum-phone and DPR 2 background cases did not run. The alignment correction passed focused checks; the final
next-head complete run remains Pending; this historical failed checkpoint is not a full pass.

### Rotated minimum-phone Mulligan alignment

After the three completed touch matches passed, the supplementary 50-case run passed ten cases
and then failed the minimum-phone Mulligan rotated to 667×375. The pane starts at x=8 with width
651 and raw offset zero, but centering overflowing content puts its first card at x=-70. Backward
scroll cannot reveal content preceding the start offset. The root inspected the actual failed PNG.
This is a production alignment defect, separate from the already corrected excess-scroll problem.

Presentation corrected the overflowing row alignment. Actual first/last opening-card inspection
now passes on both maintained root and subpath routes at 667×375 in 18.3/18.6 seconds, preserving
the full human view and own pending decision. Two actual Bevy/Taffy geometry regressions verify
reachable overflow origins on both axes and fitting-row centering at DPR 1/2. The corrected-source
production harness passes 43 default/47 test-support and strict native/WASM lint gates. The root is
reviewing the corrected browser PNG; next CI must capture and inspect the changed native Mulligan
wrapping/vertical alignment separately. Earlier complete phone matches remain factual passes.

### Alignment correction verification

The production alignment fix is stable. Maintained root/subpath lifecycle routes inspected the
first and last opening cards after minimum-phone rotation, taking 18.3 and 18.6 seconds with no
change to the human engine view or submitted-decision state. Geometry tests run actual Bevy/Taffy
layout at DPR 1 and 2, covering both overflow axes and centering when cards fit. Local corrected-source
checks passed 43 default/47 test-support client tests and strict native/WASM lints. No assertion was
removed. The final new-head 55-case conclusion and changed native Mulligan pixels remain Pending.
