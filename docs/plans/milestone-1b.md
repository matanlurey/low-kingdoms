# Milestone 1B — First Playable Battle

**Repository:** https://github.com/matanlurey/low-kingdoms

**Objective:** Implement a complete, functional, responsive, single-player card battle using the existing Milestone 1A rules engine, Rust, and Bevy.

**Deliverable:** A working native and WebAssembly game, comprehensive automated tests, documentation, and a GitHub pull request.

**Important:** This is an implementation task, not a planning exercise. Complete the milestone end-to-end. Do not stop after writing a specification or scaffolding.

---

# 1. Mission

Implement **Milestone 1B: First Playable Battle** for Low Kingdoms.

Low Kingdoms is a fantasy collectible card game inspired mechanically by Star Wars: Unlimited, with alternating player actions, persistent unit damage, card-based resources, and stronghold victory conditions.

Milestone 1A implemented a deterministic, Bevy-independent Rust rules engine.

Milestone 1B must integrate that engine into the existing Bevy application and deliver the first genuinely playable game.

A player should be able to:

1. Launch Low Kingdoms on desktop or in a browser.
2. Select **Play**.
3. Begin a battle against a basic AI opponent.
4. Inspect their opening hand and decide whether to mulligan.
5. Select starting resources.
6. Deploy units.
7. Attack enemy units and strongholds.
8. Pass or claim initiative.
9. Progress through rounds.
10. Observe the AI's actions.
11. Win or lose a complete battle.
12. Start another match.
13. Close or refresh the game and resume an unfinished battle.

The application must work with mouse and touch input and automatically adapt to desktop and mobile viewports.

## Primary success criterion

**A human must be able to play a complete Low Kingdoms match against an AI opponent on both desktop and mobile WebAssembly, using the existing Milestone 1A rules engine.**

This includes actual rendered gameplay and verified interactions—not merely compiling the application or demonstrating headless simulations.

---

# 2. Execution Instructions

You are authorized to:

- Inspect the repository and its history.
- Create and modify source files.
- Add necessary dependencies.
- Create RON-based prototype content.
- Add automated tests.
- Update documentation.
- Modify existing UI and application architecture.
- Update CI where necessary.
- Create a feature branch.
- Commit and push changes.
- Open a GitHub pull request.
- Fix CI failures on that PR.

Do not merge the PR.

Do not deploy changes outside the repository's established deployment workflow.

Work autonomously and complete the entire milestone.

Make ordinary engineering decisions yourself. Do not ask questions about naming, module organization, dependency placement, or minor implementation details.

If an ambiguity can be reasonably resolved using the decisions below, resolve it yourself and document the interpretation.

Ask for clarification only if a genuine product-level contradiction makes implementation impossible without changing an explicitly confirmed decision.

**Do not stop after completing only part of the implementation.**

---

# 3. Inspect the Existing Repository First

Before writing code, inspect the current repository thoroughly.

Read:

- `AGENTS.md`
- `STYLE_GUIDE.md`
- `Cargo.toml`
- `Cargo.lock`
- `rust-toolchain.toml`
- `docs/architecture.md`
- `docs/development.md`
- `docs/testing.md`
- `docs/plans/milestone-1a.md`
- `docs/plans/milestone-1a-acceptance.md`
- `crates/client/AGENTS.md`
- `crates/rules/AGENTS.md`
- Relevant instructions in `.agents/skills/`
- Existing GitHub Actions workflows
- Existing native and browser test infrastructure

Inspect the implementation of:

- `crates/rules`
- `crates/client`
- `apps/game`
- `tools/xtask`
- `tools/web-tests`
- `web/`

Understand the current Bevy application lifecycle, UI architecture, platform abstractions, testing conventions, and rules-engine public API.

### Important repository context

At the time this prompt was prepared, the repository had:

| Component                    | Status                   |
| ---------------------------- | ------------------------ |
| Rust 1.98.0                  | Pinned                   |
| Bevy 0.19.1                  | Pinned                   |
| Rust edition 2024            | Configured               |
| Native application           | Implemented              |
| WASM application             | Implemented              |
| Title/About screens          | Implemented              |
| `crates/rules`               | Milestone 1A implemented |
| Rules engine testing         | Implemented              |
| Playwright browser tests     | Configured               |
| Native rendering smoke tests | Configured               |
| GitHub Pages deployment      | Configured               |
| Actual gameplay UI           | Not implemented          |

**Verify the current repository state rather than assuming these details remain unchanged.**

## Existing rules-engine API

Milestone 1A already exposes domain types including:

- `Game`
- `Command`
- `AttackTarget`
- `PlayerId`
- `CardId`
- `DefinitionId`
- `UnitDefinition`
- `MatchConfig`
- `PlayerConfig`
- `PlayerView`
- `EventBatch`
- `Phase`
- `Outcome`

Its public interface includes operations equivalent to:

```rust
Game::new(config, definitions)

game.apply(player, &command)

game.legal_commands_for(player)

game.view_for(player)

event_batch.for_player(player)
```

It also implements deterministic replay and snapshot restoration.

**Use the actual implemented API. Do not create a parallel gameplay engine or redesign Milestone 1A unnecessarily.**

If you discover a genuine integration defect in 1A, fix it with focused tests and document the change.

## Existing instructions that restrict gameplay

Some repository documentation predates Milestone 1B and explicitly prohibits features such as Play controls, AI, content, and saves.

Those restrictions applied to the previous milestone.

This task explicitly authorizes those features for Milestone 1B.

Update the relevant instructions and documentation to reflect the new architecture and scope. Preserve unrelated repository conventions and restrictions.

---

# 4. Create the Milestone 1B Specification

Before implementation, add:

`docs/plans/milestone-1b.md`

This document must capture the 33 confirmed design decisions below, functional requirements, technical architecture, acceptance criteria, testing requirements, and exclusions.

**Do not open a separate planning PR or stop after writing the specification.**

The specification and implementation belong in the same feature branch and implementation PR.

Treat the decisions in this prompt as approved product requirements.

---

# 5. Confirmed Design Decisions

The following 33 decisions were established through a detailed design interview.

Do not revisit them without a concrete technical reason.

| #   | Area                 | Confirmed decision                                      |
| --- | -------------------- | ------------------------------------------------------- |
| 1   | Scope                | Functional battle prototype                             |
| 2   | Opponent             | Basic AI opponent                                       |
| 3   | Battlefield layout   | Responsive desktop CCG layout + compact mobile layout   |
| 4   | Card interaction     | Click/tap selection                                     |
| 5   | Mulligan/resourcing  | Dedicated mulligan screen; inline resourcing            |
| 6   | AI timing            | Brief delays between actions                            |
| 7   | Card display         | Adaptive cards                                          |
| 8   | Battlefield overflow | Adaptive grid                                           |
| 9   | Action feedback      | Contextual notifications + persistent event log         |
| 10  | Game speed           | Normal, Fast, Instant                                   |
| 11  | Lifecycle            | Title → Play → Battle → Results                         |
| 12  | AI strategy          | Simple deterministic heuristics                         |
| 13  | Prototype content    | Data-driven RON definitions and decks                   |
| 14  | Mobile platform      | Responsive browser game                                 |
| 15  | Persistence          | Automatic local saves                                   |
| 16  | Invalid moves        | Prevent invalid interactions                            |
| 17  | Damage/exhaustion    | Remaining Health + visual exhaustion                    |
| 18  | Minimum viewport     | 375 × 667 CSS pixels                                    |
| 19  | Card inspection      | Tap/click to inspect                                    |
| 20  | Testing              | Automated gameplay + browser tests                      |
| 21  | Background behavior  | Pause AI while inactive                                 |
| 22  | Failure behavior     | Fail gracefully                                         |
| 23  | AI setup decisions   | Deterministic mulligan/resource heuristics              |
| 24  | UI framework         | Native Bevy UI                                          |
| 25  | UI synchronization   | Hybrid state reconciliation + events                    |
| 26  | Active player        | Persistent turn indicator                               |
| 27  | Action controls      | Primary Pass button; Claim Initiative in secondary menu |
| 28  | Display scaling      | Automatic                                               |
| 29  | Offscreen targeting  | Manual scrolling                                        |
| 30  | Resource display     | Counters with inspection panel                          |
| 31  | Results              | Minimal Victory/Defeat screen                           |
| 32  | Save management      | One active save slot                                    |
| 33  | Play Again           | Fresh randomized match                                  |

Implement these decisions as described in the following sections.

---

# 6. Scope: Functional Battle Prototype

Milestone 1B is intentionally **functional rather than visually polished**.

The objective is to evaluate gameplay, usability, and interaction design without investing heavily in production presentation.

## Required

- Complete playable battles.
- Functional card rectangles with readable text.
- Basic responsive battlefield.
- Human versus AI.
- Mouse and touch interactions.
- Card selection and inspection.
- Mulligan and resource selection.
- Unit deployment.
- Attack targeting.
- Passing and initiative.
- Turn indicators.
- Damage and readiness indicators.
- AI action feedback.
- Event history.
- Automatic saving and resuming.
- Victory and defeat screens.

## Deferred

Do not implement:

- Full card illustrations.
- Elaborate animations.
- Particle effects.
- Sound effects.
- Music.
- Commander mechanics.
- Action cards.
- Upgrades.
- Keywords.
- Factions or affinities.
- Card collection management.
- Deck building.
- Sophisticated AI.
- Multiple difficulty settings.
- Multiplayer.
- Campaign progression.
- Roguelike overworld.
- Account systems.
- Cloud saves.
- PWA installation.
- Fullscreen-specific mobile features.
- Production-quality visual effects.

**Do not expand the milestone into a full game vertical slice.**

---

# 7. Application Architecture

Use the existing Rust and Bevy architecture.

## 7.1 Required dependency boundaries

The intended structure is:

```text
apps/game
    |
    v
crates/client
    |
    +------> crates/rules
    |
    +------> Bevy
    |
    +------> AI
    |
    +------> Content Loader
    |
    +------> Persistence
```

The exact module or crate boundaries for AI, content, and persistence are implementation decisions.

However:

- `crates/rules` remains independent of Bevy.
- Gameplay rules must not be duplicated in the UI.
- AI must not bypass the rules engine.
- Persistence must preserve authoritative rules state.
- Platform-specific storage and browser integration must remain isolated.
- Native and WASM must share the same gameplay implementation.

Prefer implementing AI as a small, separately testable Rust module or crate.

Do not introduce a generalized game framework or unnecessary abstraction layers.

## 7.2 Bevy UI

Use **native Bevy UI** for all gameplay presentation.

This includes:

- Battlefield.
- Cards.
- Hands.
- Resource indicators.
- Strongholds.
- Buttons.
- Menus.
- Card previews.
- Event log.
- Notifications.
- Mulligan screen.
- Results screen.

Do not implement a separate HTML/CSS gameplay interface.

HTML/CSS may continue managing the existing browser canvas host and bootstrap behavior.

Avoid custom sprite-based UI rendering when standard Bevy UI components suffice.

## 7.3 Screen state

Extend the existing application screen state to support:

```text
Title
About
Mulligan
Battle
Results
```

You may model setup and battle subphases differently if the architecture is cleaner.

The existing title and About functionality must continue working.

Add new controls for:

- Play.
- Resume Battle, when applicable.
- New Battle confirmation.
- Play Again.
- Main Menu.

Preserve the existing screen ownership and cleanup patterns.

Do not accumulate duplicate UI roots, observers, cameras, or event handlers during transitions.

---

# 8. Authoritative Game State and UI Synchronization

**Milestone 1A owns all gameplay state.**

The Bevy client must not independently calculate or enforce combat rules.

## 8.1 Presentation model

Maintain a lightweight presentation model derived from the appropriate player-visible rules-engine view.

The presentation model may contain:

- Visible player state.
- Unit presentation data.
- Hand presentation data.
- Selected card.
- Current interaction mode.
- Scroll positions.
- Open panels.
- Notification state.
- UI speed preference.

It must not become a second authoritative gameplay model.

## 8.2 Command execution

All gameplay actions should follow a consistent path:

```text
Player input
    |
    v
UI interaction handler
    |
    v
Construct rules-engine command
    |
    v
Validate and apply through Game
    |
    v
Receive EventBatch
    |
    +------> Update presentation
    |
    +------> Update event log
    |
    +------> Show notification
    |
    +------> Save authoritative state
    |
    v
Schedule next player / AI action
```

Do not mutate unit damage, readiness, resources, or stronghold Health directly from UI code.

## 8.3 Hybrid synchronization

Use a hybrid strategy:

- Structured events drive notifications and history.
- Changed state updates affected UI entities.
- Authoritative views provide the actual displayed gameplay state.
- Full reconciliation occurs after starting or restoring a match.
- Full reconciliation is also available after detecting an inconsistency.

Preserve transient presentation state where appropriate.

For example, updating a unit's Health should not unnecessarily reset the player's scroll position.

## 8.4 Hidden information

Always render the human player's permitted view.

The human may inspect:

- Their hand.
- Their resources.
- All public units.
- Public stronghold state.
- Public discard piles.

They may not inspect:

- AI hand identities.
- AI resource identities.
- Deck order.
- Other hidden authoritative data.

Use the existing rules engine's player-specific views and filtered events.

Do not accidentally expose the complete authoritative state through production UI or browser globals.

---

# 9. Battlefield Layout

Support both traditional desktop CCG layout and compact mobile portrait layout.

## 9.1 Desktop and wide viewports

Use a traditional opposing-battlefield arrangement:

```text
+------------------------------------------------------+
| Opponent | Stronghold | Resources | Deck | Hand count |
+------------------------------------------------------+
|                                                      |
|                 OPPONENT UNITS                       |
|                                                      |
+------------------------------------------------------+
|                                                      |
|                   YOUR UNITS                         |
|                                                      |
+------------------------------------------------------+
| Your Stronghold | Resources | Deck                    |
+------------------------------------------------------+
|                                                      |
|                    YOUR HAND                         |
|                                                      |
+------------------------------------------------------+
| Actions | Active Player | Speed                       |
+------------------------------------------------------+
```

The event log may appear in a right-hand sidebar.

The exact spacing and visual arrangement are implementation details.

## 9.2 Mobile portrait

Use a compact layout:

- Opponent information at the top.
- Opponent units in a horizontal scroll region.
- Your units in a horizontal scroll region.
- Compact stronghold and resource indicators.
- Your hand in a collapsible bottom tray.
- Persistent Pass control.
- Secondary action menu.
- Collapsible event log.
- Card previews in dismissible overlays.

Prioritize battlefield visibility.

The player must not need to scroll the entire page to access essential actions.

## 9.3 Responsive behavior

Automatically adapt based on viewport dimensions and orientation.

Required:

- Desktop layout.
- Mobile portrait layout.
- Mobile landscape layout.
- Window resizing.
- Device pixel ratio changes.
- Orientation changes.
- Basic mobile safe-area handling.

Avoid separate gameplay implementations for mobile and desktop.

## 9.4 Minimum viewport

Target a minimum supported viewport of:

**375 × 667 CSS pixels.**

At this size:

- Important text must remain readable.
- Cards must remain selectable.
- Essential controls must remain accessible.
- The player must be able to complete every gameplay action.
- No essential interface should be permanently obscured.

Smaller viewports are best-effort.

## 9.5 Scaling

Use automatic UI scaling.

No manual UI-scale setting is required.

Prefer scrolling and adaptive arrangement over shrinking cards or text below usable dimensions.

---

# 10. Card Presentation

Use compact, text-based placeholder cards.

No artwork is required.

## 10.1 Battlefield cards

Each visible unit displays:

- Name.
- Resource cost.
- Power.
- Remaining Health.
- Accumulated damage indicator.
- Ready/exhausted state.

Example:

```text
+--------------+
| Infantry   2 |
|              |
|              |
|    3 / 2     |
|              |
| Damage: 1    |
+--------------+
```

Keep the presentation compact.

## 10.2 Hand cards

Hand cards may be larger where space permits.

On mobile, hand cards should remain compact and expand when selected.

## 10.3 Card inspection

Clicking or tapping a visible card opens an inspection preview.

On desktop, previews should appear beside the battlefield where practical.

On mobile, use a dismissible overlay.

Inspection must not accidentally initiate an attack.

For friendly units, contextual controls distinguish between:

- Inspect.
- Attack.

For hand cards, contextual controls distinguish between:

- Inspect.
- Play.

## 10.4 Health representation

Display:

**Power / Remaining Health**

For example:

`3 / 2`

A damaged unit should also indicate accumulated damage separately.

Calculate remaining Health from the engine's public Health and damage values.

Strongholds display remaining Health.

Do not implement health regeneration or healing.

## 10.5 Exhaustion

Exhausted units appear dimmed.

Readiness must remain distinguishable on compact mobile cards.

Do not require sideways card rotation.

Avoid relying exclusively on color to convey readiness.

---

# 11. Card Interaction Model

Use click/tap selection rather than drag-and-drop.

## 11.1 Playing units

The player:

1. Selects a card in hand.
2. Sees its expanded preview.
3. Selects **Play**.
4. The client submits the corresponding rules-engine command.
5. The unit enters the battlefield exhausted.
6. The UI updates and the action passes to the AI.

Unaffordable cards remain visible but cannot be played.

## 11.2 Attacking

The player:

1. Selects a ready friendly unit.
2. Chooses **Attack**.
3. The interface enters targeting mode.
4. Valid targets become highlighted.
5. The player selects an enemy unit or stronghold.
6. The client submits the attack command.
7. Combat resolves through Milestone 1A.
8. The presentation updates.

The selected attacker remains selected while the player scrolls to find a target.

## 11.3 Canceling selection

Allow selection or targeting to be canceled by:

- Clicking/tapping empty space.
- Pressing Escape.
- Closing the relevant overlay.

Cancellation must not mutate the match.

## 11.4 Invalid actions

Prevent invalid interactions.

- Unaffordable units cannot be played.
- Exhausted units cannot initiate attacks.
- Invalid targets are not selectable during targeting.
- Unavailable commands are disabled.
- The rules engine still validates every submitted command.

Unexpected validation errors produce a brief notification.

Do not duplicate rules-engine legality calculations in the UI.

Use `legal_commands_for` or equivalent existing APIs.

---

# 12. Battlefield Overflow

The Milestone 1A engine has no battlefield unit limit.

The UI must support arbitrary numbers of units.

## Desktop

Allow units to wrap into additional rows or use another readable grid arrangement.

## Mobile

Use horizontally scrollable unit rows.

Cards must maintain minimum usable dimensions.

Indicate when additional units are offscreen.

## Targeting

Players must be able to select offscreen targets by scrolling.

Selection and highlighting must persist while scrolling.

No separate target browser or target-selection menu is required.

Ensure the scroll interaction works with touch gestures and desktop input.

---

# 13. Mulligan and Resourcing

## 13.1 Mulligan

Provide a dedicated mulligan screen.

Display the six opening cards.

Controls:

- **Keep Hand**
- **Mulligan**

After the human submits a decision, complete the AI's corresponding decision as required by the engine.

Respect the one-mulligan limit.

The AI must not reveal its private hand.

## 13.2 Initial resourcing

Perform initial resourcing within the battlefield interface.

The player may select zero, one, or two cards.

Selected cards must be visually distinguishable.

Provide a clear **Confirm Resources** control.

Do not commit the selection until confirmation.

## 13.3 Subsequent resourcing

During regroup:

- Display a contextual prompt.
- Allow selection of zero or one card.
- Confirm the decision.
- Resume normal gameplay when both players' decisions are complete.

## 13.4 Disabled interactions

While resourcing:

- Normal attacks are disabled.
- Unit deployment is disabled.
- Passing is disabled.
- Initiative claiming is disabled.

Only the appropriate resource-selection controls are available.

## 13.5 AI setup decisions

The AI handles its own mulligan and resource decisions automatically.

Do not expose the AI's private cards in the human presentation.

---

# 14. AI Opponent

Implement a basic deterministic heuristic AI.

The AI must be good enough to produce meaningful test battles.

It does not need to be a competitive opponent.

## 14.1 Architectural requirements

The AI:

- Receives its own player-visible state.
- Uses legal actions provided by Milestone 1A.
- Returns a legal command.
- Never bypasses the engine.
- Does not modify authoritative state directly.
- Does not depend on Bevy rendering.
- Can be tested headlessly.
- Uses deterministic tie-breaking.

The AI must not access hidden human information unavailable to its player perspective.

## 14.2 Mulligan heuristic

Prefer keeping opening hands with:

- At least one affordable low-cost unit.
- A reasonable cost distribution.
- Enough useful cards to establish early battlefield presence.

Otherwise mulligan once.

Use deterministic rules.

## 14.3 Initial resourcing

Prefer resourcing:

- Redundant cards.
- Expensive cards that cannot be played soon.
- Cards with lower immediate strategic value.

Preserve inexpensive units where practical.

## 14.4 Subsequent resourcing

Consider:

- Current resources.
- Cards in hand.
- Available plays.
- Cost distribution.
- Immediate tactical opportunities.

Prefer resourcing the least useful card.

Keep the heuristic simple and deterministic.

## 14.5 Gameplay priorities

The AI should generally:

1. Identify immediate winning attacks.
2. Identify tactically favorable trades.
3. Deploy affordable units.
4. Attack the enemy stronghold when appropriate.
5. Preserve worthwhile future actions.
6. Claim initiative when advantageous and available.
7. Pass when no worthwhile action remains.

These are guidelines, not a requirement for an elaborate scoring engine.

Use explicit deterministic priorities and tie-breaking.

## 14.6 Important behavior

Avoid:

- Infinite pass loops.
- Repeated invalid commands.
- Always claiming initiative prematurely.
- Resourcing all affordable units.
- Pointlessly refusing to attack.
- Depending on command enumeration order without an intentional tie-breaking rule.

The AI must support complete games.

## 14.7 No advanced AI

Do not introduce:

- Minimax.
- Monte Carlo tree search.
- Multiround lookahead.
- Neural networks.
- LLM calls.
- Difficulty settings.
- Training pipelines.

---

# 15. AI Action Timing

The AI must not instantly execute a long sequence of actions under normal circumstances.

Provide three speeds:

| Speed   | Delay                |
| ------- | -------------------- |
| Normal  | 750 ms               |
| Fast    | 150 ms               |
| Instant | No intentional delay |

Normal is the default.

A simple control allows changing speed during battle.

Changes apply immediately.

Speed affects presentation timing only.

It must not change strategic decisions or game rules.

## Scheduling

Avoid blocking sleeps or synchronous delays on the main UI thread.

Schedule AI actions using Bevy-compatible timing.

AI execution must not freeze rendering or input.

Execute one AI action at a time.

Allow the UI to show the resulting state before scheduling the next action.

## Background behavior

When the browser tab or native application becomes inactive:

- Pause AI execution.
- Preserve the current match.
- Avoid accumulating overdue actions.
- Resume scheduling when the application becomes active again.

Do not run background simulations to catch up.

---

# 16. Game Feedback

Provide both contextual notifications and a persistent event log.

## 16.1 Contextual notifications

Display brief messages for meaningful events.

Examples:

- `Opponent played Infantry`
- `Your Knight attacks Infantry`
- `Infantry takes 3 damage`
- `Opponent's Infantry was destroyed`
- `Opponent claimed initiative`
- `Round 4 begins`

Use straightforward text and basic styling.

No animation system is required.

## 16.2 Persistent event log

Maintain a chronological history of meaningful events.

Include:

- Unit deployments.
- Resource payments.
- Attacks.
- Damage.
- Destruction.
- Passing.
- Initiative changes.
- Round transitions.
- Victory or defeat.

Use structured events from the rules engine.

Do not infer what happened by comparing arbitrary UI state.

## 16.3 Layout

Desktop:

- Scrollable sidebar.

Mobile:

- Collapsible log panel.

The event log must not obstruct the main battlefield.

## 16.4 Hidden information

Filter events using the human player's perspective before displaying or recording them in the presentation history.

Do not reveal opponent hand or resource card identities.

## 16.5 Log growth

Ensure the interface remains responsive in long matches.

A bounded in-memory presentation history is acceptable if documented.

Do not discard authoritative rules state or corrupt replay functionality.

---

# 17. Active Player Indicator

Provide a persistent indicator displaying the current interaction state.

Examples:

- `Your Action`
- `Opponent Acting`
- `Choose Resources`
- `Choose Starting Resources`
- `Mulligan`
- `Game Over`

Subtly highlight the active player's battlefield.

Legal controls should be enabled only when appropriate.

Avoid repetitive banners whenever actions alternate.

---

# 18. Passing and Initiative

## Pass

A persistent **Pass** button must remain accessible without scrolling, including on mobile.

Pressing Pass submits the corresponding Milestone 1A command.

## Claim Initiative

Claim Initiative is accessed through a secondary action menu.

It must be discoverable but does not require a permanently visible button.

Disable it when initiative is unavailable.

After claiming initiative:

- The player's actions end for the round.
- AI actions continue as permitted by the engine.
- The UI reflects that initiative has been claimed.

Do not implement independent pass/initiative state machines.

Use the rules engine.

---

# 19. Prototype Content

Use **RON** for card definitions and deck lists.

Do not hardcode the decks directly into Rust source.

## 19.1 Required content

Create:

- A small collection of basic unit definitions.
- Two predefined 20-card decks.
- Appropriate stronghold configuration.

The decks should have enough variety to exercise:

- Low-cost units.
- Medium-cost units.
- Higher-cost units.
- Units with different Power/Health distributions.
- Favorable and unfavorable trades.
- Stronghold attacks.
- Resource decisions.

Use simple names such as:

- Infantry.
- Archer.
- Guard.
- Knight.
- Cavalry.

These names are illustrative, not mandatory.

## 19.2 No advanced card mechanics

All prototype cards must remain compatible with Milestone 1A.

Do not add:

- Abilities.
- Keywords.
- Actions.
- Upgrades.
- Commanders.
- Affinities.
- Factions.
- Special combat rules.

## 19.3 RON parsing

Use Serde-compatible RON parsing and validation.

Validate:

- Unique definition IDs.
- Valid statistics.
- Valid costs.
- Deck card references.
- Deck lengths.
- Stronghold configuration.
- Malformed or missing data.

Errors should be actionable.

## 19.4 Native and WASM packaging

The same content must work in native and browser builds.

Account for asynchronous asset loading and browser constraints where appropriate.

Embedding small RON files at compile time is acceptable if it is the simplest reliable solution.

Do not introduce runtime network dependencies merely to load a small prototype deck.

## 19.5 No editor

Do not build a card editor, deck editor, hot-reloading system, or workshop integration.

---

# 20. Local Persistence

Implement automatic local saving with one active save slot.

## 20.1 Saving

Save after every successfully completed gameplay action or phase transition.

A saved match must preserve enough information to resume deterministically.

At minimum:

- Authoritative rules-engine snapshot.
- Content compatibility information.
- Human and AI player identities.
- Any AI-specific deterministic state.
- Match lifecycle state.

Transient UI selection does not need to be preserved.

## 20.2 Browser storage

Use browser-local storage.

Choose an appropriate mechanism for the small prototype state size.

A localStorage-based implementation is acceptable if it reliably handles the expected snapshot size.

No account or backend is required.

## 20.3 Native storage

Use an appropriate per-user application data directory.

Do not save into arbitrary project working directories.

Keep platform-specific storage behind a small interface.

## 20.4 Loading

When the application starts:

- Check for an unfinished saved match.
- Validate its format and compatibility.
- Offer **Resume Battle** if valid.
- Otherwise present the normal title screen.

## 20.5 New Battle

Starting a new battle replaces the existing unfinished save.

Require confirmation before replacing a valid active save.

## 20.6 Completed matches

Completed matches are not resumable.

Clear the active save when the match finishes.

The results screen must remain visible until the player chooses another action.

## 20.7 Error handling

Handle:

- Missing save.
- Corrupt save.
- Incompatible save version.
- Incompatible card definitions.
- Storage unavailable.
- Storage write failures.
- Browser storage quota errors.

A damaged save must not prevent starting a new battle.

Do not silently discard a valid ongoing match after an ordinary recoverable error.

## 20.8 Resume behavior

After loading:

1. Restore authoritative rules state.
2. Rebuild the player-visible presentation.
3. Clear stale interaction selections.
4. Determine which player or phase is active.
5. Resume AI scheduling when appropriate.

Ensure reloading cannot duplicate or skip an AI action.

---

# 21. Battle Lifecycle

## 21.1 Title screen

Extend the existing title screen.

Controls:

- Play.
- Resume Battle, when a valid save exists.
- About.
- Quit on supported native platforms.

Preserve existing navigation behavior.

## 21.2 Starting a battle

When the player selects Play:

1. Load and validate prototype content.
2. Generate a fresh random seed.
3. Construct the match.
4. Assign the human and AI players.
5. Initialize the rules engine.
6. Enter the mulligan flow.

The human should be Player 0 by default unless the architecture provides a compelling reason otherwise.

The starting player remains randomized by the rules engine.

## 21.3 Battle

The battle continues until a stronghold is destroyed or another valid terminal outcome occurs.

## 21.4 Results

Display:

- Victory or Defeat.
- Final round number.
- Play Again.
- Main Menu.

If the engine returns a draw, display Draw.

Do not build a statistics dashboard or battle report.

## 21.5 Play Again

Play Again starts a fresh match using:

- A fresh random seed.
- The same predefined decks.
- A new shuffle.
- A newly randomized starting player.

Do not replay the previous match's seed.

Controlled seeds remain available internally for testing.

## 21.6 Main Menu

Returning to the title screen must clean up battle-specific UI entities.

If a match remains unfinished, preserve its valid save.

---

# 22. Error Handling and Recovery

Unexpected AI or integration errors must fail gracefully.

Examples include:

- The AI generates an illegal command.
- A rules-engine command fails unexpectedly.
- Presentation synchronization fails.
- Save restoration fails.
- Required content cannot be loaded.

## Required behavior

- Stop the affected execution path.
- Display a concise error message.
- Preserve the last valid authoritative state.
- Preserve the active save where possible.
- Prevent uncontrolled retries.
- Offer Retry and Return to Main Menu where appropriate.
- Log technical diagnostics.

Do not silently change gameplay state to recover.

An AI command rejected by Milestone 1A is a defect that should be visible in diagnostics.

Do not automatically substitute an unrelated legal action merely to hide the failure.

## Recovery

Retry may repeat a failed operation only when safe.

Ensure retries cannot apply a previously successful command twice.

---

# 23. Browser and Mobile Requirements

The browser version must run through the existing WebAssembly/WebGL2 pipeline.

Preserve:

- Existing canvas-host architecture.
- Relative static asset paths.
- Subpath deployment compatibility.
- Existing startup readiness signaling.
- Existing error reporting.
- Existing GitHub Pages deployment.

## Touch support

Support:

- Tap to select.
- Tap to activate.
- Tap to inspect.
- Horizontal swipe scrolling.
- Dismissible overlays.
- Responsive buttons.

Do not require hover or right-click.

## Mobile usability

At the minimum viewport:

- The player can inspect cards.
- The player can mulligan.
- The player can resource.
- The player can play units.
- The player can attack.
- The player can pass.
- The player can claim initiative.
- The player can inspect resources.
- The player can view the event log.
- The player can finish a battle.

Prevent accidental page scrolling from interfering with intended canvas interactions where practical.

Do not implement a PWA, service worker, or offline install flow.

---

# 24. Automated Testing

This milestone requires comprehensive automated verification.

Use the repository's existing testing infrastructure whenever possible.

## 24.1 AI tests

Test:

- Mulligan decisions.
- Initial resourcing.
- Subsequent resourcing.
- Affordable deployments.
- Favorable trades.
- Direct stronghold attacks.
- Initiative claiming.
- Passing.
- Deterministic tie-breaking.
- Legal command selection.
- Complete headless AI-controlled matches.

Ensure AI simulations terminate reliably.

Use controlled fixtures and seeds.

Do not make tests dependent on wall-clock delays.

## 24.2 Integration tests

Test the actual Bevy-to-rules-engine integration.

Verify:

- Match initialization.
- Human commands.
- AI commands.
- State reconciliation.
- Event consumption.
- Unit deployment.
- Attacking.
- Damage updates.
- Round transitions.
- Game completion.
- Restart.
- UI cleanup.

Use the production gameplay systems in tests.

Avoid constructing a separate simplified implementation solely for test execution.

## 24.3 Persistence tests

Test:

- Save creation.
- Save replacement.
- Save restoration.
- Deterministic continuation.
- Pending AI turns after restoration.
- Invalid or corrupt saves.
- Incompatible versions.
- Missing storage.
- Completed-match cleanup.
- No duplicate commands after resume.

Use an injectable or otherwise testable storage abstraction.

## 24.4 Browser tests

Extend the existing Playwright infrastructure.

Test the actual compiled WASM game.

Do not replace the game with mock HTML elements.

Required browser scenarios include:

1. Launch game.
2. Select Play.
3. Complete mulligan.
4. Select starting resources.
5. Deploy a unit.
6. Observe an AI action.
7. Advance a round.
8. Attack an enemy unit.
9. Attack a stronghold.
10. Claim initiative.
11. Inspect a card.
12. Open and close the event log.
13. Change AI speed.
14. Refresh the browser.
15. Resume the saved battle.
16. Finish a match.
17. Start another battle.
18. Return to the title screen.

Use deterministic match fixtures where needed to make these tests repeatable.

## 24.5 Viewport coverage

At minimum, test:

| Viewport   | Purpose                 |
| ---------- | ----------------------- |
| 1280 × 800 | Desktop                 |
| 1280 × 720 | Short desktop           |
| 390 × 844  | Mobile portrait         |
| 375 × 667  | Minimum supported phone |
| 844 × 390  | Mobile landscape        |

Include touch emulation for mobile tests.

Test resizing and orientation changes.

## 24.6 Rendering verification

Capture actual rendered screenshots.

Inspect:

- Card text readability.
- Battlefield layout.
- Hand layout.
- Strongholds.
- Resource indicators.
- Buttons.
- Selection highlighting.
- Overflow handling.
- Event log.
- Card previews.
- Results screen.

Screenshots must come from the actual rendered application.

A passing DOM assertion or Bevy ECS test does not prove the final rendering is usable.

## 24.7 Test-support integration

The repository already has a read-only browser test bridge.

Extend it only as necessary to observe live gameplay state.

Production builds must not expose privileged state or test-only mutation capabilities.

For deterministic browser scenarios, prefer fixed test configuration, a separately controlled test harness, or production interaction paths over privileged production hooks.

## 24.8 Existing tests

All existing Milestone 1A tests must continue passing.

Do not weaken the existing test suite.

---

# 25. Verification and CI

Follow the repository's established verification workflow.

Run the applicable commands:

```bash
just fmt
just fmt-check
just check
just check-web
just smoke-native
just smoke-web
just check-all
```

Use focused checks during development.

For example:

```bash
cargo test --locked -p low-kingdoms-rules

cargo check --locked -p low-kingdoms-client

cargo clippy --locked \
  --workspace \
  --all-targets \
  -- -D warnings
```

Use the repository's actual supported target and feature combinations.

Do not indiscriminately enable all features.

## CI requirements

Ensure CI continues to verify:

- Formatting.
- Repository structure.
- Strict Clippy.
- Rust tests.
- Rustdoc.
- Native builds.
- WASM builds.
- Browser rendering.
- Input interactions.
- Existing deployment compatibility.

Extend the browser tests to cover playable gameplay.

Do not replace meaningful gameplay tests with superficial smoke checks.

## Failure handling

When CI fails:

1. Inspect the actual failing job.
2. Read its logs.
3. Identify the cause.
4. Correct the implementation.
5. Push the fix.
6. Verify the affected checks again.

Do not disable checks merely to make the PR green.

If an environment lacks a required native rendering dependency or GPU capability, report that check as blocked.

Do not label blocked checks as passing.

---

# 26. Documentation Requirements

Update:

- `docs/plans/milestone-1b.md`
- `docs/architecture.md`
- `docs/development.md`
- `docs/testing.md`
- `README.md`, where appropriate
- Relevant `AGENTS.md` files
- Relevant repository-local skills, if their instructions have become outdated

Document:

- How to launch a battle.
- How to play against the AI.
- Basic controls.
- Mouse and touch interactions.
- Responsive layout behavior.
- AI decision architecture.
- RON card and deck format.
- Content validation.
- Persistence format and location.
- Save compatibility.
- Testing instructions.
- Known limitations.

Add:

`docs/plans/milestone-1b-acceptance.md`

This should map every acceptance criterion to implementation evidence and tests.

---

# 27. Implementation Strategy

Implement the milestone in logical phases.

These are implementation phases, not separate deliverables or PRs.

## Phase 1: Specification and architecture

- Document all 33 decisions.
- Inspect the existing rules-engine API.
- Identify appropriate module boundaries.
- Update dependency and package ownership plans.
- Define integration tests.

## Phase 2: RON content

- Implement the RON loader.
- Define basic unit content.
- Define two 20-card decks.
- Validate definitions.
- Integrate content with match initialization.

## Phase 3: AI

- Implement deterministic setup decisions.
- Implement basic gameplay heuristics.
- Add headless AI tests.
- Verify complete matches.

## Phase 4: Game session integration

- Connect the client to the rules engine.
- Implement session ownership.
- Implement command dispatch.
- Implement event consumption.
- Implement authoritative-view reconciliation.
- Implement AI scheduling.

## Phase 5: Gameplay UI

- Extend title navigation.
- Implement mulligan screen.
- Implement resourcing.
- Implement battlefield.
- Implement card inspection.
- Implement deployment and targeting.
- Implement Pass and Claim Initiative.
- Implement stronghold and resource displays.
- Implement turn indicators.

## Phase 6: Responsive behavior

- Implement desktop layouts.
- Implement mobile portrait layouts.
- Implement mobile landscape layouts.
- Implement overflow scrolling.
- Implement touch interactions.
- Validate the minimum supported viewport.

## Phase 7: Feedback and lifecycle

- Implement contextual notifications.
- Implement the event log.
- Implement AI speed controls.
- Implement game-over screens.
- Implement Play Again and Main Menu.
- Implement graceful error handling.

## Phase 8: Persistence

- Implement browser-local saves.
- Implement native-local saves.
- Implement resume behavior.
- Implement save replacement.
- Implement completed-game cleanup.
- Test error conditions.

## Phase 9: Verification

- Run focused unit tests.
- Run integration tests.
- Run browser tests.
- Verify actual gameplay interactions.
- Capture and inspect screenshots.
- Run the full repository checks.
- Update documentation and acceptance records.

Do not stop at the end of an intermediate phase.

---

# 28. Acceptance Criteria

Milestone 1B is complete only when all required criteria are satisfied.

## Gameplay

| ID  | Requirement                                      |
| --- | ------------------------------------------------ |
| G01 | Player can start a match from the title screen   |
| G02 | Match uses the actual Milestone 1A engine        |
| G03 | Two valid RON decks load successfully            |
| G04 | Human can mulligan                               |
| G05 | AI can mulligan                                  |
| G06 | Human can resource cards                         |
| G07 | AI can resource cards                            |
| G08 | Human can play units                             |
| G09 | AI can play units                                |
| G10 | Human can attack units                           |
| G11 | Human can attack strongholds                     |
| G12 | AI can attack units and strongholds              |
| G13 | Passing works                                    |
| G14 | Claiming initiative works                        |
| G15 | Regroup works                                    |
| G16 | Damage is displayed correctly                    |
| G17 | Exhaustion is displayed correctly                |
| G18 | Victory and defeat work                          |
| G19 | Complete battles can finish without intervention |
| G20 | Play Again starts a fresh match                  |

## Presentation

| ID  | Requirement                             |
| --- | --------------------------------------- |
| P01 | Functional desktop battlefield          |
| P02 | Functional mobile portrait battlefield  |
| P03 | Functional mobile landscape battlefield |
| P04 | Responsive layout                       |
| P05 | Click/tap selection                     |
| P06 | Card inspection                         |
| P07 | Contextual Play and Attack controls     |
| P08 | Legal target highlighting               |
| P09 | Scrollable crowded battlefields         |
| P10 | Persistent Pass control                 |
| P11 | Secondary Claim Initiative control      |
| P12 | Resource counters                       |
| P13 | Resource inspection                     |
| P14 | Contextual notifications                |
| P15 | Persistent event log                    |
| P16 | Active-player indicator                 |
| P17 | Normal/Fast/Instant AI speed            |
| P18 | Usable 375 × 667 viewport               |

## Persistence

| ID  | Requirement                             |
| --- | --------------------------------------- |
| S01 | Automatic local saving                  |
| S02 | Browser save support                    |
| S03 | Native save support                     |
| S04 | Resume Battle                           |
| S05 | One active save                         |
| S06 | New Battle replacement confirmation     |
| S07 | Completed matches clear resumable saves |
| S08 | Corrupt saves fail gracefully           |
| S09 | Restored matches continue correctly     |
| S10 | AI actions do not duplicate on resume   |

## Architecture

| ID  | Requirement                                   |
| --- | --------------------------------------------- |
| A01 | Rules crate remains Bevy-independent          |
| A02 | UI does not duplicate gameplay rules          |
| A03 | AI uses validated rules commands              |
| A04 | AI behavior is deterministic                  |
| A05 | UI consumes structured engine events          |
| A06 | UI derives state from player-visible views    |
| A07 | Hidden information remains protected          |
| A08 | Native and WASM share gameplay implementation |
| A09 | RON content is validated                      |
| A10 | Game errors preserve valid state              |
| A11 | Screen transitions clean up entities          |
| A12 | No unnecessary architectural abstractions     |

## Verification

| ID  | Requirement                                        |
| --- | -------------------------------------------------- |
| T01 | Existing Milestone 1A tests pass                   |
| T02 | AI unit tests pass                                 |
| T03 | Complete AI match tests pass                       |
| T04 | Bevy integration tests pass                        |
| T05 | Persistence tests pass                             |
| T06 | Desktop browser interaction tests pass             |
| T07 | Mobile touch interaction tests pass                |
| T08 | Refresh/resume tests pass                          |
| T09 | Native build passes                                |
| T10 | WASM build passes                                  |
| T11 | Strict linting passes                              |
| T12 | Formatting passes                                  |
| T13 | Native rendering smoke test passes where supported |
| T14 | Actual browser rendering is inspected              |
| T15 | Full CI passes                                     |
| T16 | Documentation and acceptance records are complete  |

If any criterion remains incomplete, report it explicitly.

Do not declare the milestone complete based solely on compilation or headless tests.

---

# 29. Product Quality Requirements

Although this is a functional prototype, it must be usable.

The user should not have to understand the internal rules-engine API to play.

Avoid:

- Invisible controls.
- Tiny mobile buttons.
- Ambiguous card selection.
- Unclear active-player state.
- Sudden unexplained AI actions.
- Accidental attacks during inspection.
- Hidden offscreen information without scroll indicators.
- Text clipping.
- Inaccessible Pass controls.
- Unrecoverable errors.
- Silent loss of unfinished matches.

The interface should explain itself through conventional CCG interactions and clear feedback.

Do not substitute excessive instructional text for functional UI.

---

# 30. Performance Requirements

The game should feel responsive on a modern phone and desktop browser.

Avoid:

- Blocking the main thread while AI actions are scheduled.
- Reconstructing the entire UI tree every frame.
- Unnecessary per-frame allocations.
- Repeatedly parsing RON during gameplay.
- Repeated save writes every frame.
- Duplicated timers or observers.
- Unbounded entity accumulation.
- Expensive layout recalculation when nothing changed.

A one-time UI rebuild after resuming a match is acceptable.

A complete presentation rewrite every frame is not.

Keep performance instrumentation lightweight.

No dedicated optimization system or telemetry service is required.

---

# 31. Final Manual Validation

Before completing the task, perform an actual playable-session review.

### Desktop

Launch the native or browser application.

Verify that you can:

1. Select Play.
2. Mulligan or keep.
3. Resource cards.
4. Play a unit.
5. Observe the AI.
6. Advance rounds.
7. Attack a unit.
8. Attack a stronghold.
9. Claim initiative.
10. Win or lose.
11. Play Again.
12. Return to Main Menu.

### Mobile

Using an actual browser viewport with touch emulation:

1. Start a match.
2. Open and close the hand tray.
3. Inspect a card.
4. Play a card.
5. Select an attacker.
6. Scroll to an offscreen target.
7. Attack that target.
8. Open the event log.
9. Access Pass and Claim Initiative.
10. Change orientation.
11. Refresh the browser.
12. Resume the match.

Inspect the screenshots and correct obvious clipping, unreadable text, or inaccessible controls.

Do not consider a successful screenshot capture equivalent to inspecting what the screenshot contains.

---

# 32. GitHub Workflow

Use a dedicated implementation branch.

Suggested branch:

`feat/milestone-1b-playable-battle`

Base it on the current `main` branch containing Milestone 1A.

Do not modify or overwrite an existing implementation branch with unrelated work.

Commit changes logically.

Suggested commits:

1. Milestone 1B specification and architecture.
2. RON content and loading.
3. Deterministic heuristic AI.
4. Gameplay session integration.
5. Battlefield and card interactions.
6. Responsive mobile UI.
7. Feedback and game lifecycle.
8. Persistence.
9. Tests, verification, and documentation.

These are suggestions; use fewer or more commits where appropriate.

## Pull request

Open one PR targeting `main`.

Suggested title:

**feat: implement Milestone 1B playable battle**

The PR must include:

- Summary.
- Architecture changes.
- AI implementation.
- RON content format.
- Gameplay UI.
- Mobile behavior.
- Persistence.
- Tests.
- Verification results.
- Screenshots or evidence paths.
- Known limitations.
- Acceptance-criteria status.

Do not merge the PR.

## CI

Inspect the actual GitHub Actions runs for the PR.

Fix failures resulting from the implementation.

Do not assert success before the checks complete.

---

# 33. Final Deliverable

When implementation is complete, provide a final report containing:

### 1. Pull request

Link directly to the PR.

### 2. What was implemented

Summarize the actual features delivered.

### 3. Architecture

Explain:

- Bevy integration.
- Rules-engine boundary.
- AI.
- Content loading.
- Persistence.
- UI synchronization.

### 4. Verification

Provide a table of checks with accurate results:

- Passed.
- Failed.
- Blocked.
- Not run.

Include the relevant operating system, target, or browser.

### 5. Visual evidence

Provide paths or links to actual native and browser screenshots, including mobile portrait.

### 6. Known limitations

Distinguish intentional Milestone 1B exclusions from incomplete requirements or defects.

### 7. Milestone status

State whether Milestone 1B is:

- Complete.
- Incomplete.
- Blocked.

Support that assessment with concrete evidence.

---

# 34. Completion Standard

**Milestone 1B is not complete until a player can actually play Low Kingdoms against a deterministic heuristic AI, through the real Bevy interface, from start to finish.**

A successful implementation must work across the existing native and WASM targets, provide a usable responsive interface, preserve unfinished games, and pass applicable automated verification.

Do not stop after producing mockups, specifications, architecture diagrams, or partial gameplay systems.

Do not substitute hardcoded gameplay behavior for the Milestone 1A engine.

Do not introduce additional card mechanics or advanced systems.

Prioritize correctness, usability, maintainability, and getting to a genuinely playable battle.

**Complete the implementation, open the PR, verify CI, and report the actual results. Do not merge.**

---

This approved specification and its implementation belong on the same Milestone 1B branch.
