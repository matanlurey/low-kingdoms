# Milestone 1B implementation acceptance

The authoritative approved specification is [milestone-1b.md](milestone-1b.md), containing all
33 confirmed decisions from the supplied worker instructions. The specification and implementation
belong to the same feature branch and one implementation PR; no separate planning PR is required.
The original [1A gameplay specification](milestone-1a.md) remains unchanged.

## Status and evidence policy

Milestone status: **complete at the VERIFIED IMPLEMENTATION CHECKPOINT
`6e537c3d16671c43edc0d3ad0df275d66da880a4`**.
Corrected-budget PUSH run 38381702292 and PR run 38381707332 both passed all seven required gates;
Pages deployment was correctly skipped. Both actual full Chromium reports passed 55/55 with
55 attempts, zero skipped/unexpected/flaky cases, and all 55 diagnostics empty. Every one of the
76 criteria has executed evidence, including genuine background/reload checks and reviewed pixels.
Local maintained native/full-Chromium limits remain Blocked, without replacing supported CI results.
This concrete implementation checkpoint remains valid when a records-only commit receives another
SHA; final documentation-head checks are reported separately in the PR. No Milestone 1B feature
is deferred and no later gameplay system was introduced.

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
| P17 | Normal/Fast/Instant          | 750/150/0 ms presentation scheduling, default Normal.            | Production controlled-time 750/150/0 ms tests, rendered speed controls, and all five actual full-Chromium background/reload cases passed.                         | Passed |
| P18 | Usable 375×667               | Compact layout keeps all game actions reachable.                 | Complete 375×667 portrait match and focused controls passed; the separate 667×375 rotated Mulligan defect is recorded under P03/P04/P06.                          | Passed |

### Persistence

| ID  | Requirement             | Implementation evidence                                                         | Verification evidence                                                                                                                                                | Status |
| --- | ----------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| S01 | Automatic local saves   | Accepted transition persists current session through client store.              | Client injected-store save/retry tests; browser saved progression.                                                                                                   | Passed |
| S02 | Browser saves           | `platform/storage/browser.rs`; `low-kingdoms.active.v1` localStorage.           | Maintained full Chromium storage and refresh/resume passed all five viewports; supplementary storage and exact manual regroup restoration also passed.               | Passed |
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

| ID  | Requirement                          | Implementation evidence                                                     | Verification evidence                                                                                                                                                                       | Status |
| --- | ------------------------------------ | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| T01 | Existing 1A tests pass               | Unchanged complete rules suite and doctest.                                 | Focused rules check and maintained workspace/native CI gates.                                                                                                                               | Passed |
| T02 | AI unit tests pass                   | `crates/battle/tests/content_ai.rs`.                                        | Focused battle and workspace tests.                                                                                                                                                         | Passed |
| T03 | Complete AI tests pass               | Seeded bounded headless AI-versus-AI matches.                               | `content_ai::complete_ai_matches_are_legal_deterministic_and_terminate_across_seeds`: 32 seeded matches.                                                                                    | Passed |
| T04 | Bevy tests pass                      | Actual production client systems in headless App.                           | Current published CI executed 43 default/47 test-support client cases with actual native feature configurations.                                                                            | Passed |
| T05 | Persistence tests pass               | Session exact restore plus injected/platform storage cases.                 | Session/native/client persistence tests and 20 real-WASM storage supplement cases passed.                                                                                                   | Passed |
| T06 | Desktop browser input passes         | Actual generated WASM with pointer/keyboard gameplay.                       | Both maintained desktop viewport full matches and real background/reload/resume scenarios passed at a6ed; independent manual desktop lifecycle completed.                                   | Passed |
| T07 | Mobile touch passes                  | Actual generated WASM with touch/swipe/orientation gameplay.                | Three complete touch matches/focused cases passed; formerly failing minimum rotated-Mulligan inspection now passes both maintained routes after production alignment correction.            | Passed |
| T08 | Refresh/resume passes                | Browser slot survives refresh; real Resume control.                         | Corrected-budget full Chromium actual background/reload/exactly-once passed all five viewports: 29.6/33.4/31.8/27.4/75.7 s. Native/replay and human refresh cases passed.                   | Passed |
| T09 | Native build passes                  | Pinned native application and libraries.                                    | Linux/macOS/Windows maintained CI matrix; local result separately recorded.                                                                                                                 | Passed |
| T10 | WASM build passes                    | Production/test-support application, battle, and rules target gates.        | `just check-web` and CI WASM gate.                                                                                                                                                          | Passed |
| T11 | Strict linting passes                | Workspace all/pedantic/nursery and rustdoc warnings denied.                 | `just check`, explicit supported WASM/feature gates.                                                                                                                                        | Passed |
| T12 | Formatting passes                    | Existing rustfmt and pinned Prettier settings.                              | `just fmt-check`; CI formatting.                                                                                                                                                            | Passed |
| T13 | Native render passes where supported | Real Bevy screenshot completion with bounded failure.                       | Current 712 native-render CI passed four scenes; root reviewed Title, corrected top-aligned Mulligan, Battlefield, and Defeat.                                                              | Passed |
| T14 | Browser pixels inspected             | Actual captures of battlefield/cards/panels/targets/log/results.            | Root personally reviewed all four current native frames and fresh desktop/phone complete-match targeting, damage, results, rotated/DPR/hand/footer frames.                                  | Passed |
| T15 | Full CI passes                       | All native, quality, WASM, browser, and native-render jobs at PR head.      | VERIFIED IMPLEMENTATION CHECKPOINT 6e: PUSH38381702292 and PR38381707332 both Passed all seven gates and actual full Chromium55/55; Pages deployment correctly skipped.                     | Passed |
| T16 | Complete documentation               | Specification, architecture/development/testing, instructions, this record. | Approved specification, architecture/development/testing/instructions, all76criterion records, and reviewed visual references complete; pinned formatting and root structure checks Passed. | Passed |

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

VERIFIED IMPLEMENTATION CHECKPOINT `6e537c3d16671c43edc0d3ad0df275d66da880a4` passed all seven
required gates in [PUSH run 38381702292](https://github.com/matanlurey/low-kingdoms/actions/runs/38381702292).
[PR run 38381707332](https://github.com/matanlurey/low-kingdoms/actions/runs/38381707332) independently
passed all seven gates as well. Pages deployment was correctly skipped in both runs. Each full
browser report and empty diagnostics were verified separately; neither run is inferred from the other.

The corrected-budget browser report confirms expected 55, 55 attempts, skipped/unexpected/flaky 0,
and 36.49 minutes. All 55 diagnostics are empty: 20 battle, 15 screen, and 20 storage reports.
All five complete matches and genuine background/reload/exactly-once cases passed. Both push and
PR non-browser gates executed 43 default/47 test-support client tests, 105 workspace tests plus
one doctest on Linux/macOS, and 103 plus one doctest on Windows (two Unix process cases excluded).
Production/support WASM remained 62,375,816/62,503,117 bytes.

Local maintained native checks remain Blocked by missing host prerequisites, and local full
Chromium remains Blocked by ProcessSingleton UNIX-socket `EPERM`. They are not successful skips.
Provisioned CI executed actual native features and full Chromium. The separate shell 50/50
excluded five background cases; the verified full suite actually executed and passed those cases.
Both corrected-budget browser reports are now complete and independently verified.

| Check                                                                  | Actual result                                                                  | Evidence or limitation                                                                                   |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- |
| Rules tests                                                            | Passed: 28 tests and one doctest                                               | Complete combat/fatigue/initiative matches unchanged.                                                    |
| Battle content/AI/session tests                                        | Passed: 24                                                                     | Includes 32 complete AI seeds and 16 saved continuation seeds.                                           |
| Maintained client tests                                                | Passed: 43 default / 47 test-support in both non-browser CI runs               | Production plugins/native features; input, geometry, privacy, lifecycle, outcomes, and storage.          |
| Native workspace tests                                                 | Passed: Linux/macOS 105 + one doctest; Windows 103 + one doctest               | Two Unix-only subprocess cases explain the difference.                                                   |
| Formatting/structure/strict Clippy/rustdoc                             | Passed                                                                         | Corrected-budget non-browser CI plus focused ten xtask tests, strict lints/docs, syntax, and formatting. |
| `just check-web`                                                       | Passed locally and in both corrected-budget CI runs                            | Production/test-support compile and strict Clippy.                                                       |
| `just build-web`                                                       | Passed; production WASM 62,375,816 bytes / support 62,503,117 bytes            | Actual corrected-budget CI measurements; unchanged product bundle.                                       |
| `just check`                                                           | Blocked locally; supported CI quality Passed                                   | Local native host prerequisites unavailable.                                                             |
| `just smoke-native`                                                    | Blocked locally; supported corrected-budget CI Passed                          | PR artifact 11803726517 exactly matches all four published/root-reviewed references.                     |
| `just smoke-web` / full maintained Chromium                            | Blocked locally; corrected-budget PUSH CI Passed 55/55                         | 55 attempts, no skipped/unexpected/flaky cases, all diagnostics empty.                                   |
| Supplementary shell subset                                             | Passed: 50/50, all five complete matches                                       | Exactly five background cases excluded; independently superseded for that behavior by full Chromium.     |
| Focused phone input / rotated Mulligan                                 | Passed                                                                         | Reversal/middle card/confirm/deploy/quick Pass and root/subpath first/last inspection.                   |
| `just check-all`                                                       | Blocked locally                                                                | Mandatory individual gates Passed on provisioned push CI; no local aggregate pass claimed.               |
| Corrected-budget PUSH native matrix/quality/WASM/native-render/browser | Passed: all seven gates                                                        | Concrete implementation checkpoint/run above.                                                            |
| Corrected-budget PR native matrix/quality/WASM/native-render/browser   | Passed: all seven gates                                                        | Actual PR result/report verified separately from push; Pages deployment correctly skipped.               |
| Corrected-budget PR browser                                            | Passed: 55/55, 55 attempts, zero skipped/unexpected/flaky cases, 47.12 minutes | All 55 diagnostic attachments empty: 20 battle, 15 screen, 20 storage.                                   |
| Independent desktop/mobile playable and pixel review                   | Passed                                                                         | Complete real input, all five full matches/background cases, reviewed native/browser frames.             |

| Corrected-budget full Chromium project | Complete-match seconds | Genuine background/reload seconds |
| -------------------------------------- | ---------------------- | --------------------------------- |
| Desktop 1280×720                       | 196.4                  | 29.6                              |
| Desktop 1280×800                       | 214.1                  | 33.4                              |
| Portrait 390×844                       | 195.7                  | 31.8                              |
| Minimum 375×667                        | 169.0                  | 27.4                              |
| Landscape 844×390 / DPR 2              | 558.4                  | 75.7                              |

| Corrected-budget PR full Chromium project | Complete-match seconds | Genuine background/reload seconds |
| ----------------------------------------- | ---------------------- | --------------------------------- |
| Desktop 1280×720                          | 255.5                  | 37.0                              |
| Desktop 1280×800                          | 286.1                  | 41.3                              |
| Portrait 390×844                          | 270.2                  | 41.3                              |
| Minimum 375×667                           | 219.6                  | 34.7                              |
| Landscape 844×390 / DPR 2                 | 719.1                  | 90.5                              |

These are software-renderer check timings, not physical-device performance guarantees.

## Visual evidence and manual review

Corrected-budget [browser artifact 11807050366](https://github.com/matanlurey/low-kingdoms/actions/runs/38381702292/artifacts/11807050366)
is 27,837,625 bytes; SHA-256
`cb509c8e0885d699a49ba65be6948145a681b3b3390032e1307e0022c36899a6`.
Its actual report confirms all 55 corrected-budget cases and empty diagnostics. Its separate
[PR browser artifact 11806453449](https://github.com/matanlurey/low-kingdoms/actions/runs/38381707332/artifacts/11806453449)
is 28,163,700 bytes; SHA-256
`bcc8aee719114ddc178c765ec774e0d786bd4a528e0634c7bd2a018f47f326aa`.
That safely verified report passed 55/55 with 55 attempts, zero skipped/unexpected/flaky cases,
47.12 minutes, and all 55 diagnostics empty. Both runs establish actual full-channel gameplay
and tab-background behavior independently. Corrected-budget
[PR native artifact 11803726517](https://github.com/matanlurey/low-kingdoms/actions/runs/38381707332/artifacts/11803726517)
exactly matches all four published/manually reviewed references; root verified their hashes against
real tree `420babee`. The budget-only correction changes no product code or rendered layout.
Earlier directly reviewed frames below retain concrete provenance; new corrected-budget results
are the current verified implementation checkpoint, not a claim about a future records-only head.

Current [native artifact 11800026645](https://github.com/matanlurey/low-kingdoms/actions/runs/38372317206/artifacts/11800026645)
belongs to PR run 38372317206 and source `71202a6bd32a7e726b38f4f0679554347c22bd75`.
ZIP SHA-256: `c0dc9e8bc8f9a5232d697b84f113c512dde2154ea14c50ef738636f4c64df231`.
The root personally inspected Title, corrected top-aligned Mulligan, Battlefield, and Defeat.
Text, cards, counters, and controls were readable with no obvious clipping. Logs were clean apart
from expected software-renderer/X warnings. Environment: Ubuntu 24.04, Openbox/Xvfb,
1280×720/DPR 1, Vulkan CPU llvmpipe LLVM 20.1.2, Mesa 25.2.8-0ubuntu0.24.04.4.
Local captures are under `evidence/ci/native-alignment/`; selected reviewed images and their
context are in [the rendered references](../references/rendered/README.md).
No interactive macOS/Windows or physical phone runtime is claimed.

### Verified full Chromium captures

[Push browser artifact 11801966935](https://github.com/matanlurey/low-kingdoms/actions/runs/38372312078/artifacts/11801966935)
is 27,785,934 bytes; SHA-256
`324e81730ed6366ef7fdcc0f2a75f336aec4c323c53a47e95b70760490983b13`.
It contains the independently verified 55/55 report and actual full-Chromium captures. The root
personally reviewed unit-combat frames at all five viewports, both root/subpath rotated minimum
Mulligan frames, minimum Results, and landscape DPR 2 Mulligan. Essential text and controls were
readable, scrolled row edges were intentional, and no visual blocker remained. All diagnostic
attachments were empty. Earlier native/manual/supplementary reviews below remain separate evidence.

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
the independent desktop lifecycle review; the separately verified maintained push matrix/full-CI conclusion appears above.
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

These establish real touch gameplay and reviewed pixels. The maintained full-channel push suite subsequently passed all 55 cases, including actual mobile
background-tab checks; shell runs alone do not prove tab focus.
Root/subpath and production-without-bridge remain in that maintained suite. No physical phone or
Safari/iOS certification is implied.

## Known limitations and exclusions

Intentional exclusions: card artwork/animation/audio, abilities/keywords, commanders, action/upgrade
cards, factions/affinities, advanced AI/difficulties, deck building/collections, campaigns, networking,
accounts, cloud/database saves, PWA/service worker, and additional gameplay systems. Six vanilla
units and fixed decks are prototype fixtures. The AI is deterministic and basic rather than
competitive. Saves require exact content/profile/version compatibility and have no migration.
Chromium touch emulation does not certify Safari/iOS hardware or physical phone performance.

All 76 requirements have executed evidence and both corrected-budget push/PR CI passed the
complete matrix. Milestone/record closure is complete at the verified implementation checkpoint.
No product rule or source correction was needed for the timing failure. Local host/socket limits
remain Blocked, and physical-phone/Safari/interactive macOS/Windows claims remain excluded.

## Corrective verification and limits

Early native CI exposed missing Xvfb window focus; Openbox now supplies real focus without bypassing
the production inactive-AI policy. Startup Resume reconciliation, unsupported font separators,
same-frame pointer/tray handling, explicit result labels, quota retry, and native invalid-UTF-8
recovery received production corrections and focused regressions. No Milestone 1A rule changed.

A historical full-Chromium attempt on source `3cf7d0f050f7966c7ee5edd5d3750a8d47a09f92` ended
33 passed/one failed/21 not run in 18.3 minutes, with the first rotated opening card unreachable at
667×375. Both desktop and portrait complete matches and their actual background/reload/exactly-once
cases passed (29.3/32.7/32.0 seconds). Minimum/DPR 2 background cases did not run. The subsequent
alignment correction is verified by actual Bevy/Taffy overflow/centering regressions at DPR 1/2,
root/subpath first/last-card input, the current 50/50 shell run, and its newly reviewed native frame.
Raw scroll positions also clamp before each input delta, so immediate reversal no longer consumes
invisible excess. Driver acknowledgements compare the human's own submitted decision/view.

The maintained browser uses supported full `channel: "chromium"`. Its background fixture disables
Playwright focus emulation via CDP, then real visibility/focus controls the production pause policy;
no document or game state is mocked. The supplementary shell excludes exactly five background
cases. All five executed and passed in the verified complete push matrix.

The earlier source-712 push passed with 900-second DPR 2 / 3600-second aggregate bounds.
The separate PR then exhausted the DPR 2 scenario budget after gameplay and a fresh restart had
already succeeded. The correction permits 1200 seconds only for `phone-landscape-dpr2` full matches
and 4200 seconds for the aggregate. Other complete matches remain 600 seconds, ordinary cases
120 seconds, expect waits 60 seconds, and CI 90 minutes. All 55 assertions/cases and zero retries
remain intact. Root reviewed the timing/comments-only three-file diff and actual failure PNG;
focused host/tooling/syntax/format checks passed. Corrected-budget push and PR full CI both passed independently.

## Corrected-budget verification

The historical source-712 PR ended 46 passed/one DPR 2 scenario timeout/eight not run after
successful round-six Results, a fresh restart, both accepted Keep commands, and visible
InitialResourcing/revision 2. Independent trace/driver review proved the last normal 60-second
poll was interrupted only 925.5 ms into its allowance by the overall scenario deadline. Root viewed
the correct restarted screen. The correction changed timing/comments only: DPR 2 scenario 1200
seconds and aggregate 4200 seconds, ordinary cases 120, expect waits 60, other matches 600,
CI 90 minutes, zero retries, and all 55 cases/assertions unchanged.

Focused tooling checks and all seven corrected-budget gates passed in both push and PR runs.
Both actual full Chromium suites passed 55/55, including all five genuine background/reload cases.
The 1200/4200-second bounds resolved the trace-proven scenario-budget failure without changing
product source, adding retries, removing cases/assertions, or weakening gameplay behavior checks.
T15/T16 and milestone closure are complete at the verified implementation checkpoint.
