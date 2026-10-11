//! Authoritative command execution, discovery, snapshots, and replay.

use crate::command::{AttackTarget, Command, LoggedCommand};
use crate::error::Error;
use crate::event::{DamageAssignment, Event, EventBatch};
use crate::model::{Card, CardId, MatchConfig, Outcome, Phase, PlayerId, UnitDefinition};
use crate::rng::Rng;
use crate::state::{Decision, PlayerState, Resource, State, Unit};
use crate::view::{OwnedResource, PlayerView, PublicPlayer, ResourceCounts, UnitView};

mod snapshot;

/// A complete match with private authoritative state and no renderer dependencies.
///
/// Commands validate into an application plan before any mutation. There is no global state,
/// clock, OS entropy, asynchronous work, or whole-match cloning on the command path.
pub struct Game {
    config: MatchConfig,
    definitions: Vec<UnitDefinition>,
    state: State,
}

enum Validated {
    Decision(Decision),
    Play {
        hand_index: usize,
        definition: UnitDefinition,
        payment: Vec<usize>,
    },
    Attack {
        attacker_index: usize,
        target: AttackTarget,
        defender_index: Option<usize>,
    },
    Pass {
        next_round: Option<u64>,
    },
    Claim,
}

impl Game {
    /// Validates inputs, allocates IDs, selects initiative, shuffles, and draws opening hands.
    ///
    /// Returns the match and its filtered initialization event batch. Catalog input order has
    /// no effect. Instance allocation, random operations, and draws use player order 0 then 1.
    ///
    /// # Errors
    /// Rejects duplicate or zero-Health definitions, unknown deck definitions, decks shorter
    /// than six cards, zero-Health strongholds, or an unrepresentable instance counter.
    pub fn new(
        config: MatchConfig,
        definitions: Vec<UnitDefinition>,
    ) -> Result<(Self, EventBatch), Error> {
        let definitions = canonical_definitions(definitions)?;
        let next_id = validate_config(&config, &definitions)?;
        let mut allocated = 1_u64;
        let players = config.players.each_ref().map(|player| {
            let deck = player
                .deck
                .iter()
                .map(|&definition| {
                    let card = Card {
                        id: CardId(allocated),
                        definition,
                    };
                    allocated += 1;
                    card
                })
                .collect();
            PlayerState::new(deck)
        });
        let mut rng = Rng::new(config.seed);
        let initiative = config
            .starting_player
            .unwrap_or_else(|| PlayerId::ALL[rng.below(2)]);
        let mut game = Self {
            config,
            definitions,
            state: State {
                players,
                phase: Phase::Mulligan,
                round: 1,
                initiative,
                claimant: None,
                active: None,
                passes: 0,
                pending: [None, None],
                rng,
                next_id,
                outcome: None,
            },
        };
        let mut events = vec![
            Event::MatchInitialized,
            Event::StartingPlayerSelected { player: initiative },
        ];
        for player in PlayerId::ALL {
            game.shuffle(player, &mut events);
        }
        for player in PlayerId::ALL {
            game.draw(player, 6, &mut events);
        }
        events.push(Event::PhaseChanged {
            phase: Phase::Mulligan,
        });
        Ok((game, EventBatch { events }))
    }

    /// Executes one command atomically and returns deterministically ordered events.
    ///
    /// Independent setup/regroup decisions emit no events until both submissions arrive.
    /// Failed commands leave every field, including RNG and pending decisions, unchanged.
    ///
    /// # Errors
    /// Rejects terminal, wrong-phase, duplicate-decision, out-of-turn, inaccessible-card,
    /// unaffordable, exhausted-attacker, invalid-target, and unavailable-initiative commands.
    /// A round-ending pass also rejects an unrepresentable next round before mutation.
    pub fn apply(&mut self, player: PlayerId, command: &Command) -> Result<EventBatch, Error> {
        let validated = self.validate(player, command)?;
        let mut events = Vec::new();
        match validated {
            Validated::Decision(decision) => {
                self.state.pending[player.index()] = Some(decision);
                if self.state.pending.iter().all(Option::is_some) {
                    self.resolve_decisions(&mut events);
                }
            }
            Validated::Play {
                hand_index,
                definition,
                payment,
            } => {
                self.play(player, hand_index, definition, &payment, &mut events);
                self.continue_action(player, &mut events);
            }
            Validated::Attack {
                attacker_index,
                target,
                defender_index,
            } => {
                self.attack(player, attacker_index, target, defender_index, &mut events);
                if self.state.outcome.is_none() {
                    self.continue_action(player, &mut events);
                }
            }
            Validated::Pass { next_round } => {
                events.push(Event::PlayerPassed { player });
                if let Some(round) = next_round {
                    self.regroup(round, &mut events);
                } else {
                    self.state.passes += 1;
                    self.state.active = Some(player.opponent());
                    events.push(Event::ActionAvailable {
                        player: player.opponent(),
                    });
                }
            }
            Validated::Claim => {
                self.state.claimant = Some(player);
                events.push(Event::InitiativeClaimed { player });
                self.continue_action(player, &mut events);
            }
        }
        Ok(EventBatch { events })
    }

    /// Lists complete legal commands, in stable order, using the command validator.
    ///
    /// Unavailable players receive an empty list. Selections are canonical subsets, so two
    /// differently ordered initial resource vectors are represented once. No hidden IDs from
    /// another player's private zones are enumerated.
    #[must_use]
    pub fn legal_commands_for(&self, player: PlayerId) -> Vec<Command> {
        let own = &self.state.players[player.index()];
        let mut candidates = Vec::new();
        match self.state.phase {
            Phase::Mulligan => candidates.extend([Command::KeepHand, Command::MulliganHand]),
            Phase::InitialResourcing => {
                let mut ids: Vec<_> = own.hand.iter().map(|card| card.id).collect();
                ids.sort_unstable();
                candidates.push(Command::ChooseInitialResources { cards: Vec::new() });
                for (index, &id) in ids.iter().enumerate() {
                    candidates.push(Command::ChooseInitialResources { cards: vec![id] });
                    for &second in &ids[index + 1..] {
                        candidates.push(Command::ChooseInitialResources {
                            cards: vec![id, second],
                        });
                    }
                }
            }
            Phase::Regroup => {
                candidates.push(Command::ChooseRegroupResource { card: None });
                for card in &own.hand {
                    candidates.push(Command::ChooseRegroupResource {
                        card: Some(card.id),
                    });
                }
            }
            Phase::Action => {
                candidates.extend([Command::Pass, Command::ClaimInitiative]);
                for card in &own.hand {
                    candidates.push(Command::PlayUnit { card: card.id });
                }
                for unit in &own.units {
                    candidates.push(Command::Attack {
                        attacker: unit.card.id,
                        target: AttackTarget::Stronghold(player.opponent()),
                    });
                    for defender in &self.state.players[player.opponent().index()].units {
                        candidates.push(Command::Attack {
                            attacker: unit.card.id,
                            target: AttackTarget::Unit(defender.card.id),
                        });
                    }
                }
            }
            Phase::Finished => {}
        }
        candidates.retain(|command| self.validate(player, command).is_ok());
        candidates
    }

    /// Projects public state and only the selected player's private hand/resources.
    #[must_use]
    pub fn view_for(&self, player: PlayerId) -> PlayerView {
        let players = PlayerId::ALL.map(|owner| {
            let state = &self.state.players[owner.index()];
            PublicPlayer {
                deck_size: state.deck.len(),
                hand_size: state.hand.len(),
                resources: ResourceCounts {
                    total: state.resources.len(),
                    ready: state
                        .resources
                        .iter()
                        .filter(|resource| resource.ready)
                        .count(),
                },
                units: state
                    .units
                    .iter()
                    .map(|unit| unit_view(owner, unit))
                    .collect(),
                discard: state.discard.clone(),
                stronghold_health: self.config.players[owner.index()].stronghold_health,
                stronghold_damage: state.stronghold_damage,
            }
        });
        let own = &self.state.players[player.index()];
        PlayerView {
            player,
            phase: self.state.phase,
            round: self.state.round,
            initiative: self.state.initiative,
            next_initiative: self.state.claimant.unwrap_or(self.state.initiative),
            active_player: self.state.active,
            initiative_claimant: self.state.claimant,
            consecutive_passes: self.state.passes,
            outcome: self.state.outcome,
            players,
            hand: own.hand.clone(),
            resources: own
                .resources
                .iter()
                .map(|resource| OwnedResource {
                    card: resource.card,
                    ready: resource.ready,
                })
                .collect(),
            decision_submitted: self.state.pending[player.index()].is_some(),
        }
    }

    /// Reproduces initialization and an ordered command log without recording it internally.
    ///
    /// Returns the reconstructed match and initialization/command event batches in log order.
    ///
    /// # Errors
    /// Returns an initialization error or the first invalid command error. No partial match
    /// is returned when the supplied log is invalid.
    pub fn replay(
        config: MatchConfig,
        definitions: Vec<UnitDefinition>,
        log: &[LoggedCommand],
    ) -> Result<(Self, Vec<EventBatch>), Error> {
        let (mut game, initialized) = Self::new(config, definitions)?;
        let mut events = vec![initialized];
        for entry in log {
            events.push(game.apply(entry.player, &entry.command)?);
        }
        Ok((game, events))
    }

    fn validate(&self, player: PlayerId, command: &Command) -> Result<Validated, Error> {
        if self.state.phase == Phase::Finished {
            return Err(Error::GameAlreadyFinished);
        }
        match command {
            Command::KeepHand | Command::MulliganHand => {
                self.validate_decision_phase(player, Phase::Mulligan)?;
                Ok(Validated::Decision(Decision::Mulligan(matches!(
                    command,
                    Command::MulliganHand
                ))))
            }
            Command::ChooseInitialResources { cards } => {
                self.validate_decision_phase(player, Phase::InitialResourcing)?;
                self.validate_resources(player, cards, 2)
            }
            Command::ChooseRegroupResource { card } => {
                self.validate_decision_phase(player, Phase::Regroup)?;
                self.validate_resources(player, &card.iter().copied().collect::<Vec<_>>(), 1)
            }
            _ => {
                if self.state.phase != Phase::Action {
                    return Err(Error::WrongPhase);
                }
                if self.state.active != Some(player) || self.state.claimant == Some(player) {
                    return Err(Error::NotYourAction);
                }
                self.validate_action(player, command)
            }
        }
    }

    fn validate_decision_phase(&self, player: PlayerId, phase: Phase) -> Result<(), Error> {
        if self.state.phase != phase {
            return Err(Error::WrongPhase);
        }
        if self.state.pending[player.index()].is_some() {
            return Err(Error::DecisionAlreadySubmitted);
        }
        Ok(())
    }

    fn validate_resources(
        &self,
        player: PlayerId,
        cards: &[CardId],
        maximum: usize,
    ) -> Result<Validated, Error> {
        if cards.len() > maximum {
            return Err(Error::TooManyResources);
        }
        let mut selected = cards.to_vec();
        selected.sort_unstable();
        if selected.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::DuplicateResource);
        }
        if selected.iter().any(|id| {
            !self.state.players[player.index()]
                .hand
                .iter()
                .any(|card| card.id == *id)
        }) {
            return Err(Error::InvalidCard);
        }
        Ok(Validated::Decision(Decision::Resources(selected)))
    }

    fn validate_action(&self, player: PlayerId, command: &Command) -> Result<Validated, Error> {
        let own = &self.state.players[player.index()];
        match *command {
            Command::PlayUnit { card } => {
                let hand_index = own
                    .hand
                    .iter()
                    .position(|candidate| candidate.id == card)
                    .ok_or(Error::InvalidCard)?;
                let definition = *self
                    .definitions
                    .iter()
                    .find(|definition| definition.id == own.hand[hand_index].definition)
                    .ok_or(Error::InvalidCard)?;
                let cost =
                    usize::try_from(definition.cost).map_err(|_| Error::InsufficientResources)?;
                let payment: Vec<_> = own
                    .resources
                    .iter()
                    .enumerate()
                    .filter(|(_, resource)| resource.ready)
                    .map(|(index, _)| index)
                    .take(cost)
                    .collect();
                if payment.len() != cost {
                    return Err(Error::InsufficientResources);
                }
                Ok(Validated::Play {
                    hand_index,
                    definition,
                    payment,
                })
            }
            Command::Attack { attacker, target } => {
                let attacker_index = own
                    .units
                    .iter()
                    .position(|unit| unit.card.id == attacker)
                    .ok_or(Error::InvalidCard)?;
                if !own.units[attacker_index].ready {
                    return Err(Error::UnitExhausted);
                }
                let defender_index = match target {
                    AttackTarget::Stronghold(owner) if owner == player.opponent() => None,
                    AttackTarget::Unit(id) => Some(
                        self.state.players[player.opponent().index()]
                            .units
                            .iter()
                            .position(|unit| unit.card.id == id)
                            .ok_or(Error::InvalidTarget)?,
                    ),
                    AttackTarget::Stronghold(_) => return Err(Error::InvalidTarget),
                };
                Ok(Validated::Attack {
                    attacker_index,
                    target,
                    defender_index,
                })
            }
            Command::Pass => {
                let ends = self.state.claimant.is_some() || self.state.passes == 1;
                let next_round = if ends {
                    Some(
                        self.state
                            .round
                            .checked_add(1)
                            .ok_or(Error::ArithmeticLimit)?,
                    )
                } else {
                    None
                };
                Ok(Validated::Pass { next_round })
            }
            Command::ClaimInitiative => {
                if self.state.claimant.is_some() {
                    return Err(Error::InitiativeUnavailable);
                }
                Ok(Validated::Claim)
            }
            _ => Err(Error::WrongPhase),
        }
    }

    fn resolve_decisions(&mut self, events: &mut Vec<Event>) {
        let phase = self.state.phase;
        let pending = std::mem::take(&mut self.state.pending);
        for (player, decision) in PlayerId::ALL.into_iter().zip(pending) {
            match decision {
                Some(Decision::Mulligan(replaced)) => {
                    if replaced {
                        let own = &mut self.state.players[player.index()];
                        own.deck.append(&mut own.hand);
                        self.shuffle(player, events);
                        self.draw(player, 6, events);
                    }
                    events.push(Event::MulliganCompleted { player, replaced });
                }
                Some(Decision::Resources(cards)) => self.resource(player, &cards, events),
                None => {}
            }
        }
        if phase == Phase::Mulligan {
            self.state.phase = Phase::InitialResourcing;
            events.push(Event::PhaseChanged {
                phase: Phase::InitialResourcing,
            });
        } else {
            self.ready(events);
            if phase == Phase::InitialResourcing {
                events.push(Event::RoundStarted {
                    round: 1,
                    initiative: self.state.initiative,
                });
            }
            self.state.phase = Phase::Action;
            self.state.active = Some(self.state.initiative);
            events.push(Event::PhaseChanged {
                phase: Phase::Action,
            });
            events.push(Event::ActionAvailable {
                player: self.state.initiative,
            });
        }
    }

    fn shuffle(&mut self, player: PlayerId, events: &mut Vec<Event>) {
        self.state
            .rng
            .shuffle(&mut self.state.players[player.index()].deck);
        events.push(Event::DeckShuffled { player });
    }

    fn draw(&mut self, player: PlayerId, requested: u8, events: &mut Vec<Event>) {
        let own = &mut self.state.players[player.index()];
        let mut cards = Vec::new();
        let mut missing = 0_u8;
        for _ in 0..requested {
            if let Some(card) = own.deck.pop() {
                own.hand.push(card);
                cards.push(card);
            } else {
                missing += 1;
            }
        }
        events.push(Event::CardsDrawn {
            player,
            count: cards.len(),
            cards: Some(cards),
        });
        if missing > 0 {
            let amount = u32::from(missing) * 3;
            own.stronghold_damage += u64::from(amount);
            events.push(Event::DeckExhaustionDamage {
                player,
                missing,
                amount,
                total: own.stronghold_damage,
            });
        }
    }

    fn resource(&mut self, player: PlayerId, selected: &[CardId], events: &mut Vec<Event>) {
        let own = &mut self.state.players[player.index()];
        let mut cards = Vec::new();
        own.hand.retain(|card| {
            if selected.binary_search(&card.id).is_ok() {
                cards.push(*card);
                false
            } else {
                true
            }
        });
        cards.sort_unstable_by_key(|card| card.id);
        own.resources
            .extend(cards.iter().map(|&card| Resource { card, ready: false }));
        own.resources
            .sort_unstable_by_key(|resource| resource.card.id);
        events.push(Event::CardsResourced {
            player,
            count: cards.len(),
            cards: Some(cards),
        });
    }

    fn ready(&mut self, events: &mut Vec<Event>) {
        for player in PlayerId::ALL {
            let own = &mut self.state.players[player.index()];
            let mut count = 0;
            for resource in &mut own.resources {
                if !resource.ready {
                    resource.ready = true;
                    count += 1;
                }
            }
            events.push(Event::ResourcesReadied { player, count });
            let mut cards = Vec::new();
            for unit in &mut own.units {
                if !unit.ready {
                    unit.ready = true;
                    cards.push(unit.card.id);
                }
            }
            cards.sort_unstable();
            events.push(Event::UnitsReadied { player, cards });
        }
    }

    fn play(
        &mut self,
        player: PlayerId,
        hand_index: usize,
        definition: UnitDefinition,
        payment: &[usize],
        events: &mut Vec<Event>,
    ) {
        let own = &mut self.state.players[player.index()];
        let mut cards = Vec::new();
        for &index in payment {
            own.resources[index].ready = false;
            cards.push(own.resources[index].card.id);
        }
        events.push(Event::ResourcesExhausted {
            player,
            count: cards.len(),
            cards: Some(cards),
        });
        let card = own.hand.remove(hand_index);
        let unit = Unit {
            card,
            power: definition.power,
            health: definition.health,
            damage: 0,
            ready: false,
        };
        events.push(Event::UnitPlayed {
            unit: unit_view(player, &unit),
        });
        own.units.push(unit);
    }

    fn attack(
        &mut self,
        player: PlayerId,
        attacker_index: usize,
        target: AttackTarget,
        defender_index: Option<usize>,
        events: &mut Vec<Event>,
    ) {
        let attacker = &mut self.state.players[player.index()].units[attacker_index];
        attacker.ready = false;
        let attacker_id = attacker.card.id;
        let power = attacker.power;
        events.push(Event::AttackDeclared {
            player,
            attacker: attacker_id,
            target,
        });
        events.push(Event::UnitExhausted { card: attacker_id });
        let mut assignments = Vec::new();
        if let Some(index) = defender_index {
            let defender = &mut self.state.players[player.opponent().index()].units[index];
            let retaliation = defender.power;
            defender.damage += u64::from(power);
            assignments.push(DamageAssignment {
                target,
                amount: power,
                total: defender.damage,
            });
            let attacker = &mut self.state.players[player.index()].units[attacker_index];
            attacker.damage += u64::from(retaliation);
            assignments.push(DamageAssignment {
                target: AttackTarget::Unit(attacker_id),
                amount: retaliation,
                total: attacker.damage,
            });
            assignments.sort_unstable_by_key(|assignment| match assignment.target {
                AttackTarget::Unit(id) => id,
                AttackTarget::Stronghold(_) => CardId(0),
            });
        } else {
            let defender = &mut self.state.players[player.opponent().index()];
            defender.stronghold_damage += u64::from(power);
            assignments.push(DamageAssignment {
                target,
                amount: power,
                total: defender.stronghold_damage,
            });
        }
        events.push(Event::CombatDamage { assignments });
        self.destroy_units(events);
        self.finish_if_lethal(events);
    }

    fn destroy_units(&mut self, events: &mut Vec<Event>) {
        let mut destroyed = Vec::new();
        for player in PlayerId::ALL {
            self.state.players[player.index()].units.retain(|unit| {
                if unit.damage >= u64::from(unit.health) {
                    destroyed.push((player, unit.card));
                    false
                } else {
                    true
                }
            });
        }
        destroyed.sort_unstable_by_key(|(_, card)| card.id);
        for (player, card) in destroyed {
            self.state.players[player.index()].discard.push(card);
            events.push(Event::UnitDestroyed { player, card });
        }
    }

    fn continue_action(&mut self, player: PlayerId, events: &mut Vec<Event>) {
        self.state.passes = 0;
        let next = if self.state.claimant == Some(player.opponent()) {
            player
        } else {
            player.opponent()
        };
        self.state.active = Some(next);
        events.push(Event::ActionAvailable { player: next });
    }

    fn regroup(&mut self, round: u64, events: &mut Vec<Event>) {
        events.push(Event::RoundEnded {
            round: self.state.round,
        });
        self.state.round = round;
        self.state.initiative = self.state.claimant.unwrap_or(self.state.initiative);
        self.state.claimant = None;
        self.state.active = None;
        self.state.passes = 0;
        self.state.phase = Phase::Regroup;
        events.push(Event::RoundStarted {
            round,
            initiative: self.state.initiative,
        });
        events.push(Event::PhaseChanged {
            phase: Phase::Regroup,
        });
        for player in PlayerId::ALL {
            self.draw(player, 2, events);
        }
        self.finish_if_lethal(events);
    }

    fn finish_if_lethal(&mut self, events: &mut Vec<Event>) {
        let lethal = PlayerId::ALL.map(|player| {
            self.state.players[player.index()].stronghold_damage
                >= u64::from(self.config.players[player.index()].stronghold_health)
        });
        let outcome = match lethal {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(PlayerId::One)),
            [false, true] => Some(Outcome::Winner(PlayerId::Zero)),
            [false, false] => None,
        };
        if let Some(outcome) = outcome {
            self.state.outcome = Some(outcome);
            self.state.phase = Phase::Finished;
            self.state.active = None;
            self.state.pending = [None, None];
            events.push(Event::PhaseChanged {
                phase: Phase::Finished,
            });
            events.push(Event::GameEnded { outcome });
        }
    }
}

const fn unit_view(owner: PlayerId, unit: &Unit) -> UnitView {
    UnitView {
        card: unit.card,
        owner,
        power: unit.power,
        health: unit.health,
        damage: unit.damage,
        ready: unit.ready,
    }
}

fn canonical_definitions(
    mut definitions: Vec<UnitDefinition>,
) -> Result<Vec<UnitDefinition>, Error> {
    definitions.sort_unstable_by_key(|definition| definition.id);
    for definition in &definitions {
        if definition.health == 0 {
            return Err(Error::InvalidDefinition(definition.id));
        }
    }
    for pair in definitions.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(Error::DuplicateDefinition(pair[0].id));
        }
    }
    Ok(definitions)
}

fn validate_config(config: &MatchConfig, definitions: &[UnitDefinition]) -> Result<u64, Error> {
    let mut next_id = 1_u64;
    for player in &config.players {
        if player.deck.len() < 6 || player.stronghold_health == 0 {
            return Err(Error::InvalidConfiguration);
        }
        for &id in &player.deck {
            if !definitions.iter().any(|definition| definition.id == id) {
                return Err(Error::UnknownDefinition(id));
            }
        }
        next_id = next_id
            .checked_add(u64::try_from(player.deck.len()).map_err(|_| Error::ArithmeticLimit)?)
            .ok_or(Error::ArithmeticLimit)?;
    }
    Ok(next_id)
}
