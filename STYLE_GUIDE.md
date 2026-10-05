# Low Kingdoms Rust style

The root `Cargo.toml` is the lint authority. Packages inherit its lints. This guide describes the
project decisions that need judgment rather than reproducing a Rust tutorial.

## Surface and structure

Use private modules and `pub(crate)` for internal collaboration. Public items serve the executable's
small bootstrap API; avoid a crate-root facade, homegrown prelude, and speculative interfaces.
Prefer module-qualified paths. A Bevy plugin is a useful composition unit when it owns a coherent
behavior, not a requirement for every function.

Keep runtime and host tooling dependencies separate. Add a dependency only for implemented behavior
or verification. A future Bevy-independent library is a dependency direction, not a reason to
create an empty crate, serializer, RNG wrapper, or generic service abstraction today.

## Lints and errors

Clippy `all`, `pedantic`, and `nursery` are strict; warnings become errors in lint/documentation
gates. First-party unsafe code is forbidden. This policy makes no claim about unsafe code in Bevy
or other dependencies. Production `unwrap`, `expect`, `dbg!`, `todo!`, and `unimplemented!` are
denied; test-only unwrap/expect follow the pinned Clippy configuration.

Fix ordinary code issues instead of exempting them. When a specific Bevy system signature or other
leaf implementation requires an exception, use the narrowest `#[expect(lint, reason = "...")]`.
Explain the constraint. Unfulfilled expectations fail so stale exceptions cannot accumulate.
Never disable a lint group or add a blanket numeric-cast/dead-code exemption.

Use small handwritten public error enums if the client needs a fallible API. Describe actual
failure conditions; do not invent a `Result` for an infallible operation. Host tooling may use
contextual error helpers where they improve process/path diagnostics. Expected missing tools,
files, adapters, and startup failures need an actionable error, not a panic or swallowed result.
Use Bevy logging in shared client code rather than a competing subscriber or direct printing.

## Comments and documentation

Use rustfmt defaults, including the 100-column budget. Hand-wrap comment prose near that width;
rustfmt does not reflow it. Comment contracts and reasons, not each obvious line.

Public documentation describes the present API. A brief first sentence provides the summary.
Add `# Errors` and `# Panics` only where those conditions apply, and describe the conditions rather
than merely naming an error type. Doctests must compile or use an accurate `no_run`, `compile_fail`,
or `text` annotation; do not use ignored examples to avoid a check.

Write en-US spelling, precise technical language, and concrete evidence. Keep change history in
commit/PR descriptions. Avoid marketing claims and unmeasured performance assertions.

## Application-specific departures from Retroglyph

The reference is [crates-lurey-io/retroglyph](https://github.com/crates-lurey-io/retroglyph), a Rust
workspace, not the older Dart repository under the owner's account. Its public-API, lint, comment,
and error discipline informs this application.

Low Kingdoms is a private Bevy application. It does not inherit Retroglyph's `no_std` constraints,
crates.io release automation, comprehensive backend matrix, license, or universal `--all-features`
gate. Renderer features and test support are checked as explicit supported configurations. A
missing required GPU fails the rendering check, and snapshot updates require inspection.
