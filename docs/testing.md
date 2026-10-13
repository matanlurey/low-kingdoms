# Testing and rendering evidence

Three layers establish different facts. Compilation checks types and target compatibility.
Headless rules/battle and Bevy tests check behavior and ownership. Actual renderer captures establish displayed pixels
only after inspection. A green result in one layer does not replace another.

## Published GitHub Pages checks

The Pages deployment job runs the production-only browser suite against GitHub's actual returned
URL after all CI gates pass. It does not load a test-support bundle. To repeat the same check:

```sh
LOW_KINGDOMS_DEPLOYED_URL=https://matanlurey.github.io/low-kingdoms/ \
LOW_KINGDOMS_EVIDENCE_DIR=evidence/pages \
node tools/web-tests/node_modules/@playwright/test/cli.js test \
  --config tools/web-tests/playwright.config.mjs
```

This mode runs production cases across the configured viewport matrix instead of the local
root/subpath support matrix. It requires
the real WASM readiness signal, successful resources, nonblank canvas captures, matching CSS/DPR
dimensions, no overflow/errors/bridge, and visible pixel changes from keyboard About/Back input.
Inspect the title/About/returned-title images. The command does not deploy; the CI job publishes
the already verified `dist/web` artifact before checking the URL. Publication failures and public
browser failures remain distinct from successful local rendering checks.

## Non-rendering checks

`just check` runs formatting, repository structure validation, strict host Clippy, unit/integration
tests and doctests, and rustdoc with warnings denied. Structural validation reads Cargo metadata
and checks package boundaries, inherited lints/unpublished metadata, required document paths, and
skill YAML front matter. It is a maintenance check, not a second architecture framework.

`just check-web` separately checks/lints application, battle, and rules packages for
`wasm32-unknown-unknown`, including
the supported production and non-default `test-support` configurations. It does not target-build
host-only `xtask`. No universal `--all-features` gate is used.

Client tests compose the actual production gameplay/navigation/screen plugins with minimum required state,
input, focus, and asset support. A test window/resource is only ECS setup and cannot provide
renderer evidence. Tests update through deferred operations and real state transitions.

The original behavior checks cover one title root and expected text/controls, About/Back through the
production activation path, repeated transition cleanup, keyboard focus/activation, browser
configuration omitting Quit, and system initialization without conflicting-query panics. Battle
checks exercise the same production command, AI scheduling, reconciliation, and screen lifecycle
paths; they must not replace those paths with a test-only gameplay implementation.

## Headless rules acceptance

Run `cargo test --locked -p low-kingdoms-rules`; these tests require no display, GPU, browser, or
Bevy. `tests/headless.rs` owns small anonymous fixtures and imports focused acceptance cases.
The suite uses only public commands/views and privileged snapshot JSON for invariants/corruption.

- `cases/matches.rs`: complete combat, deck-exhaustion, and initiative/passing games.
- `cases/rules.rs`: configuration, seeded setup, full-hand mulligans, buffered selections,
  zero/one/two initial resources, payment/readiness, duplicate units, unlimited battlefield,
  simultaneous combat/retaliation, persistent damage, fatigue/draws, and arithmetic boundaries.
- `cases/privacy.rs`: safe views, event redaction, private pending decisions, error privacy,
  and complete legal-command discovery validated against actual application.
- `cases/snapshots.rs`: pending setup/regroup decisions, claims, damaged units and terminal
  round trips; identical continuation/events; replay across seeds/overrides; incompatible catalogs,
  versions, malformed state, and round-counter overflow rejection.
- `cases/invariants.rs`: 32 deterministic legal command walks check every step for card conservation,
  globally unique/stable IDs, no lethal live units, eligible actors, exact restore/continuation,
  and terminal immutability. This adds no property-testing framework to the rules crate.
- Private RNG unit tests pin the SplitMix64 reference stream and shuffle permutation.

Strict all-targets Clippy and warnings-denied rustdoc cover the public API. `just check-web` also
checks/lints the rules crate for WASM. Compilation establishes WASM compatibility; native tests
establish executed headless behavior, without claiming a WASM runtime or physical device test.
See `docs/plans/milestone-1a-acceptance.md` for criterion mapping and actual verification evidence.

## Prototype content, AI, and session tests

Run `cargo test --locked -p low-kingdoms-battle` without a display, browser, or Bevy. The suite uses
small vanilla fixtures rather than a designed content library.

- `tests/content_ai.rs`: RON parsing/validation, exact compatibility identity, setup choices,
  affordable deployments, favorable trades, stronghold attacks, claim/pass decisions, explicit
  tie-breaking under reordered legal lists, boundary values, legal selection, and full AI-versus-AI
  simulations across fixed seeds.
- `tests/session_save.rs`: human-only views/events, command atomicity including log/revision,
  pending setup/AI turns, deterministic continuation to complete outcomes, compatibility versions,
  role/profile/content rejection, corrupt snapshots/logs/metadata/history, and bounded visible
  history with full authoritative replay.
- `src/platform/storage/native.rs` client tests: absolute per-user paths, missing slots,
  create/replace/read/clear, opaque corrupt text, failed replacement preserving old contents,
  temporary collision cleanup, and operation errors. Client storage injection also exercises
  unavailable/failed writes and completion cleanup through the production lifecycle.

AI tests receive only `PlayerView`, legal commands, and public definitions. Headless full matches
must reach a terminal outcome with a deterministic action bound. They do not substitute for a human
playing the real interface. Timing tests use controlled elapsed time instead of blocking sleeps.
Keep the existing complete combat/fatigue/initiative rules matches passing unchanged.

## Playable client acceptance

`cargo test --locked -p low-kingdoms-client` must exercise production plugins and commands for
initialization, mulligan/resources, deploy, AI eligibility/scheduling, attack/damage/readiness,
regroup, completion, restore, restart, and screen cleanup. Legal-action controls must track the
engine oracle; cancellation must leave the session unchanged. Tests compare fresh permitted views
and rendered entity data after transitions, including destroyed-unit removal and preserved scroll.

Persistence tests inject storage rather than touching a player's real slot. Cover initial and
replacement saves, confirmation/cancellation, corrupt/incompatible input, read/write/quota failures,
finished cleanup, active pending AI after restore, and retry without duplicate accepted commands.
Background timing tests pause while inactive and verify no accumulated catch-up on return.

The concrete client acceptance modules are `src/gameplay/tests.rs` and
`src/gameplay/presentation/tests.rs`, alongside existing `src/tests.rs` navigation coverage.
Production-controller tests include full matches/restarts, reverse initial-resource selection and
buffered AI draft preservation, startup Resume reconciliation, safe failed-save retries, exact
pending-AI refresh, replacement confirmation/corrupt saves, read failure recovery, and terminal
cleanup fallback. Presentation tests pin damage/readiness text, stable entities/scroll after combat,
actual legal targets, resize/orientation ownership, missing-entity reconciliation, and log scroll.

The browser battle suite uses actual compiled WASM with real mouse/touch input. Required scenarios
start from Title, keep/mulligan, confirm resources, deploy, observe the opponent, regroup, attack
both a unit and stronghold, claim initiative, inspect a card/resources, toggle log/hand panels and
speed, refresh/resume, finish, Play Again, and return to Main Menu. Test configurations may choose
a controlled seed internally; browser globals remain read-only and never mutate authoritative data.

At least one desktop and one touch-enabled mobile session must reach results through the real UI.
Touch review must scroll to an offscreen target while retaining the chosen attacker, test an
orientation change, then refresh and resume. Assert legal target highlighting, disabled invalid
controls, persistent turn/Pass indicators, and one root/camera after repeated lifecycle transitions.
Root/subpath hosting and production-without-bridge tests remain mandatory.

Capture and inspect the actual battlefield, opening/hand cards, remaining Health/damage/readiness,
strongholds, resource indicators/inspection, targeting, overflow, log, card preview, and results.
Record the exact browser/OS/viewport/DPR and evidence paths in the 1B acceptance record. A captured
PNG or passing bridge assertion alone does not establish readable pixels or usable controls.

## Native smoke

`just smoke-native` runs the real native application with its windowing/rendering path. Screenshot
support is behind `test-support`; its launch options are `--screenshot PATH [--scene title|mulligan|battle|results]`. A completed Bevy capture
is saved before normal exit. The application capture deadline is **60 seconds per scene**. The process
wrapper also defaults to 60 seconds per scene; `--timeout` changes that outer deadline, not the application
capture ceiling. Unsuccessful capture or an unavailable required adapter is a nonzero failure.

Default evidence is `evidence/native/{title,mulligan,battle,results}.png` with matching scene logs. The test-support fixture submits production controls and legal commands, isolates its save slots, and freezes AI scheduling only after reaching each capture scene. Inspect all four images for readable cards, strongholds, controls, clipped text, and a nonblank rendered screen. A delay
or startup log alone is insufficient.

The configured Linux CI virtual-display/software-GPU path runs Openbox inside Xvfb so the native window receives real focus and the inactive-window AI policy remains exercised. This path runs functional rendering checks;
executed evidence is recorded separately from the configuration. It is not a performance
measurement or a physical device certification. On an interactive desktop, also
exercise About/Back, resize, minimize/restore, normal window close, and native Quit; record the OS
that actually ran these checks. Compilation on macOS/Windows does not claim these interactions ran.
Native resize checks include the supported 320×360 logical-pixel minimum; the browser does not
inherit that desktop window constraint.

## Browser smoke

`just smoke-web` builds/serves actual generated bundles and runs pinned Chromium/Playwright tests.
Playwright 1.63.0 selects Chromium **153.0.8010.12**, browser revision **1243**, for this baseline.
Installed browser availability does not by itself establish successful WASM rendering.
Default evidence is `evidence/web/`, including server logs, captures, browser diagnostics, and
retained traces. The orchestration timeout defaults to **3600 seconds** for all 55 cases in the
viewport matrix on software renderers; ordinary tests have 120-second deadlines and complete-battle
scenarios have a 600-second deadline, with bounded startup/expect waits. An observed desktop
match took 270 seconds with software rendering, so the expanded matrix needs this bounded budget.
Required resource
failures, unhandled errors, readiness timeout, or a missing browser fail rather than skip.

Readiness comes from the live application. Nonzero canvas dimensions, successful resources, bridge
observations, and screenshots complement one another. Resolving a WASM import or finding a DOM
page title/loading message is not evidence that Bevy title text rendered.

The test-support bundle exposes a frozen, read-only `window.__LOW_KINGDOMS_TEST__` snapshot of live
screen/focus/viewport/control layout plus human-visible gameplay/selection/legal-action data.
It excludes seeds, snapshots, command logs, deck order, and opposing private cards.
The bridge has no navigation or arbitrary mutation methods
and does not maintain a parallel model. Real pointer, touch, and keyboard events activate controls.
The production bundle is smoke-tested again without test support, and its bridge must be absent.
The normal `low-kingdoms-ready` event is separate and can remain in production for the loader.

| Scenario                | Logical viewport                | Required observations                                                 |
| ----------------------- | ------------------------------- | --------------------------------------------------------------------- |
| Desktop                 | 1280 × 720                      | Title/About/Back, focus, keyboard and pointer activation.             |
| Handheld-shaped desktop | 1280 × 800                      | Readable layout and controls inside the viewport.                     |
| Phone portrait          | 390 × 844                       | Touch gameplay, hand/preview/log panels, readable layout.             |
| Minimum phone           | 375 × 667                       | Every gameplay action remains reachable; Pass stays visible.          |
| Phone landscape         | 844 × 390                       | Short-layout controls, touch gameplay, orientation changes.           |
| High DPI                | At least one scenario above     | Logical layout and input remain correct with increased scale factor.  |
| Resize                  | Change dimensions after startup | Live viewport/control layout updates without overflow or input drift. |
| Root and subpath        | `/` and `/low-kingdoms/`        | Actual static generated resources resolve in both locations.          |

Tab and Shift+Tab traverse focus; Enter and Space activate; Escape from About returns to title and
Escape at title keeps the application open. Hover/pressed/focus have visible feedback. The minimum
control height is 44 logical pixels, a project touch-target budget rather than accessibility
certification. CSS must not disable page zoom to conceal layout issues.

Chromium viewport/device-scale emulation is the automated baseline. It does not establish Safari,
iOS hardware, Steam Deck, or physical phone performance. Add browser/device claims only after
actual execution and evidence.

## Screenshot review and failures

Inspect initial reference images before approving any baseline. Normal verification must not
automatically accept changed images. Keep updates separate and review text, layout, focus, and
clipping in each affected capture. Do not mask most of the screen or use broad tolerances to hide
regressions. Exact pixel references require a consistent browser/OS/font/DPI environment and need
not match across graphics backends.

Selected, reviewed captures are maintained under `docs/references/rendered/`. Its README records
the source environment and inspection context. These manual references supplement ignored run
screenshots/logs/traces under `evidence/`; normal checks neither replace them nor automatically
accept a new reference. Compare structure and readability across backends without requiring exact
cross-backend pixel equality.

Do not suppress application errors or remove failing assertions to make a suite pass. A known
non-application graphics/browser warning may have narrow documented handling; unexpected errors
remain failures. Store generated evidence under ignored directories, with exact environment facts.

Use `docs/plans/milestone-1b-acceptance.md` for current battle criteria and check outcomes.
The scaffold and 1A records retain their earlier milestone evidence. Distinguish
passed, failed, blocked, and not run, then distinguish built, headless-tested, and visually inspected
claims. A missing display, dependency, permission, browser, or adapter blocks that check; keep the
check and complete independent work. `just check-all` cannot silently downgrade mandatory rendering.
