# 0001: Application scaffold

Status: accepted for the initial scaffold. Date: October 4, 2026.

## Decision

Use exact Bevy `0.19.1` and Rust `1.98.0` in a small virtual workspace: a thin executable, one
client library, and host-only `xtask`. The same code-defined UI/navigation runs natively and in
WASM. Keep gameplay outside this assignment and document only a future Bevy-independent library
dependency direction.

Use the supported `ui_bevy_render` feature group with explicit platform/default-font/WebGL2
support instead of Bevy's full default features. Static WASM packaging uses Cargo and a lockfile-
matched `wasm-bindgen-cli`; the shipped browser application needs no frontend framework or Node
server. WebGL2 is the initial browser renderer. WebGPU remains excluded because the pinned release
labels it experimental and enabling it overrides WebGL2 rather than providing automatic fallback.

Adopt Retroglyph's strict lint/API discipline with additional production panic/placeholder gates.
Keep local `AGENTS.md` and three procedural skills so generated code is checked against a compact
operational map. Native/browser screenshot checks complement compilation and headless tests.

## Tradeoffs

Exact pins and feature inspection require intentional dependency updates. The smaller UI-oriented
feature set reduces unused subsystems but still relies on the supported upstream group rather than
hand-minimizing every transitive feature. WebGL2 favors an available browser baseline over newer
renderer features. Native and browser smoke environments need real graphics/browser capabilities.

Development and shipping profiles, including build scripts/procedural macros, use one codegen unit
to avoid malformed optimized objects observed on constrained build hosts. This trades parallel
code generation for a portable build configuration; runtime dependency optimization stays at
level 3, and the selected stable Rust pin remains 1.98.0. The size-oriented WASM profile disables
LTO to limit cold build/link cost and avoid the unstable linking path observed as zero-length
thin-LTO intermediates on constrained hosts. This records the observed failure and configuration,
not a universal diagnosis. Just uses pinned Node for portable
argument-array invocation of Rust tooling; a direct `cargo xtask` native path remains available.
The development static-server wrapper binds loopback through the pinned `http-server` API so it
does not require network-interface enumeration.

The workspace adds one cross-crate bootstrap boundary and a host-tool package, but avoids splitting
screen code into empty service/domain packages. Strict lint groups sometimes need a narrowly
reasoned leaf exception for Bevy system signatures; they do not justify awkward general abstractions.

The initial scaffold deferred publishing. The owner subsequently authorized GitHub Pages delivery
of the verified production WASM artifact after all main-branch CI gates pass, with a real-browser
check against the returned public URL. This uses GitHub's standard Pages environment and scoped
workflow token instead of a deployment account, personal secret, or generated-assets branch.
There is no game content, editor integration, networking, serialization schema, or gameplay model.
This decision does not select their future design.
