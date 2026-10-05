---
name: rust-quality
description: Add or refactor Low Kingdoms Rust code, public APIs, errors, dependencies, lints, and workspace structure with the repository's strict inherited quality gates.
---

# Rust quality

1. Read root `AGENTS.md`, applicable nested instructions, `STYLE_GUIDE.md`, and relevant manifests.
   Inspect `git status`; preserve unrelated work. The root manifest is the lint authority.
2. Identify the implemented behavior served by each API/dependency. Keep modules private and use
   `pub(crate)` by default. Remove unused or speculative scaffolding instead of silencing it.
3. Handle real errors with contextual diagnostics. Keep client public error contracts small and
   explicit. Use Bevy logging in client code; do not panic for expected unavailable capabilities.
4. Ensure every package inherits workspace lints and metadata and remains `publish = false`.
   Keep host-only dependencies out of the application and renderer out of host tooling.
5. Fix lint failures directly. If a leaf framework constraint requires an exception, use a local
   `#[expect(..., reason = "...")]` and explain the constraint. Do not add preemptive exceptions.
6. Run `just fmt`, `just clippy`, `just test`, and `just doc`, or `just check` for the combined gate.
   Run `just check-web` when application APIs, dependencies, or features affect WASM.
7. Review the diff for blanket allowances, accidental public items, placeholders, unnecessary
   dependencies, and unrelated lockfile churn. Record meaningful outcomes in `docs/plans/scaffold.md`.

Do not upgrade Bevy, disable `pedantic`/`nursery`, or weaken documentation checks to repair a local
error. Use explicit production/test-support configurations rather than `--all-features`.
Dependency updates require matched upstream documentation, locked resolution, host/WASM feature
inspection, and affected rendering checks. See `docs/development.md` for the update procedure.
