# Low Kingdoms

A Rust/Bevy card battle prototype for native desktop and WebAssembly. Milestone 1B connects the
deterministic, renderer-independent rules engine to a responsive human-versus-AI battle. Bevy
renders every game screen; the browser page supplies only the canvas, loading status, and startup
error reporting.

## Start

Install Rust through rustup, Just, and the pinned Node/npm versions listed in
[development](docs/development.md). The repository pins Rust **1.98.0** and Bevy **0.19.1**.
Run commands from the repository root. Native play requires a usable display and graphics adapter;
browser play requires WebGL2.

```sh
just doctor
just setup
just dev
```

For the browser:

```sh
just dev-web
```

Open the localhost URL printed by that command. It rebuilds on invocation and requires HTTP;
opening `index.html` as a local file is unsupported. Native-only development can use
`cargo xtask dev` without Node. Just and browser development use the pinned Node release.

## Play

Select **Play**, inspect the six opening cards, then choose **Keep Hand** or **Mulligan**. Select up
to two cards as starting resources and confirm. The opponent makes its setup decisions privately.
You play as player zero; opening initiative is seeded and may belong to either player.

Click or tap a hand card to inspect it, then choose **Play** when available. Select a ready friendly
unit, choose **Attack**, and select a highlighted enemy unit or stronghold. Units enter exhausted,
damage persists, and exhausted defenders still retaliate. Enemy units do not automatically protect
their stronghold. Close the preview, tap empty space, or press Escape to cancel selection.

**Pass** remains accessible during actions. **Claim Initiative** is in the secondary menu and ends
your actions for that round. Starting in round two, draw two cards, optionally resource one, then
ready units and resources. Inspect your resources through their counter and review public events
in the log. The opponent uses deterministic heuristics; Normal, Fast, and Instant change its action
delay to 750 ms, 150 ms, and zero respectively. Inactive windows/tabs pause AI scheduling.

Desktop uses opposing fields and an adaptive grid. Compact layouts use scrollable unit rows, a
collapsible hand tray, and dismissible panels. Mouse wheel/scroll and touch swipes reveal offscreen
cards without canceling an attacker. The supported browser minimum is **375 × 667 CSS pixels**.
Tab/Shift+Tab and Enter/Space retain keyboard navigation; Escape returns from About to the title.

Unfinished battles save automatically to one local slot. **Resume Battle** continues a compatible
save; starting a replacement requires confirmation. Results show Victory, Defeat, or Draw and
clear the resumable slot. **Play Again** generates a fresh seed. Storage is local to the native
user or browser origin, with no account or cloud service. See
[save locations and compatibility](docs/development.md#local-saves).

## Build and verify

```sh
just build         # dist/native/
just build-web     # dist/web/, production static files
just check         # formatting, structure, strict host lints, tests, and docs
just check-web     # application, battle, and rules WASM compile/lint checks
just smoke-native  # actual native renderer and screenshot evidence
just smoke-web     # actual WASM, browser gameplay, input, and screenshots
just check-all     # all mandatory gates
```

Headless tests establish behavior; rendered captures require visual inspection. Missing graphics
or browser prerequisites block their checks. Actual results and remaining requirements are recorded
in [Milestone 1B acceptance](docs/plans/milestone-1b-acceptance.md), with the approved
[specification](docs/plans/milestone-1b.md) and [testing guide](docs/testing.md).

The web distribution uses relative paths for root or subpath hosting. It has no runtime Node,
CDN, server application, or account dependency. CI publishes the verified production bundle to
[matanlurey.github.io/low-kingdoms](https://matanlurey.github.io/low-kingdoms/) only after all required
checks pass on `main`; pull requests do not deploy. The public site reflects deployed `main`,
not an unmerged implementation branch. See [Pages deployment](docs/development.md#github-pages).

## Workspace

| Package                                 | Role                                                                       |
| --------------------------------------- | -------------------------------------------------------------------------- |
| `low-kingdoms` (`apps/game`)            | Thin native/WASM executable entry point.                                   |
| `low-kingdoms-client` (`crates/client`) | Bevy screens, input, scheduling, responsive UI, and platform storage.      |
| `low-kingdoms-battle` (`crates/battle`) | Validated RON content, view-only deterministic AI, and resumable sessions. |
| `low-kingdoms-rules` (`crates/rules`)   | Authoritative two-player commands, views, snapshots, and replay.           |
| `xtask` (`tools/xtask`)                 | Host-only development/build/test orchestration.                            |

All packages are unpublished. [Architecture](docs/architecture.md) describes ownership and
[STYLE_GUIDE.md](STYLE_GUIDE.md) records conventions. [AGENTS.md](AGENTS.md) maps coding-agent
instructions and repository-local skills. Rules-only iteration uses
`cargo test --locked -p low-kingdoms-rules`; its unchanged gameplay specification and prior evidence
are in [Milestone 1A](docs/plans/milestone-1a.md) and
[its acceptance record](docs/plans/milestone-1a-acceptance.md).

Milestone 1B uses six vanilla unit definitions and two fixed twenty-card decks. Artwork, abilities,
commanders, action/upgrade cards, factions, deck building, advanced AI, networking, campaigns, and
cloud saves are deferred. Native mobile, Safari/iOS hardware, and physical handheld performance
are not established by Chromium viewport emulation. CI native builds and actual runtime checks are
reported separately for Linux, macOS, and Windows.

[Third-party notices](THIRD_PARTY_NOTICES.md) accompany distributions. No license for original
Low Kingdoms work is asserted by this repository.
