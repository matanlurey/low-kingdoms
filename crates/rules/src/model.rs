//! Immutable definitions, stable identifiers, and match configuration.

use serde::{Deserialize, Serialize};

/// One of the match's exactly two players; invalid player IDs are unrepresentable.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PlayerId {
    /// First player, allocated before the second player.
    Zero,
    /// Second player.
    One,
}

impl PlayerId {
    /// Canonical player order for all simultaneous operations.
    pub const ALL: [Self; 2] = [Self::Zero, Self::One];

    /// Returns the opposing player.
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Zero,
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Zero => 0,
            Self::One => 1,
        }
    }
}

/// Stable identifier for an injected card definition.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DefinitionId(pub u32);

/// Stable identifier for one card occurrence; zero is never allocated.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct CardId(pub u64);

/// Static unit statistics. Definitions have no mutable match state or abilities.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnitDefinition {
    /// Unique catalog identity.
    pub id: DefinitionId,
    /// Exact number of resources exhausted to deploy this unit.
    pub cost: u32,
    /// Combat damage, including retaliation.
    pub power: u32,
    /// Positive maximum Health.
    pub health: u32,
}

/// Immutable identity preserved when a card changes zones.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// Identity unique within the match.
    pub id: CardId,
    /// Static definition identity.
    pub definition: DefinitionId,
}

/// Initial deck and stronghold for one player.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerConfig {
    /// At least six definitions, in caller-provided order before shuffling.
    pub deck: Vec<DefinitionId>,
    /// Positive maximum stronghold Health.
    pub stronghold_health: u32,
}

/// All match inputs except the separately injected definition catalog.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MatchConfig {
    /// Players in [`PlayerId::ALL`] order.
    pub players: [PlayerConfig; 2],
    /// Explicit seed; the engine never obtains entropy from the environment.
    pub seed: u64,
    /// Optional opening initiative override, which consumes no random value.
    pub starting_player: Option<PlayerId>,
}

/// Public match phase; automatic regroup drawing never waits for a command.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Phase {
    /// Both players independently choose keep or full-hand mulligan.
    Mulligan,
    /// Both players independently select up to two resources.
    InitialResourcing,
    /// Mandatory draws have resolved; players select up to one resource.
    Regroup,
    /// Alternating unit plays, attacks, passes, or initiative claims.
    Action,
    /// No further commands are accepted.
    Finished,
}

/// Final result of a match.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Outcome {
    /// The opposing stronghold was destroyed.
    Winner(PlayerId),
    /// Both strongholds were destroyed in the same regroup draw batch.
    Draw,
}
