# Versioned source map

Use **Bevy 0.19.1**. The official release and package VCS metadata identify source commit
`b56fc29d3016e641754765244b5ba3f9cc504671`. Its declared upstream minimum Rust version is 1.95.0;
this repository pins/tests its own compiler policy at 1.98.0 rather than claiming an untested MSRV.

| Question                           | Primary source                                                                                                                                                                                             |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Release identity and published API | [v0.19.1 release](https://github.com/bevyengine/bevy/releases/tag/v0.19.1), [0.19.1 API](https://docs.rs/bevy/0.19.1/bevy/)                                                                                |
| Feature groups and target support  | [Tagged Cargo.toml](https://github.com/bevyengine/bevy/blob/v0.19.1/Cargo.toml)                                                                                                                            |
| Composition and setup              | [Official setup](https://bevy.org/learn/quick-start/getting-started/setup/), [plugins](https://bevy.org/learn/quick-start/getting-started/plugins/)                                                        |
| UI components and interaction      | [Tagged button example](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/ui/widgets/button.rs)                                                                                                     |
| Tab focus navigation               | [Tagged tab-navigation example](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/ui/widgets/tab_navigation.rs)                                                                                     |
| UI layout, scaling, and text       | [Tagged UI examples](https://github.com/bevyengine/bevy/tree/v0.19.1/examples/ui), [scaling example](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/ui/ui_scaling.rs)                            |
| Application states                 | [Tagged state examples](https://github.com/bevyengine/bevy/tree/v0.19.1/examples/state)                                                                                                                    |
| Headless app/system testing        | [how_to_test_apps.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/tests/how_to_test_apps.rs), [how_to_test_systems.rs](https://github.com/bevyengine/bevy/blob/v0.19.1/tests/how_to_test_systems.rs)   |
| Screenshot lifecycle               | [Tagged screenshot example](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/window/screenshot.rs)                                                                                                 |
| WASM host and WebGL2 constraints   | [Tagged WASM directory](https://github.com/bevyengine/bevy/tree/v0.19.1/examples/wasm), [example index](https://github.com/bevyengine/bevy/blob/v0.19.1/examples/README.md)                                |
| Embedded default font              | [Tagged bevy_text source](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_text/src/lib.rs), [Fira Mono license](https://github.com/bevyengine/bevy/blob/v0.19.1/assets/fonts/FiraMono-LICENSE) |

The current Bevy website can describe a newer release over time. Verify APIs against the tagged
source or 0.19.1 API before using setup/migration guidance. Upstream sample gameplay is not part of
this scaffold.

Other workflow sources:

- [wasm-bindgen static deployment](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html):
  packaging/export approach; the actual CLI must match the lockfile.
- [Cargo workspace inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html) and
  [Clippy configuration](https://doc.rust-lang.org/stable/clippy/lint_configuration.html): manifest
  policy; validate against the pinned compiler.
- [Retroglyph STYLE_GUIDE](https://github.com/crates-lurey-io/retroglyph/blob/main/STYLE_GUIDE.md):
  inspected blob `44702a8ac3d5bd092be90203754fa17fa1f6877a`; application departures are in our guide.
- [Playwright screenshots](https://playwright.dev/docs/test-snapshots): environment-specific
  reference review, not permission to automatically accept failures.

Project command, layout, and lint decisions are Low Kingdoms policies. These links do not imply
that upstream mandates this repository structure. Do not vendor an entire source tree or maintain
a second Bevy manual here.
