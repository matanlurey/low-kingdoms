# Working in Low Kingdoms

Low Kingdoms is a private Rust/Bevy application scaffold.
It currently contains application infrastructure: a title screen, About, and navigation.
Gameplay is outside this assignment. Do not introduce gameplay content, domain types, or placeholder APIs.
Do not add a Start, Play, or Continue control until gameplay is actually implemented and authorized.

## Before editing

1. Inspect `git status` and preserve unrelated work.
2. Read this file, [STYLE_GUIDE.md](STYLE_GUIDE.md), and the instructions applicable to each edited directory.
3. For `crates/client/`, read [its AGENTS.md](crates/client/AGENTS.md) explicitly.
4. Read [the active plan](docs/plans/scaffold.md) and identify the smallest observable change.
5. Apply an appropriate repository-local skill rather than inventing another workflow.

The maintained skills are:

- [bevy-change](.agents/skills/bevy-change/SKILL.md): client, screen, input, or platform changes.
- [rust-quality](.agents/skills/rust-quality/SKILL.md): Rust, dependencies, APIs, or workspace changes.
- [verify-native-web](.agents/skills/verify-native-web/SKILL.md): startup, rendering, and completion evidence.

## Package map

| Location          | Responsibility                                                        |
| ----------------- | --------------------------------------------------------------------- |
| `apps/game`       | Thin executable entry point, package and binary `low-kingdoms`.       |
| `crates/client`   | Application composition, screens, UI interaction, and platform edges. |
| `tools/xtask`     | Host-only command orchestration; never an application dependency.     |
| `tools/web-tests` | Development-only browser tests against generated WASM bundles.        |
| `web`             | Static canvas host, loader, and CSS; no separate menu implementation. |
| `assets`          | Asset policy; no external game art or font files in this scaffold.    |

The dependency direction is executable to client to Bevy.
The host tool must not pull in the renderer. Target-build the application packages for WASM, not the workspace.
Use the same production screen/navigation plugins in native, WASM, and headless tests.
Keep platform-specific work at `crates/client/src/platform/` and launch configuration at the application edge.
Future domain code belongs in an ordinary Bevy-independent Rust library, documented but not created here.
See [architecture](docs/architecture.md) for the implemented boundaries.

## Versions and dependencies

Bevy is exactly `=0.19.1`, with default features disabled.
Use its tagged source and version-matched documentation; do not copy APIs from `main` or older tutorials.
The exact Rust toolchain is in `rust-toolchain.toml`; workspace edition is 2024 and resolver is 3.
Every package is unpublished and inherits workspace lints. Commit the application `Cargo.lock`.
Use `--locked` after dependency resolution. Auxiliary pins are documented in [development](docs/development.md).
The `wasm-bindgen-cli` version must match the resolved crate version; a mismatch is an error.
Do not add dependencies for future gameplay or silently migrate the engine to fix an unrelated issue.
Inspect host and WASM feature trees after feature/dependency changes.
Production excludes `test-support`, WebGPU, developer/editor features, and native dynamic linking.

## Commands

Run these from the repository root; [development](docs/development.md) describes setup and prerequisites.

```sh
just doctor           # read-only environment report
just setup            # pinned project development tools and browser dependencies
just dev              # actual native window
just dev-web          # rebuild and serve the actual WASM application
just build            # native release distribution
just build-web        # production static distribution
just check            # formatting, structure, host lints, tests, and docs; no GPU
just check-web        # separate application WASM compile/lint gate
just smoke-native     # real native renderer; capture evidence or fail
just smoke-web        # real browser, generated bundle, input, and screenshots
just check-all        # all mandatory gates, including rendering
```

For focused iteration, use `cargo check --locked -p low-kingdoms-client`, then relevant tests.
Before committing a Rust/tooling change, run `just check` and the affected target/rendering gates.
Use `just fmt`, `just fmt-check`, `just clippy`, `just test`, and `just doc` to run individual gates.
Do not use indiscriminate `--all-features` builds; production and test-support configurations are distinct.
CI uses the same maintained command implementation rather than a second script sequence.
Authorized GitHub Pages publishing runs only for `main` after every required CI job passes.
It deploys the production bundle already checked by the browser suite, then tests the public URL.
Read [Pages development](docs/development.md#github-pages) before changing this deployment.
Just uses pinned Node to pass argument arrays; direct `cargo xtask dev` needs no Node runtime.

## Quality and evidence

The root manifest is the lint authority. Strict Clippy and rustdoc run with warnings denied.
Fix ordinary lint failures. Framework constraints may use a leaf-level `#[expect(..., reason = "...")]`.
Do not disable a lint group, add blanket cast/dead-code allowances, or add dummy references.
Keep modules private and use `pub(crate)` by default; expose only the required bootstrap API.
Use Bevy logging in the client and actionable errors at process/platform boundaries.
No production `unwrap`, `expect`, `todo`, `unimplemented`, or debug macros without a justified local policy.

Compilation establishes target/type compatibility; headless tests establish behavior and lifecycle.
Neither proves that a renderer displayed readable pixels. Inspect native and browser captures separately.
Exercise real pointer, touch, and keyboard input, resize, high DPI, and the static bundle subpath.
See [testing](docs/testing.md) for viewports, the read-only test bridge, and screenshot review.
Do not automatically bless snapshots, suppress application errors, or turn missing GPUs into successful skips.
Store screenshots, traces, logs, and measurements under ignored `evidence/`.
Record checks as passed, failed, blocked, or not run in the active plan.
An unavailable environment is a blocker to that check; keep the acceptance check in place.

## Scope and handoff

Do not add saves, accounts, networking, analytics, runtime LLM services, editor tooling, or release automation.
Do not assert a license for original game work or copy Retroglyph's license onto this application.
Preserve third-party attribution in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Do not commit, push, publish, deploy, change access, or perform destructive Git operations without authorization.
Instructions explicitly authorizing a Git operation take precedence over this default boundary.

Finish with exact versions and start commands, an actual check-outcome table, and evidence paths.
Separate built, headless-tested, visually inspected, and not-run claims, including the OS/browser environment.
Report bundle size/timing measurements when collected, and explain remaining blockers.
Never silently weaken acceptance criteria or claim unexecuted platforms work.
