//! Headless two-player Low Kingdoms matches.
//!
//! Use [`engine::Game`] with injected [`model::UnitDefinition`] values and a
//! [`model::MatchConfig`]. Player-facing code reads [`view::PlayerView`] and filtered
//! [`event::EventBatch`] values. Authoritative JSON snapshots are explicitly privileged.
//!
//! ```
//! use low_kingdoms_rules::command::Command;
//! use low_kingdoms_rules::engine::Game;
//! use low_kingdoms_rules::model::{DefinitionId, MatchConfig, PlayerConfig, PlayerId, UnitDefinition};
//!
//! let definition = UnitDefinition { id: DefinitionId(1), cost: 1, power: 2, health: 3 };
//! let config = MatchConfig {
//!     players: std::array::from_fn(|_| PlayerConfig {
//!         deck: vec![definition.id; 20], stronghold_health: 30,
//!     }),
//!     seed: 42,
//!     starting_player: Some(PlayerId::Zero),
//! };
//! let (mut game, initialization) = Game::new(config, vec![definition])?;
//! let opening_hand = game.view_for(PlayerId::Zero).hand;
//! assert_eq!(opening_hand.len(), 6);
//! let own_events = initialization.for_player(PlayerId::Zero);
//! game.apply(PlayerId::Zero, &Command::KeepHand)?;
//! game.apply(PlayerId::One, &Command::KeepHand)?;
//! let choices = game.legal_commands_for(PlayerId::Zero);
//! assert!(choices.contains(&Command::ChooseInitialResources { cards: vec![] }));
//! # Ok::<(), low_kingdoms_rules::error::Error>(())
//! ```

pub mod command;
pub mod engine;
pub mod error;
pub mod event;
pub mod model;
pub mod view;

mod rng;
mod state;
