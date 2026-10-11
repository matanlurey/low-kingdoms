//! Ordered transition events and owner-specific event projection.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::command::AttackTarget;
use crate::model::{Card, CardId, Outcome, Phase, PlayerId};
use crate::view::UnitView;

/// One damage assignment within a simultaneous combat batch.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DamageAssignment {
    /// Public unit or stronghold receiving damage.
    pub target: AttackTarget,
    /// Damage assigned by this combat.
    pub amount: u32,
    /// Accumulated damage after the entire batch.
    pub total: u64,
}

/// Serializable, player-filtered explanation of a successful state transition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Event {
    /// Match initialization completed.
    MatchInitialized,
    /// Opening initiative was selected or overridden.
    StartingPlayerSelected {
        /// Initiative holder.
        player: PlayerId,
    },
    /// A deck was shuffled without exposing its order.
    DeckShuffled {
        /// Deck owner.
        player: PlayerId,
    },
    /// Cards were drawn; only the owner receives the identities.
    CardsDrawn {
        /// Drawing player.
        player: PlayerId,
        /// Number actually drawn.
        count: usize,
        /// `Some` for the owner, `None` for their opponent.
        cards: Option<Vec<Card>>,
    },
    /// A buffered mulligan decision has resolved, after both submissions.
    MulliganCompleted {
        /// Deciding player.
        player: PlayerId,
        /// Whether their entire hand was replaced.
        replaced: bool,
    },
    /// Selected cards moved to the exhausted resource zone.
    CardsResourced {
        /// Owner of the new resources.
        player: PlayerId,
        /// Number of cards moved.
        count: usize,
        /// Identities for the owner only.
        cards: Option<Vec<Card>>,
    },
    /// Exact resource payment, in ascending instance-ID order.
    ResourcesExhausted {
        /// Paying player.
        player: PlayerId,
        /// Number paid; zero-cost deployments emit a count of zero.
        count: usize,
        /// Resource IDs for the owner only.
        cards: Option<Vec<CardId>>,
    },
    /// Exhausted resources became ready.
    ResourcesReadied {
        /// Resource owner.
        player: PlayerId,
        /// Number whose readiness changed.
        count: usize,
    },
    /// Exhausted units became ready, in ascending instance-ID order.
    UnitsReadied {
        /// Unit owner.
        player: PlayerId,
        /// Public identities whose readiness changed.
        cards: Vec<CardId>,
    },
    /// The public phase changed.
    PhaseChanged {
        /// Newly entered phase.
        phase: Phase,
    },
    /// A round began, before its mandatory draws if this is regroup.
    RoundStarted {
        /// New round number.
        round: u64,
        /// This round's initiative.
        initiative: PlayerId,
    },
    /// An action phase ended.
    RoundEnded {
        /// Completed round.
        round: u64,
    },
    /// An eligible player now has the next action opportunity.
    ActionAvailable {
        /// Next acting player.
        player: PlayerId,
    },
    /// A temporary pass consumed an action opportunity.
    PlayerPassed {
        /// Passing player.
        player: PlayerId,
    },
    /// Initiative was secured and the player stopped acting this round.
    InitiativeClaimed {
        /// Claiming player.
        player: PlayerId,
    },
    /// A paid unit entered the battlefield exhausted.
    UnitPlayed {
        /// Complete public deployment state.
        unit: UnitView,
    },
    /// A validated attack began.
    AttackDeclared {
        /// Acting player.
        player: PlayerId,
        /// Ready attacker.
        attacker: CardId,
        /// Enemy target.
        target: AttackTarget,
    },
    /// Attacking exhausted this unit.
    UnitExhausted {
        /// Public attacker identity.
        card: CardId,
    },
    /// The entire combat damage batch, before any destruction.
    CombatDamage {
        /// Unit assignments in ascending target-ID order; one for a stronghold attack.
        assignments: Vec<DamageAssignment>,
    },
    /// A lethal unit moved to its owner's public discard pile.
    UnitDestroyed {
        /// Discard owner.
        player: PlayerId,
        /// Preserved card identity.
        card: Card,
    },
    /// Missing mandatory draws damaged a stronghold.
    DeckExhaustionDamage {
        /// Player unable to draw.
        player: PlayerId,
        /// Number of missing draws.
        missing: u8,
        /// Three times the missing draws.
        amount: u32,
        /// Accumulated stronghold damage.
        total: u64,
    },
    /// A terminal result was determined after completing simultaneous damage.
    GameEnded {
        /// Final immutable outcome.
        outcome: Outcome,
    },
}

/// Private event storage with explicit owner filtering. Debug output omits event contents.
///
/// The engine is a trusted authority. Give presentation code [`Self::for_player`] output only.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct EventBatch {
    pub(crate) events: Vec<Event>,
}

impl EventBatch {
    /// Returns ordered events with all opposing private identities omitted.
    #[must_use]
    pub fn for_player(&self, viewer: PlayerId) -> Vec<Event> {
        self.events
            .iter()
            .cloned()
            .map(|mut event| {
                match &mut event {
                    Event::CardsDrawn { player, cards, .. }
                    | Event::CardsResourced { player, cards, .. }
                        if *player != viewer =>
                    {
                        *cards = None;
                    }
                    Event::ResourcesExhausted { player, cards, .. } if *player != viewer => {
                        *cards = None;
                    }
                    _ => {}
                }
                event
            })
            .collect()
    }
}

impl fmt::Debug for EventBatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventBatch")
            .field("count", &self.events.len())
            .finish_non_exhaustive()
    }
}
