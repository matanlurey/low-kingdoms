//! Structured errors that never disclose inaccessible card information.

use std::fmt;

use crate::model::DefinitionId;

/// Initialization, command validation, or privileged snapshot failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A deck is too short or a stronghold has zero Health.
    InvalidConfiguration,
    /// A catalog ID occurs more than once.
    DuplicateDefinition(DefinitionId),
    /// A unit definition has zero Health.
    InvalidDefinition(DefinitionId),
    /// A configured deck references a missing catalog definition.
    UnknownDefinition(DefinitionId),
    /// Command category is unavailable in the current phase.
    WrongPhase,
    /// It is another player's action, or this player has claimed initiative.
    NotYourAction,
    /// A card is absent from the required owned zone, regardless of other zones.
    InvalidCard,
    /// A resourcing selection repeats an instance ID.
    DuplicateResource,
    /// More cards were selected than this phase permits.
    TooManyResources,
    /// Ready resources cannot cover this owned card's cost.
    InsufficientResources,
    /// The owned battlefield attacker is exhausted.
    UnitExhausted,
    /// Target is absent from the opposing battlefield or is a friendly stronghold.
    InvalidTarget,
    /// This player already submitted their private decision for this phase.
    DecisionAlreadySubmitted,
    /// Another player already claimed initiative this round.
    InitiativeUnavailable,
    /// No commands are accepted after a terminal result.
    GameAlreadyFinished,
    /// A representational counter cannot advance without overflow.
    ArithmeticLimit,
    /// Authoritative JSON is malformed or cannot be encoded.
    SnapshotFormat,
    /// Snapshot version differs from the current engine format.
    SnapshotVersion,
    /// Snapshot catalog differs from the supplied definitions.
    SnapshotDefinitions,
    /// Snapshot violates card conservation, phase, or domain invariants.
    SnapshotInvariant,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for Error {}
