# Low Kingdoms — Milestone 1A: Headless Rules Engine

**Technical and gameplay specification**

| Field                | Value                                              |
| -------------------- | -------------------------------------------------- |
| Project              | Low Kingdoms                                       |
| Milestone            | 1A — Headless Rules Engine                         |
| Status               | Specification complete; implementation not started |
| Repository           | https://github.com/matanlurey/low-kingdoms         |
| Language             | Rust                                               |
| Rendering dependency | None                                               |
| Primary deliverable  | crates/rules                                       |
| Next milestone       | 1B — First Playable Battle                         |

## 1. Executive summary

Milestone 1A introduces the foundational rules engine for Low Kingdoms, a turn-based collectible card game inspired mechanically by Star Wars: Unlimited.

The engine must support a complete, deterministic, two-player battle using basic unit cards, resource management, alternating actions, and direct combat.

The engine must operate independently of Bevy, rendering, networking, persistence infrastructure, or artificial intelligence.

Its primary responsibilities are:

1. Initialize matches from predefined decks.
2. Manage game phases, rounds, and player actions.
3. Validate player commands.
4. Execute legal commands deterministically.
5. Maintain authoritative game state.
6. Resolve combat and resource interactions.
7. Enforce victory and defeat conditions.
8. Produce structured events describing state changes.
9. Provide player-specific views that respect hidden information.
10. Support headless testing and deterministic replay.

The engine should establish a stable foundation for future mechanics without prematurely implementing them.

**Milestone 1A is complete when two players can finish an entire game through a headless API without requiring any UI or player-facing presentation.**

### 1.1 Design principles

**Correctness over flexibility.** Implement the rules defined in this specification. Do not introduce generalized systems for mechanics that do not yet exist.

**Determinism.** Given identical initialization parameters and the same sequence of valid player commands, the engine must produce identical states and events. This must hold across supported native platforms and WebAssembly.

**Explicit decisions.** The engine must not make strategic decisions for players. Player decisions must be represented as explicit commands. Automatic operations are permitted when their behavior is entirely determined by the rules, such as exhausting resources to pay a cost and calculating combat damage.

**Renderer independence.** The rules engine must not depend on Bevy, rendering or windowing libraries, UI frameworks, operating-system input, network services, or wall-clock time.

**Separation of definitions and instances.** Static card definitions must be distinct from the individual card instances participating in a match.

**Extensibility without speculation.** The architecture should support future mechanics, but Milestone 1A must not implement placeholder ability systems, effects frameworks, or speculative abstractions.

## 2. Scope

### 2.1 Included

| System                       | Milestone 1A |
| ---------------------------- | ------------ |
| Two-player matches           | Required     |
| Match initialization         | Required     |
| Seeded randomness            | Required     |
| Starting-player override     | Required     |
| Deck shuffling               | Required     |
| Opening hands                | Required     |
| Mulligans                    | Required     |
| Resource zones               | Required     |
| Resource spending            | Required     |
| Alternating actions          | Required     |
| Passing                      | Required     |
| Initiative claiming          | Required     |
| Round progression            | Required     |
| Unit deployment              | Required     |
| Unit exhaustion              | Required     |
| Unit combat                  | Required     |
| Stronghold combat            | Required     |
| Persistent damage            | Required     |
| Unit destruction             | Required     |
| Discard piles                | Required     |
| Deck exhaustion              | Required     |
| Victory and defeat           | Required     |
| Hidden information           | Required     |
| Command validation           | Required     |
| Structured game events       | Required     |
| State serialization          | Required     |
| Deterministic replay support | Required     |
| Automated testing            | Required     |

### 2.2 Explicitly excluded

The following systems must not be implemented during Milestone 1A:

| System                        | Planned for later |
| ----------------------------- | ----------------- |
| Commander deployment          | Yes               |
| Commander abilities           | Yes               |
| Affinities                    | Yes               |
| Factions                      | Yes               |
| Action cards                  | Yes               |
| Upgrades                      | Yes               |
| Keywords                      | Yes               |
| Triggered abilities           | Yes               |
| Persistent effects            | Yes               |
| Temporary stat modifiers      | Yes               |
| Healing                       | Yes               |
| Status effects                | Yes               |
| Combat interception           | Yes               |
| Unique-card restrictions      | Yes               |
| Board capacity limits         | Yes               |
| Deck building                 | Yes               |
| Campaign progression          | Yes               |
| Card rewards                  | Yes               |
| AI opponents                  | Yes               |
| Automated balance simulations | Yes               |
| Multiplayer networking        | Yes               |
| Game UI                       | Milestone 1B      |
| Card artwork and animation    | Later             |
| Account or save-game services | Later             |

Strongholds are included because they are necessary for victory and defeat. They have Health but no abilities or other special mechanics during Milestone 1A.

**Scope constraint:** Do not build a general-purpose CCG engine. Build the smallest maintainable implementation of the specific Low Kingdoms rules described here.

## 3. Terminology

| Term             | Definition                                                             |
| ---------------- | ---------------------------------------------------------------------- |
| Match            | A complete game between two players                                    |
| Player           | One of two participants                                                |
| Round            | A complete cycle of regrouping and alternating actions                 |
| Action           | A gameplay operation taken during the action phase                     |
| Command          | An external request to the engine, including setup decisions           |
| Initiative       | Determines which player acts first in a round                          |
| Claim Initiative | An action that secures initiative for the next round                   |
| Pass             | An action that temporarily declines to act                             |
| Ready            | A unit or resource that is available for use                           |
| Exhausted        | A unit or resource that cannot be used for actions requiring readiness |
| Resource         | A card placed facedown in the resource zone                            |
| Power            | Damage a unit deals in combat                                          |
| Health           | Maximum damage a unit can sustain before destruction                   |
| Damage           | Accumulated damage on a unit or stronghold                             |
| Stronghold       | The player's primary objective; its destruction causes defeat          |
| Definition       | Static data describing a card                                          |
| Instance         | An individual occurrence of a card within a match                      |
| Zone             | A location containing cards or units                                   |
| Event            | An immutable record describing something that happened                 |

The terms **MUST**, **SHOULD**, and **MAY** express implementation requirements, recommendations, and optional behavior.

## 4. Match configuration

A match consists of exactly two players. For Milestone 1A, these players can be identified using stable identifiers such as PlayerId(0) and PlayerId(1).

### 4.1 Required configuration

A match must be initialized with:

- Two valid players.
- One deck per player.
- One stronghold per player.
- A random seed.
- An optional starting-player override.
- A registry containing all referenced card definitions.

The engine must reject invalid match configurations with structured errors.

### 4.2 Starting player

By default, the engine randomly selects the starting player using the supplied seed. The caller may override this selection. The selected player receives initiative for the opening round.

The same configuration, seed, and override must always produce the same initialization result.

### 4.3 Deck construction

The engine receives predefined decks. Deck construction and editing are not part of Milestone 1A.

The engine must support duplicate card definitions within a deck. A normal match configuration must contain enough cards to draw a six-card opening hand.

**Implementation default:** Require at least six cards per deck.

There is no engine-enforced maximum deck size during Milestone 1A. The anticipated first playable prototype uses 20-card decks, but 20 is not a hard-coded engine restriction.

### 4.4 Strongholds

Each player begins with a stronghold. For Milestone 1A:

- Strongholds have maximum Health.
- Strongholds track accumulated damage.
- Strongholds can be attacked directly.
- Strongholds do not retaliate.
- Strongholds do not become exhausted.
- Strongholds have no abilities.
- Strongholds do not occupy the unit battlefield.

The initial maximum Health must be provided by the match configuration rather than hard-coded into the rules engine. A stronghold's Health must be greater than zero.

## 5. Game state

The engine maintains one authoritative GameState. The state must contain everything required to determine the current game position and execute the next command. It must not require external mutable state to resolve gameplay.

### 5.1 Match state

At minimum:

| Field                  | Purpose                                        |
| ---------------------- | ---------------------------------------------- |
| Match status           | Setup, active, or finished                     |
| Current phase          | Mulligan, resourcing, regroup, or action       |
| Round number           | Current round, beginning at 1                  |
| Initiative holder      | Player with current-round initiative           |
| Next initiative holder | Player designated to start the next round      |
| Active player          | Player whose action is currently expected      |
| Consecutive passes     | Tracks action-phase termination                |
| Initiative claimed     | Whether initiative has been claimed this round |
| Player states          | State belonging to each player                 |
| Pending decisions      | Submitted decisions awaiting the other player  |
| RNG state              | Deterministic random generator state           |
| Next instance ID       | Identifier allocation state                    |
| Final result           | Winner, draw, or unfinished                    |

The precise internal Rust representation is an implementation detail.

### 5.2 Player state

Each player maintains:

- Deck.
- Hand.
- Resource zone.
- Battlefield.
- Discard pile.
- Stronghold.
- Setup and regroup decision status.

Player state must be logically isolated. A player's commands cannot directly manipulate another player's private zones.

### 5.3 Card zones

**Deck.** An ordered collection of card instances. The order is hidden from both players. Drawing removes cards from the top of the deck.

**Hand.** An ordered collection of card instances. A player can inspect their own hand. The opponent can see only the hand's size. There is no hand-size limit.

**Resource zone.** A collection of facedown card instances. Each resource tracks whether it is ready or exhausted. Players can inspect their own resources. Opponents can inspect resource counts and readiness but not card identities.

**Battlefield.** Contains units currently in play. Units have public identities, statistics, damage, and readiness. There is no battlefield capacity limit.

**Discard pile.** Contains cards that have been destroyed or otherwise discarded. Discard piles are public. Cards retain their definition and instance identities. For Milestone 1A, unit destruction is the only gameplay operation that moves cards into the discard pile.

### 5.4 Card conservation

Every card instance originating in a deck must exist in exactly one card zone at any point in time.

Moving a card between zones must preserve its instance identity. No normal gameplay operation may accidentally duplicate or delete a card instance.

Strongholds are tracked separately from deck cards.

## 6. Card definitions and instances

Milestone 1A requires only basic unit cards. No actual card library or card balancing is part of this specification.

### 6.1 Card definitions

A unit card definition requires:

| Property      | Type                | Constraint             |
| ------------- | ------------------- | ---------------------- |
| Definition ID | Stable identifier   | Unique within registry |
| Cost          | Nonnegative integer | >= 0                   |
| Power         | Nonnegative integer | >= 0                   |
| Health        | Positive integer    | > 0                    |

Display names or descriptive metadata may be included if useful, but they must not influence rules resolution. No card text or abilities are supported.

### 6.2 Card instances

Each physical occurrence of a definition in a match has its own instance ID.

Two copies of the same unit definition have the same definition ID, different instance IDs, independent damage, and independent exhaustion state.

Instance IDs must remain stable throughout a match and must not be reused after a card is destroyed.

### 6.3 Unit state

A unit on the battlefield maintains:

- Instance ID.
- Definition ID.
- Owner.
- Base Power.
- Base Health.
- Accumulated damage.
- Ready or exhausted state.

Remaining Health is calculated as:

> Remaining Health = Base Health − Accumulated Damage

A unit is destroyed when:

> Accumulated Damage >= Base Health

The base statistics do not change when damage is applied.

### 6.4 Future compatibility

Future modifiers may adjust effective Power and Health. Milestone 1A must not implement a modifier system.

However, base statistics and accumulated damage must remain distinct concepts so that modifiers can be introduced without redesigning combat damage tracking.

## 7. Game initialization

Game initialization follows a deterministic sequence.

### 7.1 Create match

The caller supplies a valid match configuration. The engine:

1. Validates all referenced card definitions.
2. Creates card instances.
3. Initializes strongholds.
4. Initializes its seeded RNG.
5. Determines the starting player.
6. Shuffles both decks.
7. Draws six cards for each player.
8. Enters the mulligan phase.

For reproducibility, random operations must occur in a documented, fixed order. Player IDs, not hash-map iteration order or command arrival timing, determine any required ordering.

### 7.2 Opening hands

Each player draws exactly six cards from the top of their shuffled deck.

Players can inspect their own opening hands. The engine must not reveal the opposing player's card identities.

### 7.3 Mulligans

Each player may perform one mulligan.

Available decisions:

- Keep the current hand.
- Mulligan the current hand.

A mulligan:

1. Returns all six cards to the player's deck.
2. Shuffles that deck.
3. Draws six replacement cards.

The player cannot perform another mulligan. Both players must complete their mulligan decisions before initial resourcing begins.

**Decision visibility:** Mulligan decisions should be submitted independently. The engine must not allow one player's choice to change the other player's legal options. To avoid input-order effects, decisions should be buffered and resolved in a fixed player order once both have been submitted.

### 7.4 Initial resourcing

After mulligans, each player may select up to two cards from their hand to become resources. Selecting zero cards is legal.

Selected cards:

- Move from hand to resource zone.
- Retain their original identities.
- Become facedown.
- Initially enter exhausted.

The two players complete their decisions independently. Once both have completed initial resourcing, all starting resources become ready.

### 7.5 Begin first round

The first round begins immediately after initial resourcing.

The player holding initiative receives the first action.

**The first round does not include an additional draw or normal regroup phase.**

## 8. Round structure

A normal round consists of:

1. Regroup phase.
2. Action phase.
3. Round completion.

The first round starts directly in the action phase after setup.

### 8.1 Regroup phase

Starting with round two, regroup executes the following operations in order.

**Step 1: Draw cards.** Both players draw two cards. Deck exhaustion penalties apply when necessary. If drawing causes the game to end, remaining regroup operations are skipped.

**Step 2: Resource cards.** Each player may select up to one card from their hand to become a resource. Selecting no card is legal. The selected card moves to the resource zone exhausted. Both players must finish their selections before the phase advances.

**Step 3: Ready cards.** All exhausted units and resources become ready, including resources placed during the current regroup phase.

**Step 4: Begin action phase.** The initiative holder becomes the active player. The consecutive-pass count and initiative-claimed status reset for the new round.

### 8.2 Action phase

Players alternate taking one action at a time.

| Action           | Effect                               |
| ---------------- | ------------------------------------ |
| Play Unit        | Deploy a unit from hand              |
| Attack           | Attack a unit or stronghold          |
| Pass             | Temporarily decline to act           |
| Claim Initiative | Secure initiative for the next round |

An action normally transfers the opportunity to act to the opponent. Exceptions apply after an opponent has claimed initiative.

### 8.3 Round completion

The action phase ends when:

- Both players pass consecutively; or
- One player has claimed initiative and the remaining player passes.

The engine then advances to the next round's regroup phase, unless the match has ended. There is no fixed maximum number of rounds.

## 9. Action economy and initiative

### 9.1 Alternating actions

During the action phase, exactly one player is normally eligible to act.

That player may execute one legal action. Control then passes to the opponent. An action must complete its effects before the next action begins. There are no interrupts or reaction windows.

### 9.2 Passing

Passing consumes the player's current action opportunity. Passing does not prevent that player from acting again later in the same round.

A pass followed by an opponent's non-pass action does not terminate the round. Two consecutive passes terminate the action phase.

| Step | Player | Action    | Result          |
| ---- | ------ | --------- | --------------- |
| 1    | A      | Pass      | B acts          |
| 2    | B      | Play Unit | A may act again |
| 3    | A      | Pass      | B acts          |
| 4    | B      | Pass      | Round ends      |

### 9.3 Claiming initiative

A player may spend an action to claim initiative.

Claiming initiative:

- Secures initiative for the next round.
- Permanently ends that player's actions for the current round.
- Prevents the opponent from claiming initiative during the same round.

The remaining player may continue taking actions.

| Step | Player | Action           |
| ---- | ------ | ---------------- |
| 1    | A      | Play Unit        |
| 2    | B      | Attack           |
| 3    | A      | Claim Initiative |
| 4    | B      | Play Unit        |
| 5    | B      | Attack           |
| 6    | B      | Pass             |
| 7    | —      | Round ends       |

Player A acts first in the next round.

### 9.4 Unclaimed initiative

If neither player claims initiative, the current initiative holder retains initiative for the next round.

### 9.5 Initiative restrictions

Only one player may claim initiative in a round.

A player who has already claimed initiative may not attack, play units, pass, or claim initiative again.

The remaining player continues acting until they pass or the match ends.

### 9.6 Consecutive passes

Any non-pass action breaks the sequence of consecutive passes. Claiming initiative is not a pass.

However, because the claiming player becomes ineligible to act, the remaining player's eventual pass ends the round.

The engine must distinguish these conditions explicitly.

## 10. Resource system

Low Kingdoms uses a card-based resource system modeled after Star Wars: Unlimited.

### 10.1 Resourcing

Cards in a player's hand may become resources during designated resourcing phases.

Resources are facedown and retain their original card identities. Resources do not contribute their printed Power, Health, or Cost while in the resource zone.

For Milestone 1A, all resources are functionally interchangeable.

### 10.2 Resource costs

Each playable unit card has an integer cost greater than or equal to zero. A player may play a unit only when they control at least that many ready resources.

### 10.3 Paying costs

Playing a unit exhausts exactly the required number of ready resources.

Example: A player has five resources (four ready, one exhausted) and plays a unit costing three resources. After payment, one resource remains ready and four resources are exhausted.

### 10.4 Automatic resource selection

The engine automatically chooses which resources to exhaust. Players do not manually select payment resources.

The selection must be deterministic.

**Implementation default:** Exhaust eligible resources in ascending instance-ID order. This is an internal implementation rule, not a strategic game mechanic.

### 10.5 Zero-cost cards

A zero-cost unit can be played without exhausting resources. Playing it still consumes an action.

### 10.6 Resource readiness

Resources become ready during regroup. Previously exhausted resources remain exhausted until then. There is no automatic resource recovery during the action phase.

### 10.7 No resource pools

Milestone 1A must not implement floating resource points or temporary resource pools. Resources are exhausted directly when paying costs.

### 10.8 No overpayment

Players may not voluntarily exhaust additional resources beyond a card's required cost.

## 11. Playing units

### 11.1 Requirements

A player may play a unit when:

1. The game is in the action phase.
2. It is that player's opportunity to act.
3. The card exists in their hand.
4. The card is a supported unit card.
5. They have enough ready resources.
6. They have not claimed initiative this round.

### 11.2 Resolution

Playing a unit:

1. Validates the command.
2. Automatically exhausts the required resources.
3. Removes the card from hand.
4. Places the unit on the player's battlefield.
5. Marks the unit exhausted.
6. Emits the appropriate events.
7. Transfers the action opportunity to the opponent.

All operations form a single atomic command.

### 11.3 Unit readiness

Units enter play exhausted. Consequently, units cannot normally attack during the round in which they are played. Units become ready during the next regroup phase.

### 11.4 Multiple deployments

There is no limit on the number of units a player may deploy during a round. Each deployment consumes a separate action. Resource availability limits how many cards can be played.

### 11.5 Duplicate units

Multiple copies of the same unit definition may coexist on the battlefield. Each has independent state. No uniqueness rules apply.

### 11.6 Battlefield limits

There is no maximum number of units on either battlefield.

## 12. Combat

Combat consists of one unit attacking an enemy unit or the opposing stronghold.

The attacking player chooses the target. There are no blockers, interrupts, or defender-selected targets.

### 12.1 Attack requirements

An attack is legal when:

- The game is in the action phase.
- It is the attacking player's action.
- The attacker is a unit controlled by that player.
- The attacker exists on the battlefield.
- The attacker is ready.
- The target is a valid enemy unit or enemy stronghold.
- The attacking player has not claimed initiative.

### 12.2 Target selection

A unit may attack any enemy unit or the opposing stronghold.

Enemy units do not automatically protect the stronghold. An enemy unit may be attacked regardless of whether it is ready or exhausted.

There are no targeting restrictions based on Power, Health, or unit identity.

### 12.3 Attacking a unit

When attacking another unit:

1. Validate the attack.
2. Exhaust the attacking unit.
3. Determine both units' Power.
4. Apply combat damage simultaneously.
5. Destroy any units with lethal damage.
6. Resolve any resulting game-state changes.
7. Transfer the action opportunity.

**Simultaneous damage:** The attacking unit deals damage equal to its Power. The defending unit deals damage equal to its Power. Both damage amounts are determined before either unit is destroyed.

Example:

|                     |                       Attacker |  Defender |
| ------------------- | -----------------------------: | --------: |
| Power               |                              3 |         2 |
| Health              |                              4 |         3 |
| Existing damage     |                              1 |         0 |
| Damage after combat |                              3 |         3 |
| Outcome             | Survives at 1 remaining Health | Destroyed |

### 12.4 Mutual destruction

Both units may be destroyed during the same combat.

Example: Attacker is 3 Power / 2 Health; defender is 2 Power / 3 Health. Both units deal lethal damage and move to their respective discard piles.

The engine must not process the attacker's damage first in a way that prevents the defender from retaliating.

### 12.5 Exhausted defenders

An exhausted unit may defend against an attack. Defending does not require readiness and does not change the defender's readiness state.

### 12.6 Attacking a stronghold

When attacking a stronghold:

1. Validate the attack.
2. Exhaust the attacking unit.
3. Apply damage equal to the attacker's Power to the stronghold.
4. Check whether the stronghold has been destroyed.
5. End the match if appropriate.

Strongholds do not retaliate. The attacking unit receives no damage from the stronghold.

### 12.7 Persistent damage

Damage does not disappear at the end of combat or during regroup.

Units and strongholds retain accumulated damage until destroyed or until a future mechanic explicitly modifies it.

Healing is not implemented in Milestone 1A.

### 12.8 Destruction

A unit is destroyed when its accumulated damage equals or exceeds its Health.

Destroying a unit:

- Removes it from the battlefield.
- Moves the same card instance to its owner's discard pile.
- Preserves its identity.
- Removes its battlefield-specific damage and readiness state.

Destroyed units do not return automatically.

### 12.9 Combat restrictions

Milestone 1A does not support multiple attackers in one attack, multiple defenders, blocking, interception, first strike, retaliation suppression, damage prevention, armor, shields, combat keywords, combat triggers, attack modifiers, or attacking friendly units.

## 13. Drawing and deck exhaustion

### 13.1 Drawing cards

Drawing removes cards from the top of a player's deck and places them into their hand. The order of the deck determines which cards are drawn.

Drawing itself does not consume an action during setup or regroup.

### 13.2 Normal round draws

At the beginning of each round after the first, both players draw two cards.

### 13.3 Empty decks

When a player must draw more cards than remain in their deck:

- They draw as many cards as possible.
- Each missing card causes 3 damage to their stronghold.
- The game continues unless this damage destroys a stronghold.

Examples:

- Need 2 cards, deck has 1: draw 1; stronghold takes 3 damage.
- Need 2 cards, deck has 0: draw 0; stronghold takes 6 damage.

### 13.4 No automatic defeat

Having zero cards in the deck does not itself cause defeat. Defeat occurs when the stronghold is destroyed.

### 13.5 Simultaneous regroup draws

Both players' mandatory regroup draws are treated as occurring during the same phase step.

**Implementation default:** Resolve both players' draws and deck-exhaustion damage as a batch before determining the outcome.

This prevents player iteration order from determining the winner when both strongholds are destroyed by deck exhaustion.

If both strongholds are destroyed by the same mandatory draw step, the game ends in a draw.

## 14. Victory and defeat

### 14.1 Victory condition

A player wins when the opposing stronghold is destroyed.

A stronghold is destroyed when:

> Accumulated Damage >= Maximum Health

### 14.2 Immediate termination

When a stronghold is destroyed, the engine must immediately transition to a terminal match state, except when completing the simultaneous regroup draw batch described above.

No additional player actions may occur after the match ends.

### 14.3 Match outcomes

| Outcome       | Condition                                                          |
| ------------- | ------------------------------------------------------------------ |
| Player 0 wins | Player 1's stronghold is destroyed                                 |
| Player 1 wins | Player 0's stronghold is destroyed                                 |
| Draw          | Both strongholds are destroyed in the same simultaneous resolution |
| In progress   | Neither victory condition has occurred                             |

Draws are supported for correctness even though the initial combat mechanics are unlikely to produce them outside simultaneous deck exhaustion.

### 14.4 No alternate victory conditions

Milestone 1A does not support victory points, turn limits, special card-based victories, concessions, alternate win conditions, or campaign-specific objectives.

## 15. Command-driven architecture

The engine operates through explicit commands submitted by an external caller. The caller is responsible for choosing what a player does; the engine validates and executes commands.

### 15.1 Core conceptual API

The exact Rust API may differ, but the architecture should resemble:

~~~rust
let mut game = Game::new(config)?;

let result = game.apply(player_id, command)?;

let view = game.view_for(player_id);
~~~

The result of applying a command should contain structured events and enough information for the caller to determine the next required interaction.

The API must not require rendering or operating-system services.

### 15.2 Commands

Commands are organized by phase.

**Mulligan commands:**

~~~rust
KeepHand
MulliganHand
~~~

**Initial resource command:**

~~~rust
ChooseInitialResources {
    cards: Vec<CardInstanceId>,
}
~~~

An empty collection means the player chooses no resources.

**Regroup resource command:**

~~~rust
ChooseRegroupResource {
    card: Option<CardInstanceId>,
}
~~~

None means the player chooses not to resource a card.

**Action-phase commands:**

~~~rust
PlayUnit {
    card: CardInstanceId,
}

Attack {
    attacker: CardInstanceId,
    target: AttackTarget,
}

Pass

ClaimInitiative
~~~

Attack targets must explicitly distinguish units from strongholds:

~~~rust
enum AttackTarget {
    Unit(CardInstanceId),
    Stronghold(PlayerId),
}
~~~

These examples describe the intended interface, not mandatory Rust declarations.

### 15.3 No strategic automation

The engine must not automatically choose cards to resource, decide whether to mulligan, select units to play, select attackers, select attack targets, pass, or claim initiative.

The engine may automatically shuffle decks, draw mandatory cards, exhaust resources to pay costs, calculate combat damage, destroy units, ready units and resources, advance phases when completion conditions are met, and determine victory and defeat.

The distinction is whether an operation requires player judgment.

### 15.4 Atomic commands

A command must either:

1. Succeed and apply all required state changes; or
2. Fail and leave the complete game state unchanged.

Partial execution is prohibited. This includes preserving RNG state, pending decisions, event sequence, and instance-ID allocation when an action fails.

### 15.5 Command validation

The engine must reject:

- Commands from invalid players.
- Commands submitted in the wrong phase.
- Out-of-turn actions.
- Playing cards not owned by the player.
- Playing cards not in hand.
- Playing cards without sufficient resources.
- Attacking with an exhausted unit.
- Attacking with a nonexistent unit.
- Attacking an invalid target.
- Selecting nonexistent cards as resources.
- Selecting duplicate instance IDs.
- Selecting too many resource cards.
- Multiple mulligans.
- Multiple resourcing decisions in one phase.
- Actions after claiming initiative.
- Claims after initiative has already been claimed.
- Any command after the match has ended.

Errors must be structured and actionable. An invalid command must never crash the engine.

## 16. State machine

The match lifecycle must be explicit.

~~~text
Match Created
      |
      v
  Initialize
      |
      v
 Mulligan Phase
      |
      v
Initial Resourcing
      |
      v
 Action Phase <--------------------+
      |                            |
      | Round complete             |
      v                            |
 Regroup Phase                     |
      |                            |
      | Draw cards                 |
      | Resource cards             |
      | Ready cards                |
      +----------------------------+

At any point where a stronghold is destroyed:
      |
      v
 Game Finished
~~~

### 16.1 Phase progression

The engine must know which commands are legal in the current phase, which players may submit commands, whether additional decisions are required, and whether the phase can advance automatically.

### 16.2 Multi-player setup decisions

Both players must submit mulligan decisions, initial resource decisions, and regroup resource decisions.

These decisions are not alternating combat actions.

**Implementation default:** Either player may submit their decision first, but the engine buffers decisions and resolves them in fixed player order once both submissions are present.

This ensures arrival order does not affect game results.

### 16.3 Automatic phase transitions

The engine should advance automatically when no player decision remains.

For example, when both players have submitted regroup resource decisions:

1. Commit resource selections.
2. Ready units and resources.
3. Enter the action phase.
4. Set the active player to the initiative holder.

No separate EndRegroup command should be necessary.

### 16.4 Terminal states

A finished game is immutable through the command API. The game may still be inspected or serialized. All subsequent gameplay commands must fail.

## 17. Game events

The engine must emit structured events describing successful operations.

Events allow future presentation layers, tests, replays, and diagnostics to observe what happened without reconstructing changes by comparing entire states.

### 17.1 Event requirements

Events must be typed, immutable, deterministically ordered, independent of rendering, sufficient to explain meaningful game-state changes, and compatible with serialization.

### 17.2 Event categories

| Category   | Example events                                       |
| ---------- | ---------------------------------------------------- |
| Setup      | MatchInitialized, StartingPlayerSelected             |
| Deck       | DeckShuffled, CardsDrawn                             |
| Mulligan   | MulliganCompleted                                    |
| Resources  | CardsResourced, ResourcesExhausted, ResourcesReadied |
| Round      | RoundStarted, RoundEnded                             |
| Actions    | PlayerPassed, InitiativeClaimed                      |
| Deployment | UnitPlayed                                           |
| Combat     | AttackDeclared, DamageDealt, UnitDestroyed           |
| Fatigue    | DeckExhaustionDamage                                 |
| Victory    | GameEnded                                            |

Exact names are not prescribed. Do not introduce an event hierarchy larger than required.

### 17.3 Event ordering

Events must appear in a stable, documented order.

For example, playing a unit should produce events corresponding to resource payment, unit deployment, then action completion.

Combat should produce events corresponding to attack declaration, attacker exhaustion, damage assignment, unit destruction, then match termination (if applicable).

Simultaneous damage should be represented without implying that one unit's death prevented the other's damage.

Events describing simultaneous unit destruction must use a deterministic ordering, such as ascending instance ID.

### 17.4 Public and private events

Some events contain hidden information.

For example, drawing a card reveals its identity only to its owner.

The engine must support player-specific event visibility.

A public draw event may reveal which player drew and how many cards were drawn. The owning player's private event may additionally reveal card identities.

Private information must not leak through opponent-facing events.

### 17.5 Events versus state

Events describe transitions. The authoritative current state remains the source of truth.

The engine must not require consumers to replay the event stream merely to determine the current legal state.

## 18. Hidden information

Hidden information is part of the rules engine's responsibility. It must not be left entirely to the future UI.

### 18.1 Visibility matrix

| Information        | Owner                | Opponent             |
| ------------------ | -------------------- | -------------------- |
| Own hand           | Full                 | Count only           |
| Opponent hand      | Count only           | Full                 |
| Own deck order     | Hidden               | Hidden               |
| Deck size          | Visible              | Visible              |
| Own resources      | Full                 | Count/readiness only |
| Opponent resources | Count/readiness only | Full                 |
| Battlefield units  | Full                 | Full                 |
| Unit statistics    | Full                 | Full                 |
| Unit damage        | Full                 | Full                 |
| Discard piles      | Full                 | Full                 |
| Stronghold Health  | Full                 | Full                 |
| Initiative         | Full                 | Full                 |

### 18.2 Player-specific views

The engine must expose a safe method for obtaining a player's perspective, conceptually:

~~~rust
game.view_for(player_id)
~~~

The result must not reveal information the player is not entitled to know.

Private card identities must be omitted, not merely marked as hidden while retaining identifying metadata.

### 18.3 Authoritative state

The engine itself retains all information necessary to resolve the game.

A privileged internal state or diagnostic snapshot may expose complete information for testing. Such access must be clearly separated from player-facing views.

### 18.4 Future multiplayer compatibility

Milestone 1A does not implement networking or authentication. However, its visibility model should be suitable for future authoritative multiplayer execution.

The caller is trusted to provide the correct acting player ID; verifying a real user's identity is outside scope.

## 19. Determinism and randomness

Determinism is a core requirement.

### 19.1 Seeded random generator

The engine must use an explicitly seeded pseudo-random number generator.

It must not rely on system time, operating-system randomness during gameplay, global mutable randomness, unspecified iteration ordering, or renderer state.

### 19.2 Random operations

The following operations may consume randomness:

- Initial starting-player selection.
- Initial deck shuffling.
- Mulligan shuffling.

No other random mechanics exist during Milestone 1A.

### 19.3 Reproducibility

Given identical match configuration, card definitions, seed, starting-player override, and command sequence, the engine must produce identical card ordering, starting player, state transitions, events, and final result.

This must work across supported platforms running the same rules-engine version.

### 19.4 Stable ordering

The engine must not allow hash-map iteration order to influence resource payment, event ordering, shuffling, instance-ID assignment, phase progression, or combat resolution.

Use explicit canonical ordering wherever necessary.

### 19.5 Invalid commands

Invalid commands must not consume random values.

Submitting an invalid command and then continuing with valid commands must produce the same outcome as if the invalid command had never been submitted.

## 20. Serialization and replay

The engine must support serializing its complete authoritative state. This enables future save systems, debugging, and deterministic testing.

### 20.1 Snapshot requirements

A snapshot must contain sufficient information to resume execution exactly. This includes:

- Current phase.
- Round number.
- Player state.
- Every card's zone and state.
- Stronghold damage.
- Resource readiness.
- Unit readiness and damage.
- Initiative state.
- Active player.
- Pending player decisions.
- RNG state.
- Instance-ID allocation state.
- Match outcome.

### 20.2 Definition compatibility

Snapshots depend on the card definitions used to create the game.

The serialization format should contain a version identifier and a way to detect incompatible card definitions, such as a catalog fingerprint.

The engine must not silently load a snapshot using incompatible definitions.

### 20.3 Replay

A game should be reproducible from:

1. Initial match configuration.
2. Card definitions.
3. Seed.
4. Starting-player override.
5. Ordered command log.

The engine does not need to store every command internally. A caller may maintain the command log.

### 20.4 Snapshot round-trip

Serializing and deserializing a game must preserve its behavior.

After restoration, identical subsequent commands must produce identical states and events.

### 20.5 Format

Use a standard Rust serialization approach, such as Serde. JSON is sufficient for Milestone 1A.

No database, filesystem service, cloud storage, or save-game UI is required.

Cross-version snapshot migration is outside scope.

## 21. Legal action discovery

The engine should provide a way to determine which commands are currently legal. This will support the future Bevy UI and AI opponents.

### 21.1 Requirements

The caller must be able to determine:

- Which player or players may act.
- Which command categories are available.
- Which cards can be played.
- Which units can attack.
- Which targets can be attacked.
- Whether passing is legal.
- Whether initiative can be claimed.

### 21.2 Interface

The exact API is flexible, conceptually:

~~~rust
game.legal_commands_for(player_id)
~~~

The engine may return complete legal commands or structured legal-action information sufficient to construct them.

### 21.3 Validation authority

Legal-action discovery is a convenience. The command execution path remains authoritative.

The engine must validate every submitted command even if the caller previously requested legal actions.

### 21.4 No AI

The rules engine must not rank, score, or select legal actions. Strategic evaluation belongs to a future AI layer.

## 22. Error handling

The engine must use structured errors rather than panics for invalid input or illegal gameplay.

### 22.1 Error categories

| Error                    | Meaning                                      |
| ------------------------ | -------------------------------------------- |
| InvalidConfiguration     | Match configuration is invalid               |
| UnknownCardDefinition    | Referenced definition does not exist         |
| InvalidPlayer            | Player identifier is invalid                 |
| WrongPhase               | Command cannot be executed in current phase  |
| NotYourAction            | Player is not eligible to act                |
| InvalidCard              | Referenced card is unavailable or invalid    |
| InsufficientResources    | Card cost cannot be paid                     |
| UnitExhausted            | Attacking unit is not ready                  |
| InvalidTarget            | Target is not a legal enemy                  |
| DecisionAlreadySubmitted | Player already completed this phase decision |
| InitiativeUnavailable    | Initiative has already been claimed          |
| GameAlreadyFinished      | Match is terminal                            |

The final Rust error types should reflect actual operations rather than reproducing this table mechanically.

### 22.2 Transactional failure

For every invalid command:

- State remains unchanged.
- No events are committed.
- RNG remains unchanged.
- Current player remains unchanged.
- Current phase remains unchanged.

### 22.3 Hidden-information safety

Errors returned through player-facing interfaces must not disclose hidden opponent information.

For example, attempting to reference an inaccessible card must not reveal whether that card exists in the opponent's hand.

### 22.4 Internal invariants

An internal invariant violation is distinct from a player submitting an invalid command.

Normal invalid gameplay must never be treated as an unrecoverable engine failure.

## 23. Rust architecture

### 23.1 Workspace structure

Extend the existing repository with a new crate.

Suggested structure:

~~~text
low-kingdoms/
├── apps/
│   └── game/
├── crates/
│   ├── client/
│   └── rules/
│       ├── Cargo.toml
│       ├── AGENTS.md
│       ├── src/
│       │   ├── lib.rs
│       │   ├── card.rs
│       │   ├── command.rs
│       │   ├── combat.rs
│       │   ├── engine.rs
│       │   ├── error.rs
│       │   ├── event.rs
│       │   ├── game.rs
│       │   ├── player.rs
│       │   ├── resource.rs
│       │   ├── state.rs
│       │   └── view.rs
│       └── tests/
├── docs/
│   └── plans/
│       ├── scaffold.md
│       └── milestone-1a.md
└── Cargo.toml
~~~

This is a suggested module layout, not a requirement to create every listed file. Prefer fewer modules when the implementation remains clear.

### 23.2 Crate name

Recommended package name: **low-kingdoms-rules**.

It should be a normal Rust library. No executable is required.

### 23.3 Dependencies

The rules crate must not depend on Bevy.

Use a minimal dependency set. Reasonable dependencies include:

- Serde for serialization.
- A deterministic RNG implementation.
- A lightweight structured-error library, if justified.

Do not add libraries for speculative features.

### 23.4 Dependency direction

The intended future dependency structure is:

~~~text
apps/game
    |
    v
crates/client
    |
    v
crates/rules
~~~

The rules crate must never depend on the client crate.

Milestone 1A does not require connecting the client to the new rules crate. That integration belongs to Milestone 1B.

### 23.5 Domain types

Prefer explicit Rust types over unstructured maps or stringly typed APIs.

Examples:

- PlayerId
- CardDefinitionId
- CardInstanceId
- GamePhase
- MatchResult
- PlayerCommand
- GameEvent
- GameState

Use enums for finite sets of meaningful states. Use newtypes where mixing identifiers would cause correctness problems.

### 23.6 Engine ownership

The rules engine owns gameplay state transitions.

External callers must not be able to mutate authoritative state through unrestricted public fields. Prefer private implementation fields with carefully defined public operations.

### 23.7 No premature abstraction

Do not introduce generalized effect interpreters, ability scripting, dynamic plugin registries, runtime reflection, event-sourcing infrastructure, generic entity-component systems, renderer-specific abstractions, networking protocols, or future card-type placeholders that require unimplemented behavior.

Simple Rust enums, structs, and functions are sufficient.

### 23.8 Repository conventions

Follow the existing project's conventions:

- Rust 1.98.0.
- Edition 2024.
- Existing strict Clippy configuration.
- Existing formatting rules.
- Existing workspace dependency policies.
- Existing AGENTS.md guidance.
- Existing development and testing workflow.

The current Bevy version is 0.19.1, but the rules crate must not depend on it.

Update the repository's architecture and agent documentation to describe the new rules crate.

## 24. Testing strategy

Milestone 1A is a headless milestone. Testing is the primary mechanism for demonstrating correctness.

The implementation should prioritize readable, deterministic tests over elaborate test infrastructure.

### 24.1 Unit tests

**Initialization:**
- Two players are initialized.
- Decks are shuffled deterministically.
- Both players receive six cards.
- Starting initiative is seeded.
- Explicit starting-player overrides work.
- Invalid configurations are rejected.

**Mulligans:**
- Keeping a hand preserves it.
- Mulligans return and reshuffle the original six cards.
- Exactly six replacement cards are drawn.
- Players cannot mulligan twice.
- Both players must complete decisions before setup progresses.
- Submitted decision order does not change the result.

**Initial resources:**
- Zero, one, or two resources are legal.
- More than two is rejected.
- Selected cards leave the hand.
- Resources become ready before the first action phase.
- Opponents cannot inspect resourced card identities.

**Round progression:**
- Round one skips regroup.
- Later rounds draw two cards.
- Players may resource up to one card.
- Units and resources become ready after resourcing.
- Initiative determines the starting player.

**Actions:**
- Actions alternate correctly.
- Playing a unit consumes one action.
- Passing does not permanently end a player's actions.
- Two consecutive passes end the round.
- Non-pass actions reset the consecutive-pass sequence.
- Claiming initiative ends that player's actions.
- The remaining player can perform multiple actions.
- Initiative persists if nobody claims it.

**Resources:**
- Costs are enforced.
- Zero-cost cards can be played.
- Payment exhausts exactly required resources.
- Automatic payment is deterministic.
- Exhausted resources cannot be spent.
- Resources ready during regroup.

**Unit deployment:**
- Units enter exhausted.
- Units cannot attack immediately.
- Units become ready during regroup.
- Duplicate unit definitions are permitted.
- Battlefield capacity is unrestricted.

**Combat:**
- Only ready units can attack.
- Attackers become exhausted.
- Defenders do not become exhausted.
- Exhausted units can defend.
- Damage is simultaneous.
- Damage persists.
- Lethal damage destroys units.
- Both units can be destroyed.
- Strongholds do not retaliate.
- Enemy strongholds may be attacked directly.
- Enemy units do not restrict stronghold targeting.

**Deck exhaustion:**
- Drawing from a deck with one remaining card works.
- Each missing draw causes 3 stronghold damage.
- Empty decks do not automatically cause defeat.
- Deck exhaustion can destroy strongholds.
- Simultaneous lethal exhaustion produces a draw.

**Victory:**
- Stronghold destruction ends the game.
- The surviving player wins.
- No actions are permitted after game completion.
- Terminal state and result are preserved through serialization.

### 24.2 Determinism tests

Create tests that initialize two matches using identical configurations and seeds. Submit identical command sequences.

Verify that state snapshots, event sequences, and final outcomes match.

Repeat using different seeds and starting-player overrides.

### 24.3 Invalid-command tests

For each major invalid-command category:

1. Capture the complete state.
2. Submit the invalid command.
3. Verify the expected error.
4. Verify the state is unchanged.

Tests should include RNG and pending decision state.

### 24.4 Serialization tests

Verify:

- State can be serialized and deserialized.
- Restored state equals original.
- Continued execution produces identical results.
- Incompatible snapshot versions or card definitions are rejected.

### 24.5 Hidden-information tests

Verify that player-specific views do not reveal:

- Opponent hand identities.
- Opponent resource identities.
- Either player's deck order.
- Private mulligan decisions before resolution.
- Private card identities through events or validation errors.

### 24.6 Invariant tests

The engine should verify important invariants:

- Every card instance exists in exactly one zone.
- Every instance ID is unique.
- A destroyed unit is not still on the battlefield.
- Ready resources never become exhausted without a valid operation.
- The active player is eligible to act.
- A terminal game accepts no further gameplay commands.
- No valid command causes arithmetic overflow.

Property-based testing is appropriate where it improves confidence without introducing a large supporting framework.

### 24.7 Complete scripted matches

At least three complete matches must be demonstrated through headless tests.

**Scenario A — Combat victory.** Two players deploy units, attack, and eventually destroy a stronghold.

**Scenario B — Deck exhaustion victory.** A match continues until one player's deck-exhaustion damage destroys their stronghold.

**Scenario C — Initiative and passing.** A match demonstrates passing, claiming initiative, continued actions by the remaining player, and correct next-round initiative.

These scenarios should use minimal test fixtures rather than a fully designed card library.

## 25. Acceptance criteria

Milestone 1A is complete when all of the following are true.

### 25.1 Functional acceptance

| ID  | Criterion                                                   |
| --- | ----------------------------------------------------------- |
| F01 | Two-player matches initialize successfully                  |
| F02 | Starting-player selection is deterministic and overrideable |
| F03 | Decks shuffle reproducibly                                  |
| F04 | Opening hands and mulligans work                            |
| F05 | Initial resources work                                      |
| F06 | Regroup phases work                                         |
| F07 | Alternating actions work                                    |
| F08 | Passing and claiming initiative work                        |
| F09 | Unit deployment works                                       |
| F10 | Resource payments work                                      |
| F11 | Unit exhaustion and readiness work                          |
| F12 | Unit combat works                                           |
| F13 | Stronghold attacks work                                     |
| F14 | Persistent damage works                                     |
| F15 | Unit destruction works                                      |
| F16 | Deck exhaustion damage works                                |
| F17 | Victory and defeat work                                     |
| F18 | Hidden information is respected                             |
| F19 | Invalid commands do not mutate state                        |
| F20 | Complete matches can be played headlessly                   |

### 25.2 Technical acceptance

| ID  | Criterion                                          |
| --- | -------------------------------------------------- |
| T01 | Rules crate has no Bevy dependency                 |
| T02 | State transitions are deterministic                |
| T03 | Commands are explicitly validated                  |
| T04 | Structured events are emitted                      |
| T05 | Player-specific views are supported                |
| T06 | State serialization works                          |
| T07 | Snapshot restoration works                         |
| T08 | Deterministic replay works                         |
| T09 | Complete scripted matches pass                     |
| T10 | Existing strict Rust lints pass                    |
| T11 | Existing repository tests remain passing           |
| T12 | Native compilation works on supported CI platforms |
| T13 | Rules crate compiles for WebAssembly               |
| T14 | Public APIs are documented                         |
| T15 | Architecture and agent documentation are updated   |

### 25.3 Verification commands

The existing repository's verification commands remain authoritative. Relevant checks should include the equivalent of:

~~~bash
cargo test --locked -p low-kingdoms-rules

cargo clippy --locked \
  -p low-kingdoms-rules \
  --all-targets \
  -- -D warnings

cargo check --locked \
  -p low-kingdoms-rules \
  --target wasm32-unknown-unknown

just check
~~~

The new crate must be incorporated into the maintained repository verification workflow. Do not create a parallel CI system when the existing one can be extended.

### 25.4 Evidence

The implementation must include an accurate completion report documenting implemented features, tests executed, passed checks, failed or blocked checks, target/platform verification, known limitations, and remaining work for Milestone 1B.

Do not claim a platform has been runtime-tested merely because compilation succeeded.

## 26. Implementation sequence

The implementation should proceed in small, reviewable increments. This defines a recommended order, not mandatory PR boundaries.

### Phase 1: Domain model

Implement rules crate, player identifiers, card definitions, card instances, zones, unit state, stronghold state, match configuration, and game state.

**Acceptance:** A valid two-player game state can be created and inspected.

### Phase 2: Initialization and setup

Implement seeded randomness, deck shuffling, opening hands, mulligans, initial resourcing, and starting initiative.

**Acceptance:** A match reaches its first action phase deterministically.

### Phase 3: Actions and resources

Implement Play Unit, resource payment, Pass, Claim Initiative, action alternation, round completion, and regroup progression.

**Acceptance:** Players can deploy units and advance through multiple rounds.

### Phase 4: Combat and victory

Implement attack validation, attacker exhaustion, simultaneous combat damage, persistent damage, unit destruction, stronghold damage, deck exhaustion, victory, and defeat.

**Acceptance:** Complete matches can be played headlessly.

### Phase 5: Engine interfaces

Implement or finalize structured errors, game events, player-specific views, legal action discovery, serialization, and replay support.

These interfaces may be developed alongside earlier phases if doing so keeps the implementation simpler.

**Acceptance:** The engine is suitable for integration with a future presentation layer.

### Phase 6: Verification and documentation

Implement comprehensive deterministic tests, complete scripted matches, invariant tests, serialization tests, hidden-information tests, native and WASM compilation checks, and documentation updates.

**Acceptance:** All Milestone 1A criteria pass.

## 27. Rules summary

The following summarizes the confirmed gameplay decisions from the interview.

| #   | Decision            | Rule                                            |
| --- | ------------------- | ----------------------------------------------- |
| 1   | Scope               | Minimal playable rules engine                   |
| 2   | Initiative          | Players may claim initiative                    |
| 3   | Passing             | SWU-style passing and initiative                |
| 4   | Starting player     | Seeded random with explicit override            |
| 5   | Mulligans           | One full-hand mulligan                          |
| 6   | Initial resources   | Up to two cards                                 |
| 7   | Resource visibility | Facedown, owner-visible                         |
| 8   | Deck exhaustion     | Three stronghold damage per missing draw        |
| 9   | Round structure     | Draw, resource, ready, action                   |
| 10  | Unit readiness      | Units enter exhausted                           |
| 11  | Combat              | Simultaneous damage, persistent wounds          |
| 12  | Targeting           | Unrestricted enemy unit or stronghold targeting |
| 13  | Statistics          | Base statistics plus accumulated damage         |
| 14  | Battlefield         | Unlimited units and duplicates                  |
| 15  | Resource payment    | Automatic deterministic exhaustion              |
| 16  | Action economy      | One card per action                             |
| 17  | Engine control      | Explicit command-driven API                     |

These decisions are locked for Milestone 1A. Changes should require an explicit rules decision rather than being introduced opportunistically during implementation.

## 28. Implementation defaults and assumptions

The interview established the gameplay rules above. Several supporting details were not individually decided but are necessary to produce a complete implementation specification.

| Detail                              | Default                            |
| ----------------------------------- | ---------------------------------- |
| Players                             | Exactly two                        |
| Stronghold Health                   | Supplied by match configuration    |
| Minimum normal deck size            | Six cards                          |
| Maximum deck size                   | Not enforced                       |
| Hand-size limit                     | None                               |
| First-round regroup                 | Skipped                            |
| Mulligan decisions                  | Buffered until both players submit |
| Resourcing decisions                | Buffered until both players submit |
| Command processing                  | Atomic                             |
| Automatic resource spending         | Ascending instance-ID order        |
| Card instance identifiers           | Stable, never reused               |
| Discard visibility                  | Public                             |
| Simultaneous lethal deck exhaustion | Draw                               |
| Game duration                       | No round limit                     |
| Invalid commands                    | Structured errors; no mutation     |
| Randomness                          | Explicit deterministic PRNG        |
| Snapshot format                     | Versioned serialization            |
| Card definitions                    | Injected into engine               |
| Effects system                      | Not implemented                    |
| AI                                  | Not implemented                    |

These are implementation defaults, not additional card or combat-design decisions. They may be revisited if concrete implementation or testing evidence demonstrates a problem.

## 29. Non-goals and scope protection

Milestone 1A must not expand into Milestone 1B or later game systems.

**Do not implement presentation.** No Play button, combat board, card UI, graphics, animations, or input integration is required.

**Do not design cards.** Use minimal fixture definitions sufficient to exercise the rules. Do not create a starter card library, named units, faction content, or card balance tables.

**Do not implement commanders.** Commanders are a future gameplay system and should not influence the initial rules implementation beyond the architectural requirement that additional mechanics remain possible.

**Do not implement actions or upgrades.** These card types require effects and targeting behaviors outside the agreed 1A scope.

**Do not implement a generic effect engine.** The initial engine only needs the concrete behaviors established in this document.

**Do not implement AI.** Legal action discovery is sufficient preparation for AI integration.

**Do not implement persistence infrastructure.** Serializable game state is required; databases and save-game management are not.

**Do not implement multiplayer.** The engine must support two logical players, but networking, authentication, synchronization, and remote authority are separate concerns.

**Do not overengineer extensibility.** Favor straightforward code and clean domain boundaries over speculative abstractions.

## 30. Definition of done

Milestone 1A is done when an automated headless test can:

1. Construct two valid decks using basic unit definitions.
2. Initialize a match with a deterministic seed.
3. Complete both players' mulligan decisions.
4. Select starting resources.
5. Start the first action phase.
6. Play units by spending resources.
7. Alternate actions between the players.
8. Attack units and strongholds.
9. Resolve simultaneous combat damage.
10. Destroy units and move them to discard piles.
11. Pass and claim initiative.
12. Complete rounds and regroup.
13. Draw cards and acquire additional resources.
14. Apply deck exhaustion damage when necessary.
15. Destroy a stronghold.
16. Produce the correct final match result.
17. Reproduce the same game using its initial configuration and command sequence.
18. Serialize and restore an intermediate state.
19. Continue that restored game deterministically.
20. Verify that illegal commands cannot corrupt the game state.

All existing repository quality requirements must continue to pass.

No rendering, card artwork, AI, or campaign integration is necessary.

## Milestone 1B handoff

Once 1A is complete, Milestone 1B should integrate the rules crate with the existing Bevy client.

The client will be responsible for rendering the game, collecting player input, and presenting events. The rules engine will remain responsible for legality, state transitions, combat, and victory.

~~~text
                  Bevy Client
                      |
                      | Player command
                      v
                Rules Engine
                      |
             Validate + Execute
                      |
                      v
              Updated GameState
                      |
                      | Events / PlayerView
                      v
                  Bevy Client
                      |
                      v
               Render game UI
~~~

The Bevy client must not duplicate gameplay rules. The rules crate must not know that Bevy exists.

**Final deliverable for 1A:** A tested, deterministic Rust library capable of executing an entire Low Kingdoms battle through explicit commands, with no gameplay UI and no dependencies on the rendering engine.
