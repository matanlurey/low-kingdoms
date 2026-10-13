# Rendered references

These screenshots were manually reviewed on 2026-10-05 after the actual Bevy renderer checks passed. They show readable text, unclipped controls, responsive layout, hover feedback, and keyboard focus. The UI uses Bevy 0.19.1 and the bundled `FiraMono-subset.ttf` font; the application version is 0.1.0.

The web captures come from the generated WASM bundles used by `cargo xtask smoke-web --timeout 900`: all 12 cases passed in 465.6 seconds, with zero skipped, unexpected, or flaky cases. Root and subpath checks exercised mouse/touch, Tab, Shift+Tab, Enter, Space, Escape, About/Back, resize, and a live DPR 2-to-1 change. Production startup checks verified rendered pixels, CSS/backing-buffer dimensions, successful resource loads, and absence of the test bridge. Production and `test-support` configurations also passed `cargo xtask check-web`, including strict Clippy.

Web environment: Linux 6.18.44 x86_64, Node 24.19.0, Playwright 1.63.0, Chromium 153.0.8010.12, WebGL2 through ANGLE Vulkan 1.3.0 with SwiftShader Device (Subzero). Phone cases use Chromium mobile/touch emulation. Screenshots use CSS-pixel dimensions; the 844×390 DPR 2 canvas has a 1688×780 backing buffer. Web screenshots were inspected by the web implementation agent, with additional review by the coordinating agent.

Native environment: Linux 6.18.44 x86_64 with glibc 2.39, Rust 1.98.0, Xvfb 1280×720×24 at `DISPLAY=127.0.0.1:109`, Vulkan software rendering through llvmpipe LLVM 20.1.2 and Mesa 25.2.8-0ubuntu0.24.04.2. The coordinating and native-tooling agents inspected the native title image. The coordinating agent also inspected About and the minimum-size title after real X11 keyboard/resize events. About/Back, focus, resize, and native Quit passed; minimize/restore requires a window manager and was not run. The native title includes About and Quit; the web title includes About.

| Screenshot                                                                       | Logical canvas | DPR | Captured state                         |
| -------------------------------------------------------------------------------- | -------------- | --- | -------------------------------------- |
| [native-title-1280x720.png](native-title-1280x720.png)                           | 1280×720       | 1   | Native title                           |
| [native-about-1280x720.png](native-about-1280x720.png)                           | 1280×720       | 1   | Native About after keyboard activation |
| [native-title-320x360.png](native-title-320x360.png)                             | 320×360        | 1   | Native title at minimum window size    |
| [web-title-1280x720.png](web-title-1280x720.png)                                 | 1280×720       | 1   | Title                                  |
| [web-title-hovered-1280x720.png](web-title-hovered-1280x720.png)                 | 1280×720       | 1   | About hovered                          |
| [web-title-focused-1280x720.png](web-title-focused-1280x720.png)                 | 1280×720       | 1   | About keyboard focus                   |
| [web-about-1280x720.png](web-about-1280x720.png)                                 | 1280×720       | 1   | About                                  |
| [web-title-1280x800.png](web-title-1280x800.png)                                 | 1280×800       | 1   | Title                                  |
| [web-title-390x844.png](web-title-390x844.png)                                   | 390×844        | 1   | Portrait title                         |
| [web-about-390x844.png](web-about-390x844.png)                                   | 390×844        | 1   | Portrait About                         |
| [web-title-844x390-dpr2.png](web-title-844x390-dpr2.png)                         | 844×390        | 2   | Landscape title                        |
| [web-about-844x390-dpr2.png](web-about-844x390-dpr2.png)                         | 844×390        | 2   | Landscape About                        |
| [web-title-resized-900x550.png](web-title-resized-900x550.png)                   | 900×550        | 1   | Title after desktop resize             |
| [web-title-after-dpr-change-390x844.png](web-title-after-dpr-change-390x844.png) | 390×844        | 1   | Title after orientation and DPR change |
| [web-production-title-1280x720.png](web-production-title-1280x720.png)           | 1280×720       | 1   | Production title                       |
| [web-production-title-844x390-dpr2.png](web-production-title-844x390-dpr2.png)   | 844×390        | 2   | Production landscape title             |

Regenerate renderer evidence with `just smoke-native` and `just smoke-web`, inspect the images and environment logs under `evidence/`, and explicitly select replacement references after the checks pass. These files are a reviewed visual record. Verification uses live geometry, real input, and captured-pixel assertions; replacing a reference requires manual review.

## Milestone 1B battle references

These battle captures were manually inspected on 2026-10-13. They supplement the original scaffold
references above. They establish readable pixels and the particular interactions described here;
the [Milestone 1B acceptance record](../../plans/milestone-1b-acceptance.md) separately records the
complete browser matrix and all verification outcomes.

The four native captures come from the successful native-render job in
[Actions run 38360813050](https://github.com/matanlurey/low-kingdoms/actions/runs/38360813050),
code checkpoint `a6eddfc5d6193895e09dd0e8fd1485982564b8f7`. Environment: Ubuntu 24.04,
Xvfb/Openbox at 1280×720/DPR 1, Mesa 25.2.8, and llvmpipe LLVM 20.1.2 Vulkan CPU rendering.
All four were inspected by the coordinating worker; text, cards, counters, and controls were
readable without obvious clipping.

The web captures come from real mouse/touch interaction with the generated WASM client using
official Chromium 153.0.8010.12 headless shell, Playwright 1.63.0, and ANGLE/SwiftShader on
Ubuntu 24.04.3. Screenshots are in CSS pixels, at DPR 1. These particular landscape frames used
a live orientation/viewport change; the maintained suite separately verifies landscape DPR 2.
The minimum-size Hand tray leaves Pass and the other action controls reachable. Resources expose
the human's card identities and both players' public counts. The portrait Resume frame restored
round 2, revision 10, and the exact pending human regroup decision after a real refresh.
The coordinating worker inspected every selected frame. Mobile frames use Chromium touch
emulation and do not establish physical-device or Safari behavior.

| Screenshot                                                                             | Logical canvas | Captured state                                |
| -------------------------------------------------------------------------------------- | -------------- | --------------------------------------------- |
| [milestone-1b-native-title-1280x720.png](milestone-1b-native-title-1280x720.png)       | 1280×720       | Native Title                                  |
| [milestone-1b-native-mulligan-1280x720.png](milestone-1b-native-mulligan-1280x720.png) | 1280×720       | Six-card opening hand and mulligan controls   |
| [milestone-1b-native-battle-1280x720.png](milestone-1b-native-battle-1280x720.png)     | 1280×720       | Battlefield, hand, counters, actions, and log |
| [milestone-1b-native-results-1280x720.png](milestone-1b-native-results-1280x720.png)   | 1280×720       | Terminal result and restart/menu controls     |
| [milestone-1b-web-hand-375x667.png](milestone-1b-web-hand-375x667.png)                 | 375×667        | Compact Hand tray with persistent footer      |
| [milestone-1b-web-preview-375x667.png](milestone-1b-web-preview-375x667.png)           | 375×667        | Card inspection and contextual Play           |
| [milestone-1b-web-deployment-375x667.png](milestone-1b-web-deployment-375x667.png)     | 375×667        | Deployed exhausted unit                       |
| [milestone-1b-web-resources-844x390.png](milestone-1b-web-resources-844x390.png)       | 844×390        | Resource inspection panel                     |
| [milestone-1b-web-log-844x390.png](milestone-1b-web-log-844x390.png)                   | 844×390        | Chronological event log panel                 |
| [milestone-1b-web-resume-390x844.png](milestone-1b-web-resume-390x844.png)             | 390×844        | Restored pending regroup after refresh        |
