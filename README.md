# Low Kingdoms

A code-first Rust/Bevy application scaffold with a title screen and an About screen. There is no
gameplay yet. The application UI is rendered by Bevy on native desktop and in the browser; the web
page supplies a canvas, loading status, and startup-error reporting.

The title offers **About**, plus **Quit** on native desktop. About includes the application version
and **Back**. Mouse, touch, Tab/Shift+Tab, Enter/Space, and Escape are the intended controls. Escape
returns from About and does not exit the title screen.

## Start

Install Rust through rustup, Just, and the pinned Node/npm versions listed in
[development](docs/development.md). The repository pins Rust **1.98.0** and Bevy **0.19.1**.
Run commands from the repository root. A usable native display and graphics adapter are required
for the native application; browser rendering requires WebGL2.

```sh
just doctor
just setup
just dev
```

Just recipes use the pinned Node release to invoke the Rust tool with argument arrays. For a
native-only path without Node, use `cargo xtask dev` directly; the native application itself has
no Node dependency.

For the browser:

```sh
just dev-web
```

Open the localhost URL printed by that command. It rebuilds on invocation; it does not provide hot
reload. Serve browser output over HTTP, not by opening `index.html` as a local file.

```sh
just build       # dist/native/
just build-web   # dist/web/, production static files
```

The web distribution uses relative paths and can be hosted at a root URL or a subpath. It has no
runtime Node requirement, CDN dependency, server application, or account system. Commands build
locally; they do not deploy or publish anything.

CI publishes the verified production WASM bundle to GitHub Pages after a merge/push to `main`
passes every required check. Pull requests do not deploy. Open the browser build at
[matanlurey.github.io/low-kingdoms](https://matanlurey.github.io/low-kingdoms/).
Pages uses GitHub Actions as its publishing source. CI checks the published application with real
Chromium after deployment. See [Pages deployment](docs/development.md#github-pages).

## Verify

```sh
just check          # formatting, structure, strict host lints, headless tests, and docs
just check-web      # application-only WASM compile and lints
just smoke-native   # actual native window and screenshot evidence
just smoke-web      # actual generated WASM bundle and browser input/screenshots
just check-all      # all of the above
```

These checks establish different things. Headless tests cover screen navigation and entity
lifecycle; renderer captures need visual inspection. A missing graphics adapter or browser fails
or blocks rendering verification rather than passing as a skip. Generated evidence is under
`evidence/` and is ignored by Git. See [testing](docs/testing.md) and the
[acceptance plan](docs/plans/scaffold.md) for the checks actually completed and environment limits.

## Workspace

| Package                                 | Role                                                  |
| --------------------------------------- | ----------------------------------------------------- |
| `low-kingdoms` (`apps/game`)            | Thin native/WASM executable entry point.              |
| `low-kingdoms-client` (`crates/client`) | Bevy application, screen lifecycle, and presentation. |
| `xtask` (`tools/xtask`)                 | Host-only development/build/test orchestration.       |

All packages are unpublished. [Architecture](docs/architecture.md) explains dependency direction;
[STYLE_GUIDE.md](STYLE_GUIDE.md) records the Rust conventions. [AGENTS.md](AGENTS.md) is the
operational map for coding agents, with three substantive repository-local skills under `.agents/`.

Linux, macOS, and Windows are intended desktop targets. CI configuration and target compilation
are distinct from an interactive runtime test on each operating system. Native mobile, console
packaging, Steam integration, editor workflows, and gameplay are deferred. A phone-shaped browser
viewport does not establish Safari/iOS hardware support or Steam Deck certification.

Third-party attribution is in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). No license for
original Low Kingdoms work is asserted by this scaffold.
