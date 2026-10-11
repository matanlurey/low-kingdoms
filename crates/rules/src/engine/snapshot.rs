//! Versioned, privileged JSON snapshots with catalog and state invariant validation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{Game, canonical_definitions, validate_config};
use crate::error::Error;
use crate::model::{CardId, MatchConfig, Outcome, Phase, PlayerId, UnitDefinition};
use crate::state::{Decision, State};

const VERSION: u32 = 1;

#[derive(Serialize)]
struct Snapshot<'a> {
    version: u32,
    definitions: &'a [UnitDefinition],
    config: &'a MatchConfig,
    state: &'a State,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSnapshot {
    version: u32,
    definitions: Vec<UnitDefinition>,
    config: MatchConfig,
    state: State,
}

impl Game {
    /// Serializes full authoritative state, including hidden zones and pending decisions.
    ///
    /// **Privileged:** never send this JSON to either player. Use [`Self::view_for`] instead.
    /// The canonical catalog is stored exactly, avoiding hash-collision compatibility checks.
    ///
    /// # Errors
    /// Returns `SnapshotFormat` if JSON serialization fails.
    pub fn snapshot(&self) -> Result<String, Error> {
        serde_json::to_string(&Snapshot {
            version: VERSION,
            definitions: &self.definitions,
            config: &self.config,
            state: &self.state,
        })
        .map_err(|_| Error::SnapshotFormat)
    }

    /// Restores a privileged snapshot against the caller's canonical definition catalog.
    ///
    /// Validates version, exact catalog equality, configuration, ownership/identity conservation,
    /// unit statistics, live-unit damage, resource ordering, decisions, phase, and outcome.
    /// Cross-version migrations and storage infrastructure are outside this API.
    ///
    /// # Errors
    /// Rejects malformed JSON, unknown fields, incompatible versions/catalogs, invalid
    /// definitions/configuration, or any violated authoritative state invariant.
    pub fn restore(json: &str, definitions: Vec<UnitDefinition>) -> Result<Self, Error> {
        let stored: StoredSnapshot =
            serde_json::from_str(json).map_err(|_| Error::SnapshotFormat)?;
        if stored.version != VERSION {
            return Err(Error::SnapshotVersion);
        }
        let definitions = canonical_definitions(definitions)?;
        if stored.definitions != definitions {
            return Err(Error::SnapshotDefinitions);
        }
        let next_id = validate_config(&stored.config, &definitions)?;
        let game = Self {
            config: stored.config,
            definitions,
            state: stored.state,
        };
        game.validate_snapshot(next_id)?;
        Ok(game)
    }

    fn validate_snapshot(&self, next_id: u64) -> Result<(), Error> {
        if self.state.next_id != next_id || self.state.round == 0 || self.state.passes > 1 {
            return Err(Error::SnapshotInvariant);
        }
        let mut expected = BTreeMap::new();
        let mut allocated = 1_u64;
        for owner in PlayerId::ALL {
            for &definition in &self.config.players[owner.index()].deck {
                expected.insert(CardId(allocated), (owner, definition));
                allocated += 1;
            }
        }
        for owner in PlayerId::ALL {
            let player = &self.state.players[owner.index()];
            let maximum_damage = u64::from(self.config.players[owner.index()].stronghold_health)
                - 1
                + u64::from(u32::MAX);
            if player.stronghold_damage > maximum_damage {
                return Err(Error::SnapshotInvariant);
            }
            let cards = player
                .deck
                .iter()
                .chain(&player.hand)
                .chain(&player.discard)
                .chain(player.resources.iter().map(|resource| &resource.card))
                .chain(player.units.iter().map(|unit| &unit.card));
            for card in cards {
                if expected.remove(&card.id) != Some((owner, card.definition)) {
                    return Err(Error::SnapshotInvariant);
                }
            }
            if player
                .resources
                .windows(2)
                .any(|pair| pair[0].card.id >= pair[1].card.id)
            {
                return Err(Error::SnapshotInvariant);
            }
            for unit in &player.units {
                let definition = self
                    .definitions
                    .iter()
                    .find(|definition| definition.id == unit.card.definition)
                    .ok_or(Error::SnapshotInvariant)?;
                if unit.power != definition.power
                    || unit.health != definition.health
                    || unit.damage >= u64::from(unit.health)
                {
                    return Err(Error::SnapshotInvariant);
                }
            }
            self.validate_pending(owner)?;
        }
        if !expected.is_empty() || self.state.pending.iter().all(Option::is_some) {
            return Err(Error::SnapshotInvariant);
        }
        self.validate_phase()?;
        let lethal = PlayerId::ALL.map(|owner| {
            self.state.players[owner.index()].stronghold_damage
                >= u64::from(self.config.players[owner.index()].stronghold_health)
        });
        let outcome = match lethal {
            [true, true] => Some(Outcome::Draw),
            [true, false] => Some(Outcome::Winner(PlayerId::One)),
            [false, true] => Some(Outcome::Winner(PlayerId::Zero)),
            [false, false] => None,
        };
        if self.state.outcome != outcome
            || (self.state.phase == Phase::Finished) != outcome.is_some()
        {
            return Err(Error::SnapshotInvariant);
        }
        Ok(())
    }

    fn validate_pending(&self, owner: PlayerId) -> Result<(), Error> {
        if let Some(decision) = &self.state.pending[owner.index()] {
            match (self.state.phase, decision) {
                (Phase::Mulligan, Decision::Mulligan(_)) => {}
                (Phase::InitialResourcing | Phase::Regroup, Decision::Resources(cards)) => {
                    let maximum = if self.state.phase == Phase::InitialResourcing {
                        2
                    } else {
                        1
                    };
                    let hand: BTreeSet<_> = self.state.players[owner.index()]
                        .hand
                        .iter()
                        .map(|card| card.id)
                        .collect();
                    if cards.len() > maximum
                        || cards.windows(2).any(|pair| pair[0] >= pair[1])
                        || cards.iter().any(|card| !hand.contains(card))
                    {
                        return Err(Error::SnapshotInvariant);
                    }
                }
                _ => return Err(Error::SnapshotInvariant),
            }
        }
        Ok(())
    }

    fn validate_phase(&self) -> Result<(), Error> {
        let state = &self.state;
        match state.phase {
            Phase::Mulligan | Phase::InitialResourcing => {
                if state.round != 1
                    || state.active.is_some()
                    || state.claimant.is_some()
                    || state.passes != 0
                {
                    return Err(Error::SnapshotInvariant);
                }
                for player in &state.players {
                    if player.hand.len() != 6
                        || !player.resources.is_empty()
                        || !player.units.is_empty()
                        || !player.discard.is_empty()
                        || player.stronghold_damage != 0
                    {
                        return Err(Error::SnapshotInvariant);
                    }
                }
            }
            Phase::Regroup => {
                if state.round < 2
                    || state.active.is_some()
                    || state.claimant.is_some()
                    || state.passes != 0
                {
                    return Err(Error::SnapshotInvariant);
                }
            }
            Phase::Action => {
                if state.active.is_none()
                    || state.active == state.claimant
                    || state.pending.iter().any(Option::is_some)
                    || (state.claimant.is_some() && state.passes != 0)
                {
                    return Err(Error::SnapshotInvariant);
                }
            }
            Phase::Finished => {
                if state.active.is_some() || state.pending.iter().any(Option::is_some) {
                    return Err(Error::SnapshotInvariant);
                }
            }
        }
        Ok(())
    }
}
