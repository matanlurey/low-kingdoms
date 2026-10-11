# Development

Run project commands from the repository root. Rustup selects the exact repository toolchain;
Cargo commands used by verification and packaging use the committed lockfile.

## Versions and prerequisites

| Tool                      | Repository pin/source                                              |
| ------------------------- | ------------------------------------------------------------------ |
| Rust                      | `1.98.0`, `rust-toolchain.toml`; edition 2024, resolver 3.         |
| Bevy                      | Exactly `=0.19.1`, root `Cargo.toml`.                              |
| Node                      | `24.19.0`, `tools/tool-versions.toml`; development commands/tests. |
| npm                       | Compatible npm `11.9.0` used with the pinned Node distribution.    |
| Just                      | `1.43.0`, `tools/tool-versions.toml`.                              |
| Playwright                | `1.63.0`, tool pins and committed browser-test lockfile.           |
| Static server             | `http-server` `14.1.1`, tool pins and browser-test lockfile.       |
| Maintained text formatter | Prettier `3.9.9`, tool pins and browser-test lockfile.             |
| `wasm-bindgen-cli`        | Derived from the resolved `wasm-bindgen` entry in `Cargo.lock`.    |

`tools/tool-versions.toml` is the auxiliary CLI pin authority. Browser-test package metadata and
lockfile must agree with those pins. Rust's exact compiler policy is also reflected by the workspace
`rust-version`; this scaffold does not claim an older untested compiler.

Install rustup and the pinned Node distribution through their official installation instructions.
If Just is unavailable, bootstrap with `cargo xtask setup`; setup installs the pinned project-local
Just binary under `.tools/`. Otherwise use `just setup`.

```sh
cargo xtask doctor
cargo xtask setup
```

Setup installs the pinned toolchain components/WASM target, project-local Just and matched
`wasm-bindgen-cli`, locked npm development dependencies, and Playwright Chromium. It does not
install privileged operating-system packages, configure an editor or agents globally, or collect
credentials. After setup, use the project-local Just executable if Just is not on PATH.

Local binaries are `.tools/just-1.43.0/bin/just` and
`.tools/wasm-bindgen-0.2.129/bin/wasm-bindgen` (with `.exe` on Windows). The latter directory follows
the lockfile version when upgraded. Just recipes use the pinned Node interpreter and call Cargo
through `spawnSync` argument arrays; they do not interpolate user arguments into a platform shell.
Native development can use `cargo xtask dev` directly without Node. Formatting and browser commands
still require their Node-based development tools.

`just doctor` is read-only. It reports versions, installed targets, missing local tools, and known
platform prerequisites. A missing capability is reported with a setup/action instruction.

Linux requires a native display and the X11/Wayland development/runtime libraries required by
Winit, plus a Vulkan/OpenGL loader and usable adapter. On Debian/Ubuntu, the CI/native environment
typically needs `libx11-dev`, `libxcursor-dev`, `libxi-dev`, `libxrandr-dev`, `libxkbcommon-dev`,
`libxkbcommon-x11-0` (the X11 runtime companion),
`libwayland-dev`, `libvulkan1`, Mesa drivers, and `xvfb` for virtual-display smoke checks. Install
system packages through your OS package manager explicitly. Playwright may require additional OS
browser libraries; its diagnostic lists them. macOS requires the command-line developer tools;
Windows requires the Rust MSVC toolchain and corresponding Visual Studio build tools.

## Headless rules iteration

The authoritative rules are in `docs/plans/milestone-1a.md`. The engine is independent of Bevy;
its API and deterministic ordering are documented in `docs/architecture.md` and generated rustdoc.
Injected anonymous unit definitions suffice to execute complete matches. No client integration
or designed card library is included in 1A.

```sh
cargo test --locked -p low-kingdoms-rules
cargo clippy --locked -p low-kingdoms-rules --all-targets -- -D warnings
cargo check --locked -p low-kingdoms-rules --target wasm32-unknown-unknown
cargo tree --locked -p low-kingdoms-rules
just check
just check-web
```

The workspace gates include rules tests, Clippy, rustdoc, metadata, and crate-local instructions.
The WASM gate checks/lints the rules library alongside the client and executable while excluding
host tools. All existing CI jobs continue to use these commands; the native matrix executes the
headless rules suite on Linux, macOS, and Windows. Renderer smoke gates remain in place.

`Game::new(config, definitions)` returns a game and initialization events. Submit typed decisions
with `game.apply(player, &command)`; inspect `game.view_for(player)` and filtered
`events.for_player(player)`. Record `LoggedCommand` entries externally for `Game::replay`.
Never deliver authoritative snapshots to player-facing code. Snapshot JSON and injected catalogs
are sufficient for exact continuation; filesystem, database, and networking infrastructure are absent.

## Native and browser iteration

```sh
just dev
just dev-web
```

The native command builds/runs the actual window. The web command rebuilds the WASM distribution,
runs the matched `wasm-bindgen-cli` with `--target web`, and serves HTTP on port **4173** by default.
Open the printed URL. There is no watcher or hot reload; restart the command to rebuild.

The native window starts at 1280×720 logical pixels and has a 320×360 logical-pixel minimum resize
constraint. The browser canvas follows its available viewport without that native window constraint.

Static serving uses the pinned `http-server` API through `tools/web-tests/serve.mjs`, bound to
`127.0.0.1`. Direct loopback binding avoids the CLI's unnecessary network-interface enumeration
and keeps the server usable in restricted environments. Server-start errors, including an occupied
port, fail the command. This development wrapper is not shipped or needed by the static site.

The host tool accepts command overrides without shell interpolation:

```sh
cargo xtask dev-web --port 4200
cargo xtask build-web --output dist/custom-web
cargo xtask smoke-native --timeout 90
```

The shared syntax is `cargo xtask COMMAND [--port N] [--output PATH] [--timeout SECONDS]`.
Only commands for which an option is relevant use that option. Defaults and argument validation
live in the tool; subprocess failure propagates rather than producing a successful-looking report.
The same overrides work through Just, for example `just dev-web --port 4200`.
For `check-all --output PATH`, native and browser evidence go into `PATH/native/` and `PATH/web/`.

Use `cargo check --locked -p low-kingdoms-client` for a focused type check, then focused tests and
`just check`. Application changes affecting WASM also need `just check-web`. Presentation/startup
changes need the affected renderer smoke checks and screenshot inspection.

## Command map

| Command             | Behavior                                                             |
| ------------------- | -------------------------------------------------------------------- |
| `just doctor`       | Read-only tool/target/prerequisite report.                           |
| `just setup`        | Explicit pinned project-tool and browser dependency setup.           |
| `just dev`          | Build/run native from the repository root.                           |
| `just dev-web`      | Rebuild development WASM and serve HTTP, default port 4173.          |
| `just build`        | Package native release output in `dist/native/`.                     |
| `just build-web`    | Package production static output in `dist/web/`, without test hooks. |
| `just fmt`          | Format Rust and selected maintained text/config formats.             |
| `just fmt-check`    | Check formatting without changes.                                    |
| `just clippy`       | Strict workspace host lints, including tests and tooling.            |
| `just test`         | Host unit/integration tests and doctests; no GPU required.           |
| `just doc`          | First-party documentation with warnings denied.                      |
| `just check`        | Formatting, structural checks, host lints, tests, and docs.          |
| `just check-web`    | Compile/lint the application and rules packages for WASM.            |
| `just smoke-native` | Actual native renderer, completed screenshot, bounded failure.       |
| `just smoke-web`    | Generated bundles, real Chromium input, screenshots, and traces.     |
| `just check-all`    | All mandatory gates, including native/browser rendering.             |

## Files, assets, and rendering features

`dist/`, `target/`, `.tools/`, `node_modules/`, and `evidence/` are generated/ignored. Commit Cargo
and npm lockfiles. The shipped web output uses relative URLs and requires HTTP; opening files with
`file://` is unsupported. Test the same static distribution at `/` and `/low-kingdoms/`.

Keep selected, visually reviewed reference captures in `docs/references/rendered/`, with their
environment and review notes in its README. They supplement the ignored per-run `evidence/`
artifacts and are maintained manual references, not an automatic cross-backend pixel baseline.

Initial UI uses Bevy's embedded default font. No loose font/art files are needed.
[The asset policy](../assets/README.md) and [notices](../THIRD_PARTY_NOTICES.md) govern later additions.
Current distributions include the notices and no runtime asset directory; `assets/README.md` is
repository policy, not a shipped asset. Extend packaging when an implemented feature first needs
loose assets. Source paths must not depend on an absolute developer-machine directory.
WASM binding generation emits JavaScript/WASM/snippets and disables unused TypeScript declarations.

The selected Bevy group is `ui_bevy_render`, with explicit `std`, `bevy_winit`, `default_font`,
`webgl2`, `x11`, `wayland`, and `png`. It supplies the used UI/text/render/input/state capabilities
without the 3D, glTF, audio, or editor feature groups. `png` supports screenshot output.
WebGPU is not enabled: the pinned release describes it as experimental and it overrides WebGL2,
rather than providing fallback. No WASM threads or cross-origin isolation headers are required.

Development first-party code uses optimization level 1 and runtime dependencies use level 3.
Development, native release, and `wasm-release` use `codegen-units = 1`; development build scripts
and procedural macros explicitly use the same value in `[profile.dev.build-override]`. This reduces
parallel LLVM code generation to avoid malformed optimized objects observed on constrained build
hosts. The tradeoff is less parallel code generation, not a lower runtime optimization baseline.
Host-only `xtask` keeps its faster-build optimization override. Native release and size-oriented
`wasm-release` remain separate profiles, and Rust stays pinned to 1.98.0. Record measured output
rather than promising a bundle-size or load-time budget.

`wasm-release` uses `opt-level = "z"` with `lto = "off"`. Disabling cross-crate LTO limits cold
build/link cost and avoids the unstable path observed as zero-length thin-LTO intermediates on
constrained hosts. The observation does not establish a universal compiler cause or guarantee;
keep build outcomes and bundle-size measurements separate from this configuration decision.

## GitHub Pages

The repository owner authorized automatic browser deployment. `.github/workflows/ci.yml` publishes
on pushes to `main`, including merged pull requests, or a manual workflow run targeting `main`.
Pull-request runs only verify. Main runs are serialized so a new commit does not interrupt an
active deployment; pull-request verification can replace stale runs.

The browser job builds and tests production plus support bundles, then uploads only `dist/web`
with the pinned `actions/upload-pages-artifact` action. The deployment job waits for the native
Linux/macOS/Windows matrix, quality, WASM, browser, and native renderer checks. A failed gate blocks
publication. It uses `github-pages`, `actions/configure-pages`, and `actions/deploy-pages`, with
`pages: write` and `id-token: write` scoped to that job. No personal token, deployment secret, or
generated-assets branch is needed. The downloaded game requires no GitHub credentials.

Enable **Settings → Pages → Build and deployment → Source: GitHub Actions** once before the first
deployment. This is a repository administration operation; the connected GitHub code tools do not
expose it. GitHub Pages for a private repository requires an eligible GitHub plan. Do not change
repository visibility to work around an unavailable plan.

The expected project URL is `https://matanlurey.github.io/low-kingdoms/`. Relative loader/module/WASM
paths preserve the project subpath; GitHub Pages supplies HTTPS and the WASM content type. Only
production files and the third-party notices are published, with no read-only test bridge.

After deploying, CI uses the exact `page_url` returned by GitHub and runs real Chromium on all four
configured viewports. It checks resources, application readiness, captured pixels, CSS/DPR bounds,
absence of the bridge, and keyboard About/Back navigation. These failures fail the deployment job;
the uploaded `pages-browser-evidence` artifact retains images, errors, and traces. Record the run,
deployed URL, actual browser/GPU environment, and manually inspected captures before claiming the
site works. A successful artifact upload alone does not establish a working deployment.

## Dependency updates

1. Identify the implemented behavior or defect requiring the change. Obtain separate scope for a
   Bevy migration; its exact scaffold pin is not an invitation to follow every new release.
2. Read the version-matched upstream changelog/source and update the one authoritative pin.
3. Resolve and review lockfile changes. Keep `wasm-bindgen-cli` matched to the resulting crate.
4. Inspect `cargo tree -e features --locked -p low-kingdoms` for host and the same command with
   `--target wasm32-unknown-unknown`. Avoid accidental renderer/editor/3D feature expansion.
5. Run `just check`, `just check-web`, and the affected native/browser smoke checks. Production
   and `test-support` are distinct configurations; do not use a blanket `--all-features` gate.
6. Record tested versions, screenshots, measured bundle size/build timing, and blockers in the
   acceptance plan. Refresh docs/tool lockfiles only where the actual change requires it.
