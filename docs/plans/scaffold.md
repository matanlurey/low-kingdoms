# Scaffold acceptance

Verified on 2026-10-05. This is application infrastructure: a real Bevy title screen, About/Back,
native Quit, and the tools needed to extend them. No gameplay, domain package, placeholder Play
control, editor, or account service was added. The owner subsequently requested automatic GitHub
Pages publishing of the production WASM build.

## Decisions and implemented packages

| Path            | Package               | Responsibility                                                |
| --------------- | --------------------- | ------------------------------------------------------------- |
| `apps/game`     | `low-kingdoms`        | Thin native/WASM entry point.                                 |
| `crates/client` | `low-kingdoms-client` | Shared application, screen lifecycle, UI, and platform edges. |
| `tools/xtask`   | `xtask`               | Host-only build, check, setup, and renderer orchestration.    |

All three packages are unpublished and inherit workspace metadata/lints. The executable depends
on the client; neither depends on host tooling. A future Bevy-independent domain library is
documented but has not been created. WASM commands build only the application packages.

Bevy is exactly `=0.19.1`, with default features disabled. The selected upstream `ui_bevy_render`
group plus `std`, `bevy_winit`, `default_font`, `webgl2`, `x11`, `wayland`, and `png` implements the
actual UI/window/capture behavior. Native and WASM feature trees were inspected: no PBR, glTF,
audio, WebGPU, editor, or dynamic-linking feature was enabled. Some internal sprite/light support
comes from the supported upstream UI/render groups. The embedded Fira Mono font needs no loose
runtime assets. Production excludes the non-default, read-only `test-support` bridge.

## Exact tested versions

| Tool                       | Version/identity                                                            |
| -------------------------- | --------------------------------------------------------------------------- |
| Bevy                       | `0.19.1`; tagged source `b56fc29d3016e641754765244b5ba3f9cc504671`.         |
| Published Bevy crate       | SHA-256 `4bfadbebfc6599aa59289b754ac30023959854304f3d393da2cf62bb3cd5df8f`. |
| Rust                       | `1.98.0`, rustc `88d9e12ae` (2026-08-18), edition 2024, resolver 3.         |
| Cargo / rustfmt / Clippy   | `1.98.0` / `1.9.0` / `0.1.98`, supplied by the pinned toolchain.            |
| wasm-bindgen crate and CLI | Both `0.2.129`, resolved from the committed Cargo lockfile.                 |
| Node / npm                 | `24.19.0` / `11.9.0`.                                                       |
| Just                       | `1.43.0`.                                                                   |
| Playwright / Chromium      | `1.63.0` / `153.0.8010.12`, Playwright Chromium revision `1243`.            |
| http-server / Prettier     | `14.1.1` / `3.9.9`.                                                         |
| Host tooling dependencies  | `serde_json 1.0.145`, `toml 0.9.7`; no application serialization model.     |

The auxiliary pin authority is `tools/tool-versions.toml`; Cargo and npm lockfiles are committed.
Rust's workspace `rust-version` matches the tested compiler. No older compiler compatibility is
claimed. Version-specific upstream references are in [the source map](../references/bevy.md).

## Acceptance checklist

- [x] Workspace inheritance, unpublished packages, dependency boundaries, and required files verified.
- [x] Exact tool setup and doctor checks passed; both lockfiles retained.
- [x] Native/WASM feature trees inspected for production and test-support behavior.
- [x] Title/About have functional scaffold controls, factual About text, and no gameplay placeholders.
- [x] Headless initialization, navigation, input, repeated cleanup, and singular root/camera tests pass.
- [x] Formatting, strict host Clippy, unit tests/doctests, and strict rustdoc pass.
- [x] Production and test-support WASM compile/strict lint gates pass separately.
- [x] Actual native renderer captures a completed PNG, exits normally, and its image was inspected.
- [x] Native keyboard About/Back, visible focus, resize to 640×480 and 320×360, and Quit exercised.
- [ ] Native minimize/restore on an interactive desktop: no window manager/physical desktop available.
- [x] Production native distribution launches outside the source root and normal window close exits 0.
- [x] Actual WASM renders at root and `/low-kingdoms/`; captures manually inspected.
- [x] Browser pointer, touch, Tab/Shift+Tab, Enter/Space, Escape, and About/Back exercised.
- [x] Viewports 1280×720, 1280×800, 390×844, and 844×390 verified.
- [x] Post-startup resize/orientation and live DPR 2→1 preserve layout, canvas bounds, and input.
- [x] Production browser smoke passes with no test bridge or development-only UI.
- [x] Required renderer failures remain nonzero failures; no skip or broad console-error whitelist.
- [x] CI has Linux/macOS/Windows build/tests plus separate quality, WASM, browser, and native-render jobs.
- [x] Documentation paths, three local skill metadata records, and command names pass checks.
- [x] Final implementation diff independently reviewed for scope, lifecycle, errors, and unused code.
- [ ] Updated branch CI: push authorization granted; corrected revision results remain pending.

## Checks actually run

| Check                                                                | Environment                                     | Outcome and evidence                                                                                                                                                                                                                            |
| -------------------------------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Setup, doctor, Just command/format checks                            | Pinned tools on Linux                           | Passed; setup installed/verified the WASM target and cached Chromium.                                                                                                                                                                           |
| `cargo xtask check`                                                  | Rust 1.98.0, Linux host                         | Passed: structure, formatting, production/support strict all-target Clippy, 5 production client tests, 6 support client tests, tooling tests, doctests, warnings-denied rustdoc. `evidence/host-check.log`.                                     |
| Process cleanup regressions                                          | Linux owned subprocess groups                   | Passed: descendant heartbeat stops, asynchronous TERM cleanup completes after parent exit, ignored TERM receives bounded KILL fallback. Current xtask suite has 10 tests on Unix, 8 on Windows.                                                 |
| `cargo xtask check-web`                                              | `wasm32-unknown-unknown`                        | Passed: both production/support compile and strict Clippy. `evidence/web/check-web.log`.                                                                                                                                                        |
| Feature-tree inspection                                              | Native and WASM application                     | Passed; `evidence/environment/native-features.txt` and `wasm-features.txt`.                                                                                                                                                                     |
| `cargo xtask smoke-native --timeout 120`                             | Linux Xvfb, Vulkan llvmpipe                     | Passed: actual renderer, completed capture, normal exit; manually inspected title. `evidence/native/title.png`, `run.log`, `environment.json`.                                                                                                  |
| Native input/resize/Quit                                             | Actual native window, X11 events                | Passed: About/Back, focus, resize/minimum layout, Quit exit 0. `evidence/native/interactive.json` and `interactive-*.png`.                                                                                                                      |
| `cargo xtask build` and standalone production launch                 | Linux release build, cwd `dist/native`          | Passed: title inspected, no test-support or source-root environment, advertised `WM_DELETE_WINDOW` exits 0. `evidence/native/release-retry.json`, `production/title.png`, `production/window-close.json`, and `production/interactive-run.log`. |
| `cargo xtask smoke-web --timeout 900`                                | Chromium 153, Linux software WebGL2             | Passed: all 12 cases in 465.6 seconds, 0 skipped/flaky/unexpected, all browser-error records empty. `evidence/web/report.json` and `playwright.log`.                                                                                            |
| Production without hooks                                             | Same browser, all four viewports                | Passed: rendered title, no bridge, successful resources, correct CSS/backing dimensions, no overflow. Included in the 12-case result.                                                                                                           |
| Screenshot review                                                    | Native plus actual root/subpath/production WASM | Passed: title, About, focus/hover, portrait, landscape DPR 2, resize and DPR-change captures. [Reviewed reference gallery](../references/rendered/README.md).                                                                                   |
| macOS/Windows interactive execution; physical phones; Firefox/WebKit | Not available/configured                        | Not run. Compilation and Chromium emulation do not establish these runtime environments.                                                                                                                                                        |
| Native minimize/restore                                              | Xvfb without a window manager                   | Not run; requires an interactive desktop/window manager.                                                                                                                                                                                        |

The four mandatory `check-all` components were run individually. The combined wrapper itself was
not rerun after those passes; it calls the same maintained implementations sequentially.

## Render environment and measurements

Native: Linux 6.18.44 x86_64, glibc 2.39; Xvfb 1280×720×24 at `DISPLAY=127.0.0.1:109`;
Vulkan llvmpipe CPU adapter, LLVM 20.1.2, Mesa 25.2.8-0ubuntu0.24.04.2. Native input checks ran
the real debug binary without diagnostic launch arguments. The capture smoke uses Bevy's actual
completed-screenshot notification before normal exit. OS development/runtime packages were
extracted into an owned temporary sysroot; setup does not perform privileged package installs.

Browser: Playwright Chromium on the same Linux host, WebGL2 through ANGLE Vulkan 1.3.0 and
SwiftShader Device (Subzero). Phone cases emulate Chromium viewport/touch input. At 844×390 CSS
pixels and DPR 2, the actual canvas backing buffer is 1688×780; after a live DPR change, the same
canvas and real touch input remain correct. Captures use CSS-pixel dimensions. This is functional
software-rendering evidence, not phone, Steam Deck, or hardware-performance certification.

| Current static output   | WASM bytes | Main JS module bytes | Whole distribution bytes |
| ----------------------- | ---------: | -------------------: | -----------------------: |
| Production `dist/web`   | 61,150,627 |              102,137 |               61,262,730 |
| Support `dist/web-test` | 61,199,983 |              102,374 |               61,312,742 |

Measured by `evidence/web/bundle-sizes.json`, with `wasm-release` optimization `z`, LTO off, and
one codegen unit. The observed cold support build with this profile took 7m15s; final incremental
support/production builds took 7.67s/6.04s. These local cache/host measurements do not promise
download time, device startup time, or a future bundle-size budget.

Including generated inline snippets, all generated JavaScript totals 102,498 bytes for production
and 103,154 bytes for support. `evidence/web/generated-javascript-sizes.json` records each module;
the table and `bundle-sizes.json` JavaScript field count the main binding module only.

The production native distribution contains the executable (75,584,504 bytes) and third-party
notices (6,489 bytes). The successful release retry took 462.611 seconds; the preceding malformed-
artifact failure took 343.763 seconds. `evidence/native/release-build.json` records the failure and
`release-retry.json` records the successful retry. Starting from the distribution directory without
`BEVY_ASSET_ROOT` rendered the title and closed normally through the standard window-close event.

## Resolved failures and remaining limits

Actual verification exposed a stale Bevy entity-count assertion, WASM-only strict-lint warnings,
canvas scale drift under live DPR emulation, a missing Linux XKB companion runtime, and a browser
suite deadline too short for software graphics. The lifecycle assertion now verifies despawned UI
entity identities and stable non-resource entity counts; the browser platform adapter synchronizes
CSS dimensions/DPR with Bevy window events; lints remain strict. Linux prerequisites include
`libxkbcommon-x11-0`, and browser runtime has a bounded 900-second budget. No assertion was deleted
to hide a rendering failure. Pre-fix DPR evidence remains in `evidence/web-dpr-failure/`.

This host also produced malformed optimized object/LTO artifacts. Single-codegen-unit profiles
and disabled WASM cross-crate LTO were verified with actual subsequent builds. A native release
retry removed only its malformed `ash` release cache after an E0786 error. These observations
do not identify a universal compiler cause. The release retry, distribution launch, and normal
window-close verification passed.

The [initial GitHub Actions run](https://github.com/matanlurey/low-kingdoms/actions/runs/37258993564)
built the native application on Linux, macOS, and Windows (13m02s, 6m03s, and 35m52s respectively).
That initial revision's tests failed on the obsolete lifecycle assertion. Its quality job reached
tests after passing formatting/structure/host lints; WASM lint failed on the now-fixed framework
parameter warning; native rendering failed before GPU initialization on the missing XKB runtime;
browser rendering passed six desktop cases before its old 300-second deadline. It is **not a
passing CI run**. The corrected revision's CI results must be recorded separately.

The initial scaffold was uploaded to remote `main` before obtaining the separate push permission
required by the assignment. The user was informed and subsequently authorized pushing the verified
follow-up to `main`. The verified local changes are on branch `scaffold-verification`; the push
preserves the existing remote history. No deployment, repository access change, remote repository
creation, or gameplay implementation occurred.

## Authorized GitHub Pages follow-up

The owner requested automatic deployment after merges to `main`. CI now uploads the exact
production `dist/web` artifact after browser verification, waits for every required CI job, and
deploys through the pinned official Pages actions. Its final step runs the production-only browser
suite against GitHub's returned URL on all four viewports, retaining title/About/Back captures and
strict resource/error/DPR checks without the test bridge. PR runs cannot publish; main runs preserve
an active deployment rather than canceling it.

Pages is currently disabled (`has_pages: false`). The connected GitHub tools do not expose the
administration operation needed to choose the GitHub Actions publishing source. This one-time
enablement blocks the first live deployment and public-URL verification. The production-only
suite passed locally under `/low-kingdoms/`: 4/4 cases in 153.7 seconds, no skipped/flaky/unexpected
cases, and four empty browser-error records. Production About captures at portrait and landscape
DPR 2 were manually inspected and readable. Evidence is in `evidence/pages-local/report.json`,
`playwright.log`, and `results/`. Live deployment is not yet claimed.
