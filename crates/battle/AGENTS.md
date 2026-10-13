# Working in prototype battles

Read root `AGENTS.md`, `STYLE_GUIDE.md`, `.agents/skills/rust-quality/SKILL.md`, and the complete
`docs/plans/milestone-1b.md` before editing. Milestone 1B authorizes content, AI, and local saves.

This crate remains Bevy-independent. The rules engine owns all authoritative match behavior.
`content` validates embedded RON definitions and exactly two twenty-card decks. Keep its six
vanilla units small; no abilities, keywords, commanders, actions, upgrades, affinities, or factions.
The exact content signature includes names, statistics, deck order, and stronghold configuration.

`ai` receives only its own `PlayerView`, legal commands, and public card definitions. Never pass it
`Game`, deck lists, snapshots, or the human's hidden hand/resource identities. Choose exclusively
from the legal list with explicit deterministic scoring and instance-ID tie-breaking. Do not add
randomness, multiround search, runtime LLMs, or a generic strategy framework.

Session and save code must apply commands through `Game`, retain only human-filtered event history,
and restore exact content/profile compatibility. Platform filesystem/browser storage stays in the
client, outside this library. Do not add raw hidden-state getters or globally shared mutable state.

Run this crate's focused tests, strict all-targets Clippy, rustdoc, and WASM check. Meaningful tests
cover content failures, AI tactics/setup/tie-breaking, complete matches, continuation, and privacy.
Use `just check` and `just check-web` for workspace verification; never weaken those gates.
