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
`rust-version`; the repository does not claim an older untested compiler.

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
Injected anonymous unit definitions suffice to execute complete rules tests. Milestone 1A stays
content-agnostic; the separate battle library supplies Milestone 1B prototype content and sessions.

```sh
cargo test --locked -p low-kingdoms-rules
cargo clippy --locked -p low-kingdoms-rules --all-targets -- -D warnings
cargo check --locked -p low-kingdoms-rules --target wasm32-unknown-unknown
cargo tree --locked -p low-kingdoms-rules
just check
just check-web
```

The workspace gates include rules tests, Clippy, rustdoc, metadata, and crate-local instructions.
The WASM gate checks/lints the rules and battle libraries alongside the client and executable while excluding
host tools. All existing CI jobs continue to use these commands; the native matrix executes the
headless rules suite on Linux, macOS, and Windows. Renderer smoke gates remain in place.

`Game::new(config, definitions)` returns a game and initialization events. Submit typed decisions
with `game.apply(player, &command)`; inspect `game.view_for(player)` and filtered
`events.for_player(player)`. Record `LoggedCommand` entries externally for `Game::replay`.
Never deliver authoritative snapshots to player-facing code. Snapshot JSON and injected catalogs
are sufficient for exact continuation. Rules contain no filesystem/database/networking code; local
1B save I/O lives at the client platform edge.

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

Native smoke captures four scenes through `test-support`: Title, Mulligan, an actually deployed
battlefield, and Results. Each scene receives an isolated save directory and a bounded capture
before exit. For direct diagnostic launches, use `--screenshot PATH --scene title|mulligan|battle|results`;
these are test-support options, not production menu or privileged gameplay APIs. Linux virtual-display
runs must provide a window manager/focus (CI uses Openbox inside Xvfb), otherwise AI correctly stays
paused. Inspect all four captures rather than treating completed files as visual approval.

Use `cargo check --locked -p low-kingdoms-client` for a focused type check, then focused tests and
`just check`. Application changes affecting WASM also need `just check-web`. Presentation/startup
changes need the affected renderer smoke checks and screenshot inspection.

## Prototype battle iteration

Read [Milestone 1B](plans/milestone-1b.md) and [its acceptance record](plans/milestone-1b-acceptance.md)
for the approved scope and actual verification. The same Bevy gameplay systems run natively and in
WASM. Select Play, keep/mulligan the opening hand, then confirm up to two starting resources.
Inspect hand cards before Play; select a ready friendly unit and Attack before selecting a target.
Pass is persistent and Claim Initiative is in the secondary menu. Round-two onward resourcing is
inline. Results offer Play Again with a fresh seed or Main Menu.

Compact browser layouts use touch-selectable cards, scrollable unit rows, a hand tray, and overlays.
Swiping reveals offscreen targets while retaining attacker selection. The supported minimum is
375×667 CSS pixels; desktop and landscape layouts adapt automatically, including resize/DPR changes.
AI timing is Normal 750 ms, Fast 150 ms, or Instant zero delay; inactive tabs/windows pause execution.
Timing never changes deterministic choices. The event log retains the last 256 filtered events.

```sh
cargo test --locked -p low-kingdoms-battle
cargo clippy --locked -p low-kingdoms-battle --all-targets -- -D warnings
cargo check --locked -p low-kingdoms-battle --target wasm32-unknown-unknown
cargo test --locked -p low-kingdoms-client
just check
just check-web
just smoke-native
just smoke-web
```

The maintained Playwright matrix uses the supported full `channel: "chromium"` browser for real
multi-tab focus/visibility checks. Setup installs the pinned browser; a working headless shell is
not a substitute for the full-channel background scenarios. If local process/UNIX-socket policy
blocks Chromium's ProcessSingleton, record the full smoke as blocked and verify on a provisioned
runner. Do not disable the background policy or replace actual tab activity with a mocked field.
The full 55-case smoke is bounded to 3600 seconds (browser CI job 90 minutes). Complete-match
tests allow 600 seconds, with only `phone-landscape-dpr2` allowed 900 seconds: its local 482.5-second
measurement and observed desktop CI/local 6.1/4.3-minute ratio project about 681 seconds. Ordinary
cases remain 120 seconds. This changes one bounded deadline without removing assertions, retrying
cases, or replacing the required full-channel check.

Headless battle tests cover content, view-only AI, complete simulations, and exact saved continuation.
Client tests use production command/scheduling/presentation paths. Browser tests use real compiled
WASM and pointer/touch input; they do not install an HTML gameplay mock or a writable game bridge.
Inspect actual captures before claiming the changed interface is usable. Controlled test seeds are
internal test configuration; production fresh matches use platform entropy.

## Prototype content

`assets/content/cards.ron` contains a Serde-compatible list of named vanilla unit records:

```ron
[
    (id: (1), name: "Infantry", cost: 1, power: 2, health: 2),
    (id: (2), name: "Archer", cost: 2, power: 3, health: 2),
]
```

`assets/content/decks.ron` contains exactly two records, each with twenty `DefinitionId` references
and a positive `stronghold_health`. For example, one record's shape is:

```text
(cards: [(1), (2), ... exactly twenty entries ...], stronghold_health: 30)
```

The abbreviated deck example is explanatory, not valid fixture data. Use the committed full RON
files and `Content::parse` tests. All six prototype units have only name, ID, cost, Power, and Health;
no card text, abilities, factions, keywords, or extra types are accepted. Costs and Power use `u32`
and may be zero; Health and stronghold Health must be positive. Unknown fields, empty names/catalogs,
duplicate IDs, wrong deck counts/lengths, and unknown references are rejected before match creation.

The catalog is ID-sorted; deck input order is preserved. `Content::load` embeds the files with
`include_str!`, providing identical native/browser content without runtime file access or fetches.
The exact canonical compatibility signature includes names, all statistics, deck references/order,
and strongholds. RON whitespace/definition ordering does not invalidate a save; changing its
meaningful content does. Parse once when initializing the client and reuse validated content.

## Local saves

Milestone 1B has one automatic active slot. Accepted human/AI commands and automatic phase changes
save the current authoritative session, including private pending decisions. Title offers Resume
Battle for a compatible unfinished save. Starting a new battle asks before replacement; Main Menu
preserves unfinished progress. Results clear resumable progress; a validated finished record left
by failed cleanup is refused as a resume target.

| Platform         | Active slot                                                                                                        |
| ---------------- | ------------------------------------------------------------------------------------------------------------------ |
| Browser          | Origin-scoped `localStorage` key `low-kingdoms.active.v1`.                                                         |
| Windows          | `%LOCALAPPDATA%/low-kingdoms/active-v1.json`.                                                                      |
| macOS            | `$HOME/Library/Application Support/low-kingdoms/active-v1.json`.                                                   |
| Linux/other Unix | Absolute `$XDG_DATA_HOME/low-kingdoms/active-v1.json`, otherwise `$HOME/.local/share/low-kingdoms/active-v1.json`. |

Native replacement writes/syncs a sibling temporary file, then replaces the active slot. It never
stores saves relative to the current working directory. Browser storage is scoped to protocol,
host, and port: the development server and deployed Pages site have separate slots. Browser privacy
settings, storage clearing, or quota limits can prevent persistence. There are no accounts, cloud
copies, database, or multiple-slot management.

The version-1 session envelope contains exact content identity, human/opponent roles, AI profile
version/stateless state, seed, revision, phase, full rules snapshot, ordered successful command log,
and bounded human-filtered history. The client may add versioned UI preferences outside session text.
Restore checks the engine snapshot and replays commands, verifies deterministic AI decisions, and
compares exact continuation state. Unsupported format/profile/roles/content and inconsistent saves
are rejected. No migration is implemented. Protective limits are 2 MiB and 4096 successful commands;
the visible history cap of 256 events does not truncate authoritative replay.

Treat save text and seeds as privileged: they contain or reconstruct hidden cards. Do not expose
or print them through UI, the read-only test bridge, or error logs. Local ownership is not an
anti-cheat boundary. Missing/corrupt/incompatible saves show recovery without constructing a partial
match. Storage failures preserve the last valid in-memory state and old slot where possible; retry
writes the current session and cannot reapply the successful game command. The UI surfaces failures
and pauses the affected path rather than silently continuing to overwrite progress.

## Command map

| Command             | Behavior                                                                  |
| ------------------- | ------------------------------------------------------------------------- |
| `just doctor`       | Read-only tool/target/prerequisite report.                                |
| `just setup`        | Explicit pinned project-tool and browser dependency setup.                |
| `just dev`          | Build/run native from the repository root.                                |
| `just dev-web`      | Rebuild development WASM and serve HTTP, default port 4173.               |
| `just build`        | Package native release output in `dist/native/`.                          |
| `just build-web`    | Package production static output in `dist/web/`, without test hooks.      |
| `just fmt`          | Format Rust and selected maintained text/config formats.                  |
| `just fmt-check`    | Check formatting without changes.                                         |
| `just clippy`       | Strict workspace host lints, including tests and tooling.                 |
| `just test`         | Host unit/integration tests and doctests; no GPU required.                |
| `just doc`          | First-party documentation with warnings denied.                           |
| `just check`        | Formatting, structural checks, host lints, tests, and docs.               |
| `just check-web`    | Compile/lint the application, battle, and rules packages for WASM.        |
| `just smoke-native` | Actual native title/mulligan/battle/results screenshots, bounded failure. |
| `just smoke-web`    | Generated bundles, real Chromium input, screenshots, and traces.          |
| `just check-all`    | All mandatory gates, including native/browser rendering.                  |

## Files, assets, and rendering features

`dist/`, `target/`, `.tools/`, `node_modules/`, and `evidence/` are generated/ignored. Commit Cargo
and npm lockfiles. The shipped web output uses relative URLs and requires HTTP; opening files with
`file://` is unsupported. Test the same static distribution at `/` and `/low-kingdoms/`.

Keep selected, visually reviewed reference captures in `docs/references/rendered/`, with their
environment and review notes in its README. They supplement the ignored per-run `evidence/`
artifacts and are maintained manual references, not an automatic cross-backend pixel baseline.

UI uses Bevy's embedded default font. Prototype RON is embedded into the battle library; no loose
font/art/content files are required at runtime.
[The asset policy](../assets/README.md) and [notices](../THIRD_PARTY_NOTICES.md) govern later additions.
Distributions include the notices and no loose runtime asset directory; `assets/README.md` is
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

After deploying, CI uses the exact `page_url` returned by GitHub and runs real Chromium on all five
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
