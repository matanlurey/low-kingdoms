# Scaffold acceptance

Implementation and verification are in progress. This checklist records observable acceptance,
not planned gameplay. Update it from actual command output and inspected evidence before handoff.

## Decisions

- Bevy is exact `0.19.1`; release/source identity is verified at
  `b56fc29d3016e641754765244b5ba3f9cc504671`.
- Rust is exact `1.98.0`, edition 2024, resolver 3. Tool availability/build checks remain separate.
- Three real packages separate executable, client, and host tooling; no domain package is created.
- UI-oriented Bevy features, WebGL2 static output, default embedded font, no external art/font.
- Explicit production and `test-support` configurations; read-only browser observations only.
- Exact auxiliary pins are in `tools/tool-versions.toml`; WASM CLI follows the lockfile crate.
- Generated evidence belongs under ignored `evidence/native/` and `evidence/web/`.

## Acceptance checklist

- [ ] Workspace metadata/lint inheritance, unpublished packages, and dependency boundaries verified.
- [ ] Pinned tools install/doctor report checked; Cargo/npm lockfiles committed.
- [ ] Native and WASM feature trees inspected for supported configurations.
- [ ] Title/About screens have only functional scaffold controls and no gameplay placeholders.
- [ ] Headless initialization/navigation/repeated cleanup/input tests pass without query panics.
- [ ] Formatting, strict host Clippy, tests/doctests, and strict rustdoc pass.
- [ ] Separate production and test-support WASM compile/lint checks pass.
- [ ] Native renderer launches, screenshot completes, image inspected, normal exit checked.
- [ ] Interactive native About/Back, resize, minimize/restore, window close, and Quit exercised.
- [ ] Actual WASM bundle renders at root and `/low-kingdoms/`; captures inspected.
- [ ] Browser pointer, touch, keyboard traversal/activation, and Escape exercised.
- [ ] Viewports 1280×720, 1280×800, 390×844, and 844×390 checked.
- [ ] Post-startup resize and high-DPI scenario checked without clipping/overflow/input drift.
- [ ] Production browser smoke passes and test bridge is absent.
- [ ] Native/browser failures cannot silently pass as skipped renderer checks.
- [ ] Linux/macOS/Windows CI compile/test jobs and rendering jobs configured with verified action pins.
- [ ] Instructions/skill metadata/document paths refer to implemented commands and pass checks.
- [ ] Final diff reviewed for unused/speculative code, name consistency, and scope.

## Check evidence

No successful build or rendered-runtime result is claimed by the initial plan. Replace pending
rows below with the exact checks run and their outcomes, including failures/blockers.

| Check                                   | Environment                                     | Outcome | Evidence                                                                        |
| --------------------------------------- | ----------------------------------------------- | ------- | ------------------------------------------------------------------------------- |
| Bevy release/package identity           | Official release and published package metadata | Passed  | Versioned source map.                                                           |
| Documentation and local skills          | Linux, Prettier 3.9.9                           | Passed  | 15 documents formatted; local links and three skill names/descriptions checked. |
| Toolchain/setup/doctor                  | Pending                                         | Not run | Pending.                                                                        |
| Structure/format/host lint/tests/docs   | Pending                                         | Not run | Pending.                                                                        |
| WASM compile/lints and feature tree     | Pending                                         | Not run | Pending.                                                                        |
| Native renderer capture/inspection      | Pending                                         | Not run | `evidence/native/` when collected.                                              |
| Interactive native window checks        | Pending                                         | Not run | Record actual OS/display.                                                       |
| Chromium bundle/input/viewports/subpath | Pending                                         | Not run | `evidence/web/` when collected.                                                 |
| Production no-hooks visual smoke        | Pending                                         | Not run | Production capture and bridge absence.                                          |
| macOS/Windows execution                 | Pending CI/desktop                              | Not run | Do not infer runtime support from configuration.                                |

## Measurements and blockers

Record production WASM/JS byte sizes and actual build times after packaging; no load-time or binary-
size promise is made before measurement. Record browser/GPU/display facts from runtime logs.

No blocker has been recorded yet. An unavailable renderer, display, browser, system dependency,
permission, or tool must be recorded with its exact diagnostic and affected check. Keep the
acceptance requirement in place and finish independent checks. CI execution must be reported
separately from adding CI configuration.
