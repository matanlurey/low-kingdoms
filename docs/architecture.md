# Architecture

The workspace has three implemented packages. `apps/game` depends on `crates/client`; the client
depends on Bevy and narrowly required browser bindings. `tools/xtask` orchestrates host processes
and does not depend on the client or renderer. Browser tests live outside the Rust package graph.

| Location                      | Owned behavior                                                       |
| ----------------------------- | -------------------------------------------------------------------- |
| `apps/game/src/main.rs`       | Calls the client's module-qualified bootstrap API.                   |
| `crates/client/src/app.rs`    | Application composition and renderer/window setup.                   |
| `crates/client/src/screens/`  | Title/About state, entry construction, and state-scoped cleanup.     |
| `crates/client/src/ui/`       | Shared style, input/focus, and navigation activation.                |
| `crates/client/src/platform/` | Native/browser launch edges and capability differences.              |
| `tools/xtask/`                | Root/path resolution, subprocesses, packaging, checks, and evidence. |
| `tools/web-tests/`            | Real browser input and inspection of generated WASM distributions.   |
| `web/`                        | Canvas host and WASM startup/error loader.                           |

## Application composition and ownership

The executable delegates to `low_kingdoms_client::app::run()`. Client composition installs the
screen and shared UI interaction plugins. Native and WASM use these same plugins and screen
construction functions. Renderer/window setup is separable so headless tests can compose the
actual application behavior without initializing a GPU.

The persistent application camera is created once. Bevy application state owns the title and About
screens. Each screen has one root with `DespawnOnExit`; descendant ownership removes its controls
and text on exit. Repeated About/Back transitions must not accumulate roots, cameras, handlers, or
observers. Meaningful `Name` values and small marker components give tests entity identity without
making internal presentation entities public application APIs.

Input activation reaches the production navigation path. Screen state is authoritative in Bevy.
The browser does not maintain its own screen model or replace the rendered menu with DOM controls.
Centralized style values and logical viewport dimensions control layout; physical framebuffer
dimensions and device pixel ratio remain separate from input/layout coordinates.

## Platform boundaries

The native edge creates a resizable window titled Low Kingdoms. Native Quit requests Bevy's normal
application exit. Browser configuration uses `#game-canvas`, hides Quit, and communicates startup
readiness to the loader. Escape from About returns to the title on both platforms.

Browser output uses WebGL2 with no WebGPU or threaded WASM requirement. HTML/CSS owns the canvas
parent's available size, while Bevy renders the product UI. Relative runtime URLs allow the same
static output at a root URL or a subpath. There is no runtime Node service, CDN, service worker,
network protocol, or gameplay state in JavaScript.

The normal `low-kingdoms-ready` event dismisses the loading overlay after live UI readiness. It is
not proof of pixel correctness. The non-default `test-support` feature supplies native screenshot
support and a read-only browser bridge named `window.__LOW_KINGDOMS_TEST__`. It derives observations
from the live world and permits no state mutation. Production packaging excludes that bridge.

## Tooling and distributions

`Justfile` uses pinned Node as its portable recipe interpreter and delegates substantial work to
host-only `xtask`. It passes arguments through `spawnSync` arrays without shell interpolation.
Direct `cargo xtask` remains the Rust-only command path; CI calls the same maintained commands.
Only application packages are built for `wasm32-unknown-unknown`; target-building the whole workspace
would incorrectly include host tools. Cargo profiles separate native release from size-oriented
`wasm-release`. Native dynamic linking is not required.

Production output belongs in `dist/native/` and `dist/web/`. Browser test support uses
`dist/web-test/`. Screenshots, logs, traces, and measurements belong in ignored `evidence/`.
Generated output is never a new source of application truth.

Development static serving calls the pinned `http-server` API through `tools/web-tests/serve.mjs`.
The wrapper binds to loopback directly, avoiding unnecessary CLI network-interface enumeration.
It does not become part of the shipped static application.

## Future library boundary

Future domain/rules code should live in an ordinary Rust library without a Bevy dependency.
Rendering components and Bevy entity IDs must not become its public representation. Future
simulation or server code could depend on that library without pulling in the renderer.

That dependency rule is the entire future boundary in this scaffold. No domain crate, empty
interface, content schema, protocol, or gameplay model is created now.
