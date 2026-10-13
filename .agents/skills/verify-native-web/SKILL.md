---
name: verify-native-web
description: Verify Low Kingdoms startup, build tooling, renderer features, assets, canvas sizing, and native/browser UI using actual renderers, real input, and reviewed evidence before declaring completion.
---

# Verify native and web

1. Read `docs/testing.md`, the approved milestone specification, and its acceptance record. Run `just doctor` and record exact versions,
   operating system, display/GPU context, and browser environment. Use `just setup` only when tools
   are missing; it must not silently install privileged system packages.
2. Run `just check` and `just check-web`. Build production with `just build` and `just build-web`.
   Keep production and non-default `test-support` configurations separate.
3. Run `just smoke-native`. It must launch the real window/renderer, await completed capture, and
   exit nonzero on timeout or missing required adapter. Inspect title, mulligan, battlefield, and results captures. On an available
   interactive desktop, exercise About/Back, resize, minimize/restore, close, and Quit separately.
4. Run `just smoke-web` against generated static bundles served over HTTP. Require application
   readiness, successful resources, nonzero canvas dimensions, and no unexpected browser errors.
   Exercise pointer, touch, Tab/Shift+Tab, Enter/Space, and Escape through browser input.
5. Check root and subpath hosting; desktop 1280×720; handheld-shaped 1280×800; phone portrait
   390×844 and the 375×667 minimum; phone landscape 844×390; post-startup resize; and at least one high-DPI scenario.
   Verify computed controls fit the viewport and the page has no canvas-created overflow.
6. Exercise a full battle through real input, including resource selections, deployment, attacks, Pass/Claim, inspection, speed, save/resume, results, and restart. Inspect captures for battlefield/hand/log/resource layouts, title/About, focus, contrast, and clipping. Read-only live-world
   bridge observations complement screenshots; they are not pixel evidence by themselves.
7. Run the production startup/visual smoke without hooks and assert the test bridge is absent.
   Keep screenshots, logs, traces, bundle-size measurements, and timings in ignored `evidence/`.
8. Update the acceptance plan with passed, failed, blocked, or not-run outcomes and precise evidence
   paths. Report build, headless, and inspected-render results separately.

Missing required GPUs or browsers never pass as skips. Never automatically bless snapshots,
delete failing assertions, mask most of the screen, or broadly whitelist console errors. Initial
reference images and later updates need explicit visual inspection in a consistent environment.
Chromium viewport emulation does not establish Safari/iOS or physical handheld support.
