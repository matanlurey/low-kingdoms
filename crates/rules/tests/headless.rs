//! Public-API headless rules acceptance suite.
#![cfg(test)]

#[path = "cases/invariants.rs"]
mod invariants;
#[path = "cases/matches.rs"]
mod matches;
#[path = "cases/privacy.rs"]
mod privacy;
#[path = "cases/rules.rs"]
mod rules;
#[path = "cases/snapshots.rs"]
mod snapshots;

use low_kingdoms_rules::command::Command;
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::event::{Event, EventBatch};
use low_kingdoms_rules::model::{
    CardId, DefinitionId, MatchConfig, Phase, PlayerConfig, PlayerId, UnitDefinition,
};

const A: PlayerId = PlayerId::Zero;
const B: PlayerId = PlayerId::One;

#[derive(Clone)]
struct Fixture {
    config: MatchConfig,
    definitions: Vec<UnitDefinition>,
}

impl Fixture {
    fn new(cost: u32, power: u32, health: u32) -> Self {
        Self {
            config: MatchConfig {
                players: std::array::from_fn(|_| PlayerConfig {
                    deck: vec![DefinitionId(1); 20],
                    stronghold_health: 12,
                }),
                seed: 42,
                starting_player: Some(A),
            },
            definitions: vec![UnitDefinition {
                id: DefinitionId(1),
                cost,
                power,
                health,
            }],
        }
    }

    fn game(&self) -> Game {
        Game::new(self.config.clone(), self.definitions.clone())
            .expect("valid fixture")
            .0
    }

    fn start(&self, resources: [usize; 2]) -> Game {
        let mut game = self.game();
        keep(&mut game);
        for (player, count) in PlayerId::ALL.into_iter().zip(resources) {
            let cards = game
                .view_for(player)
                .hand
                .iter()
                .take(count)
                .map(|card| card.id)
                .collect();
            submit(
                &mut game,
                player,
                &Command::ChooseInitialResources { cards },
            );
        }
        assert_eq!(game.view_for(A).phase, Phase::Action);
        game
    }
}

fn submit(game: &mut Game, player: PlayerId, command: &Command) -> EventBatch {
    game.apply(player, command).expect("scripted legal command")
}

fn keep(game: &mut Game) {
    assert!(submit(game, A, &Command::KeepHand).for_player(A).is_empty());
    submit(game, B, &Command::KeepHand);
}

fn deploy(game: &mut Game, player: PlayerId) -> CardId {
    let card = game.view_for(player).hand[0].id;
    submit(game, player, &Command::PlayUnit { card });
    card
}

fn end_round(game: &mut Game) -> Vec<Event> {
    let player = game.view_for(A).active_player.expect("action phase");
    let mut events = submit(game, player, &Command::Pass).for_player(A);
    if game.view_for(A).phase == Phase::Action {
        events.extend(submit(game, player.opponent(), &Command::Pass).for_player(A));
    }
    assert!(matches!(
        game.view_for(A).phase,
        Phase::Regroup | Phase::Finished
    ));
    events
}

fn regroup(game: &mut Game) {
    assert_eq!(game.view_for(A).phase, Phase::Regroup);
    submit(game, A, &Command::ChooseRegroupResource { card: None });
    submit(game, B, &Command::ChooseRegroupResource { card: None });
}
