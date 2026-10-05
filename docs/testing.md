# Testing and rendering evidence

Three layers establish different facts. Compilation checks types and target compatibility.
Headless Bevy checks behavior and ownership. Actual renderer captures establish displayed pixels
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

This mode runs four production cases instead of the local root/subpath support matrix. It requires
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

`just check-web` separately checks/lints application packages for `wasm32-unknown-unknown`, including
the supported production and non-default `test-support` configurations. It does not target-build
host-only `xtask`. No universal `--all-features` gate is used.

Client tests compose the actual production navigation/screen plugins with minimum required state,
input, focus, and asset support. A test window/resource is only ECS setup and cannot provide
renderer evidence. Tests update through deferred operations and real state transitions.

The behavior checks cover one title root and expected text/controls, About/Back through the
production activation path, repeated transition cleanup, keyboard focus/activation, browser
configuration omitting Quit, and system initialization without conflicting-query panics.

## Native smoke

`just smoke-native` runs the real native application with its windowing/rendering path. Screenshot
support is behind `test-support`; its launch option is `--screenshot PATH`. A completed Bevy capture
is saved before normal exit. The application capture deadline is **60 seconds**. The process
wrapper also defaults to 60 seconds; `--timeout` changes that outer deadline, not the application
capture ceiling. Unsuccessful capture or an unavailable required adapter is a nonzero failure.

Default evidence is `evidence/native/title.png` and `evidence/native/run.log`. Inspect the image for
the title, About/Quit controls, readability, clipped text, and a nonblank rendered screen. A delay
or startup log alone is insufficient.

The configured Linux CI virtual-display/software-GPU path runs functional rendering checks;
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
retained traces. The orchestration timeout defaults to **900 seconds** for the full viewport matrix
on software renderers; individual tests and startup also have bounded deadlines. Required resource
failures, unhandled errors, readiness timeout, or a missing browser fail rather than skip.

Readiness comes from the live application. Nonzero canvas dimensions, successful resources, bridge
observations, and screenshots complement one another. Resolving a WASM import or finding a DOM
page title/loading message is not evidence that Bevy title text rendered.

The test-support bundle exposes a frozen, read-only `window.__LOW_KINGDOMS_TEST__` snapshot of live
screen/focus/viewport/control layout. The bridge has no navigation or arbitrary mutation methods
and does not maintain a parallel model. Real pointer, touch, and keyboard events activate controls.
The production bundle is smoke-tested again without test support, and its bridge must be absent.
The normal `low-kingdoms-ready` event is separate and can remain in production for the loader.

| Scenario                | Logical viewport                | Required observations                                                 |
| ----------------------- | ------------------------------- | --------------------------------------------------------------------- |
| Desktop                 | 1280 × 720                      | Title/About/Back, focus, keyboard and pointer activation.             |
| Handheld-shaped desktop | 1280 × 800                      | Readable layout and controls inside the viewport.                     |
| Phone portrait          | 390 × 844                       | Touch activation, wrapping, no scrolling/clipping.                    |
| Phone landscape         | 844 × 390                       | Controls fit the short viewport, touch activation.                    |
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

Use `docs/plans/scaffold.md` for the acceptance checklist and check-outcome record. Distinguish
passed, failed, blocked, and not run, then distinguish built, headless-tested, and visually inspected
claims. A missing display, dependency, permission, browser, or adapter blocks that check; keep the
check and complete independent work. `just check-all` cannot silently downgrade mandatory rendering.
