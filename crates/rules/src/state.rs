//! Private authoritative match state; only engine operations mutate it.

use serde::{Deserialize, Serialize};

use crate::model::{Card, Outcome, Phase, PlayerId};
use crate::rng::Rng;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct Resource {
    pub(crate) card: Card,
    pub(crate) ready: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct Unit {
    pub(crate) card: Card,
    pub(crate) power: u32,
    pub(crate) health: u32,
    pub(crate) damage: u64,
    pub(crate) ready: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct PlayerState {
    // The top of the deck is the final element. Neither player-facing projection exposes it.
    pub(crate) deck: Vec<Card>,
    pub(crate) hand: Vec<Card>,
    pub(crate) resources: Vec<Resource>,
    pub(crate) units: Vec<Unit>,
    pub(crate) discard: Vec<Card>,
    pub(crate) stronghold_damage: u64,
}

impl PlayerState {
    pub(crate) const fn new(deck: Vec<Card>) -> Self {
        Self {
            deck,
            hand: Vec::new(),
            resources: Vec::new(),
            units: Vec::new(),
            discard: Vec::new(),
            stronghold_damage: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) enum Decision {
    Mulligan(bool),
    Resources(Vec<crate::model::CardId>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private authoritative types stay crate-local; pub conflicts with unreachable_pub."
)]
pub(crate) struct State {
    pub(crate) players: [PlayerState; 2],
    pub(crate) phase: Phase,
    pub(crate) round: u64,
    pub(crate) initiative: PlayerId,
    pub(crate) claimant: Option<PlayerId>,
    pub(crate) active: Option<PlayerId>,
    pub(crate) passes: u8,
    pub(crate) pending: [Option<Decision>; 2],
    pub(crate) rng: Rng,
    pub(crate) next_id: u64,
    pub(crate) outcome: Option<Outcome>,
}
