//! Embedded, validated RON prototype cards and decks shared by native and browser matches.

use std::collections::BTreeSet;
use std::fmt;

use low_kingdoms_rules::model::{DefinitionId, MatchConfig, PlayerConfig, UnitDefinition};
use serde::{Deserialize, Serialize};

const CARDS: &str = include_str!("../../../assets/content/cards.ron");
const DECKS: &str = include_str!("../../../assets/content/decks.ron");

/// One named vanilla unit; its immutable statistics are passed directly to the rules engine.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CardDefinition {
    /// Stable identity referenced by decks and rules-engine instances.
    pub id: DefinitionId,
    /// Readable prototype name without abilities or rules text.
    pub name: String,
    /// Exact deployment resource cost, including legal zero-cost units.
    pub cost: u32,
    /// Printed combat Power.
    pub power: u32,
    /// Positive printed maximum Health.
    pub health: u32,
}

impl CardDefinition {
    /// Returns the renderer-independent definition without presentation text.
    #[must_use]
    pub const fn unit_definition(&self) -> UnitDefinition {
        UnitDefinition {
            id: self.id,
            cost: self.cost,
            power: self.power,
            health: self.health,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Deck {
    cards: Vec<DefinitionId>,
    stronghold_health: u32,
}

/// A parsed, validated catalog and exactly two twenty-card prototype decks.
#[derive(Clone, Debug)]
pub struct Content {
    cards: Vec<CardDefinition>,
    decks: [PlayerConfig; 2],
    signature: String,
}

impl Content {
    /// Loads the same compile-time embedded RON files on native and WASM targets.
    ///
    /// # Errors
    /// Returns a diagnostic if the embedded catalog or decks fail parsing or validation.
    pub fn load() -> Result<Self, ContentError> {
        Self::parse(CARDS, DECKS)
    }

    /// Parses definitions and decks, then validates all references before creating match inputs.
    ///
    /// Definition order is canonicalized by ID. Deck order remains meaningful because the rules
    /// engine deterministically shuffles its caller-provided order. Costs and Power use the rules
    /// engine's complete `u32` domain; maximum Health must be positive.
    ///
    /// # Errors
    /// Rejects malformed RON, unknown fields, empty catalogs or names, duplicate IDs, zero Health,
    /// any deck count other than two, any deck length other than twenty, unknown card references,
    /// and zero stronghold Health. Also reports a failure to encode the compatibility signature.
    pub fn parse(cards_ron: &str, decks_ron: &str) -> Result<Self, ContentError> {
        let mut cards: Vec<CardDefinition> =
            ron::from_str(cards_ron).map_err(|error| ContentError::Syntax {
                source: "cards.ron",
                message: error.to_string(),
            })?;
        let decks: Vec<Deck> = ron::from_str(decks_ron).map_err(|error| ContentError::Syntax {
            source: "decks.ron",
            message: error.to_string(),
        })?;
        if cards.is_empty() {
            return Err(ContentError::EmptyCatalog);
        }
        cards.sort_by_key(|card| card.id);
        let mut identities = BTreeSet::new();
        for card in &cards {
            if !identities.insert(card.id) {
                return Err(ContentError::DuplicateDefinition(card.id));
            }
            if card.name.trim().is_empty() {
                return Err(ContentError::EmptyName(card.id));
            }
            if card.health == 0 {
                return Err(ContentError::InvalidHealth(card.id));
            }
        }
        if decks.len() != 2 {
            return Err(ContentError::DeckCount(decks.len()));
        }
        for (player, deck) in decks.iter().enumerate() {
            if deck.cards.len() != 20 {
                return Err(ContentError::DeckLength {
                    player,
                    actual: deck.cards.len(),
                });
            }
            if deck.stronghold_health == 0 {
                return Err(ContentError::InvalidStronghold(player));
            }
            for definition in &deck.cards {
                if !identities.contains(definition) {
                    return Err(ContentError::UnknownDefinition {
                        player,
                        definition: *definition,
                    });
                }
            }
        }
        let decks: [PlayerConfig; 2] = decks
            .into_iter()
            .map(|deck| PlayerConfig {
                deck: deck.cards,
                stronghold_health: deck.stronghold_health,
            })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|decks: Vec<PlayerConfig>| ContentError::DeckCount(decks.len()))?;
        let signature = serde_json::to_string(&(&cards, &decks))
            .map_err(|error| ContentError::Signature(error.to_string()))?;
        Ok(Self {
            cards,
            decks,
            signature,
        })
    }

    /// Returns the canonical ID-ordered public definition catalog, without deck contents.
    #[must_use]
    pub fn cards(&self) -> &[CardDefinition] {
        &self.cards
    }

    /// Builds fresh authoritative inputs using an explicit seed and seeded opening initiative.
    #[must_use]
    pub fn config(&self, seed: u64) -> MatchConfig {
        MatchConfig {
            players: self.decks.clone(),
            seed,
            starting_player: None,
        }
    }

    /// Returns an exact canonical compatibility string for names, statistics, decks, and Health.
    ///
    /// This is canonical JSON of the ID-sorted definitions and ordered player configurations,
    /// rather than a collision-prone hash. Whitespace and definition ordering in RON do not change
    /// it; changing a name, statistic, reference, deck order, or stronghold Health does.
    #[must_use]
    pub fn signature(&self) -> &str {
        &self.signature
    }
}

/// Actionable catalog/deck diagnostics; no partial match is created when loading fails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentError {
    /// RON syntax, type, missing-field, or unknown-field error, including source location.
    Syntax {
        /// Embedded logical file that failed parsing.
        source: &'static str,
        /// RON's line/column diagnostic.
        message: String,
    },
    /// No unit definitions were supplied.
    EmptyCatalog,
    /// More than one definition uses this ID.
    DuplicateDefinition(DefinitionId),
    /// This unit name is empty or contains only whitespace.
    EmptyName(DefinitionId),
    /// This unit has zero maximum Health.
    InvalidHealth(DefinitionId),
    /// The supplied number of decks differs from two.
    DeckCount(usize),
    /// One deck does not contain exactly twenty cards.
    DeckLength {
        /// Zero-based player index.
        player: usize,
        /// Supplied card count.
        actual: usize,
    },
    /// A deck references a definition absent from the catalog.
    UnknownDefinition {
        /// Zero-based player index.
        player: usize,
        /// Unknown static identity.
        definition: DefinitionId,
    },
    /// This zero-based player has zero stronghold Health.
    InvalidStronghold(usize),
    /// Canonical compatibility data could not be serialized.
    Signature(String),
}

impl fmt::Display for ContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { source, message } => write!(formatter, "{source}: {message}"),
            Self::EmptyCatalog => formatter.write_str("cards.ron must contain at least one unit"),
            Self::DuplicateDefinition(id) => {
                write!(formatter, "duplicate card definition {}", id.0)
            }
            Self::EmptyName(id) => write!(formatter, "card {} needs a nonempty name", id.0),
            Self::InvalidHealth(id) => write!(formatter, "card {} needs positive Health", id.0),
            Self::DeckCount(actual) => {
                write!(
                    formatter,
                    "decks.ron needs exactly two decks, found {actual}"
                )
            }
            Self::DeckLength { player, actual } => {
                write!(
                    formatter,
                    "player {player} deck needs twenty cards, found {actual}"
                )
            }
            Self::UnknownDefinition { player, definition } => {
                write!(
                    formatter,
                    "player {player} deck references unknown card {}",
                    definition.0
                )
            }
            Self::InvalidStronghold(player) => {
                write!(
                    formatter,
                    "player {player} stronghold needs positive Health"
                )
            }
            Self::Signature(message) => {
                write!(formatter, "cannot encode content identity: {message}")
            }
        }
    }
}

impl std::error::Error for ContentError {}
