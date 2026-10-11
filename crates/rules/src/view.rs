//! Safe owner-specific projections. No deck order or opposing private IDs appear here.

use serde::{Deserialize, Serialize};

use crate::model::{Card, Outcome, Phase, PlayerId};

/// A public unit's printed statistics and independent battlefield state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnitView {
    /// Stable card identity and definition.
    pub card: Card,
    /// Controlling player.
    pub owner: PlayerId,
    /// Printed Power; damage never changes it.
    pub power: u32,
    /// Printed maximum Health.
    pub health: u32,
    /// Persistent damage accumulated before destruction.
    pub damage: u64,
    /// Whether this unit may attack.
    pub ready: bool,
}

/// A resource visible only to its owner.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OwnedResource {
    /// Underlying card identity, preserved when resourced.
    pub card: Card,
    /// Whether this resource can pay a cost.
    pub ready: bool,
}

/// Public resource information without individual card identifiers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceCounts {
    /// Number of facedown cards.
    pub total: usize,
    /// Number currently spendable.
    pub ready: usize,
}

/// Public information about one player.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicPlayer {
    /// Number of cards remaining; no deck identities or order.
    pub deck_size: usize,
    /// Number of cards held; no opposing hand identities.
    pub hand_size: usize,
    /// Resource total and readiness, without identities.
    pub resources: ResourceCounts,
    /// Public battlefield in deployment order.
    pub units: Vec<UnitView>,
    /// Public discard pile in destruction order.
    pub discard: Vec<Card>,
    /// Configured positive stronghold maximum Health.
    pub stronghold_health: u32,
    /// Persistent damage, which can exceed maximum Health in a terminal match.
    pub stronghold_damage: u64,
}

/// Complete information the selected logical player is entitled to inspect.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayerView {
    /// Perspective used to construct this view.
    pub player: PlayerId,
    /// Current public phase.
    pub phase: Phase,
    /// Current round, beginning at one.
    pub round: u64,
    /// Initiative for this round.
    pub initiative: PlayerId,
    /// Initiative secured for the next round, or retained if unclaimed.
    pub next_initiative: PlayerId,
    /// Player whose action is expected; absent during decisions and after finishing.
    pub active_player: Option<PlayerId>,
    /// Player who permanently stopped acting this round by claiming initiative.
    pub initiative_claimant: Option<PlayerId>,
    /// Consecutive temporary passes in the current action phase.
    pub consecutive_passes: u8,
    /// Public result, if finished.
    pub outcome: Option<Outcome>,
    /// Public information for both players in canonical order.
    pub players: [PublicPlayer; 2],
    /// This player's ordered hand only.
    pub hand: Vec<Card>,
    /// This player's resources only, in ascending instance-ID order.
    pub resources: Vec<OwnedResource>,
    /// Whether this player has already committed their current private phase decision.
    pub decision_submitted: bool,
}
