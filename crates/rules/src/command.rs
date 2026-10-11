//! Explicit player decisions and replay log entries.

use serde::{Deserialize, Serialize};

use crate::model::{CardId, PlayerId};

/// Target of one ready unit's attack.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AttackTarget {
    /// An enemy battlefield unit.
    Unit(CardId),
    /// The enemy stronghold, regardless of enemy units.
    Stronghold(PlayerId),
}

/// One atomic decision submitted by the caller.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Command {
    /// Keep the six-card opening hand.
    KeepHand,
    /// Return, shuffle, and replace the entire opening hand once.
    MulliganHand,
    /// Commit zero, one, or two distinct cards from the player's hand.
    ChooseInitialResources {
        /// Selection order has no gameplay effect; resolution uses ascending IDs.
        cards: Vec<CardId>,
    },
    /// Commit zero or one card during a later round's regroup.
    ChooseRegroupResource {
        /// `None` declines resourcing.
        card: Option<CardId>,
    },
    /// Pay the exact cost and deploy a unit exhausted.
    PlayUnit {
        /// Card in the acting player's hand.
        card: CardId,
    },
    /// Exhaust a ready attacker and resolve simultaneous combat damage.
    Attack {
        /// Unit on the acting player's battlefield.
        attacker: CardId,
        /// Enemy unit or stronghold.
        target: AttackTarget,
    },
    /// Temporarily decline to act; later actions in this round remain possible.
    Pass,
    /// Take next-round initiative and permanently stop acting this round.
    ClaimInitiative,
}

/// Caller-maintained ordered command log entry for deterministic replay.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoggedCommand {
    /// Trusted logical player identity; authentication belongs to the caller.
    pub player: PlayerId,
    /// Decision to validate and execute.
    pub command: Command,
}
