# Assets

Milestone 1B embeds prototype RON data from `assets/content/cards.ron` and
`assets/content/decks.ron` into `low-kingdoms-battle` with `include_str!`. Both native and WASM load
and validate those same strings once before match creation. No runtime fetch, working-directory
lookup, or loose asset copying is required. The policy file itself is not shipped.

The catalog contains six original vanilla units with names, stable IDs, cost, Power, and Health.
The two decks each contain twenty references and positive stronghold Health. Definitions have no
abilities, keywords, factions, or additional card types. See
[the content format](../docs/development.md#prototype-content) and content validation tests before
editing the files. Changing names, statistics, deck order, or stronghold configuration invalidates
existing saves through their exact compatibility signature.

The application uses code-defined Bevy UI and Bevy's bundled default font, with no external card
art or additional font files. Fira Mono attribution and the SIL Open Font License remain in
[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md), which accompanies generated distributions.
No additional third-party asset was introduced by the prototype RON data.

Add other assets only when authorized implemented behavior needs them. Record source, author,
license, and distribution conditions. Do not download assets from a CDN at runtime or depend on
an absolute developer-machine path. If a feature later needs loose assets, update packaging and
target verification together. Generated output is not source content.
