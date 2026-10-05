# Assets

The initial application uses code-defined Bevy UI and Bevy's bundled default font. It does not
bundle external game art or additional font files. Fira Mono attribution and its SIL Open Font
License are recorded in [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

Keep future assets in this directory only when an implemented feature uses them. Record source,
author, license, and any distribution conditions when adding an asset. Do not download assets from
a CDN at runtime or depend on an absolute developer-machine path.

Native development commands run from the repository root. There are no loose runtime assets to
copy in this scaffold; the distribution omits this policy file and includes the notices. Extend
packaging when implemented behavior requires loose assets. Generated output is not source assets.
