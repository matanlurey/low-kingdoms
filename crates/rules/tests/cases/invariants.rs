//! Invariants over varied deterministic legal command walks, without an AI or test framework.

use std::collections::BTreeSet;

use low_kingdoms_rules::command::Command;
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::error::Error;
use low_kingdoms_rules::model::{DefinitionId, Phase, PlayerId, UnitDefinition};

use crate::{A, Fixture};

fn assert_conservation(game: &Game, expected: usize) {
    let snapshot: serde_json::Value =
        serde_json::from_str(&game.snapshot().expect("snapshot")).expect("json");
    let mut ids = BTreeSet::new();
    for player in snapshot["state"]["players"].as_array().expect("players") {
        for zone in ["deck", "hand", "discard", "resources", "units"] {
            for entry in player[zone].as_array().expect("zone") {
                let card = if matches!(zone, "resources" | "units") {
                    &entry["card"]
                } else {
                    entry
                };
                let id = card["id"].as_u64().expect("ID");
                assert!(id > 0);
                assert!(ids.insert(id), "instance occurs in more than one zone");
            }
        }
        for unit in player["units"].as_array().expect("units") {
            assert!(
                unit["damage"].as_u64().expect("damage") < unit["health"].as_u64().expect("health")
            );
        }
    }
    assert_eq!(ids.len(), expected);
    assert_eq!(
        ids,
        (1..=u64::try_from(expected).expect("length")).collect()
    );
    let view = game.view_for(A);
    if let Some(active) = view.active_player {
        assert_eq!(view.phase, Phase::Action);
        assert_ne!(view.initiative_claimant, Some(active));
    }
}

#[test]
fn varied_legal_walks_preserve_identity_conservation_atomicity_and_restore() {
    for seed in 0_u64..32 {
        let mut fixture = Fixture::new(0, 1, 3);
        fixture.config.seed = seed;
        fixture.config.starting_player = None;
        fixture.definitions.push(UnitDefinition {
            id: DefinitionId(2),
            cost: 2,
            power: 5,
            health: 2,
        });
        for player in &mut fixture.config.players {
            for index in (0..player.deck.len()).step_by(3) {
                player.deck[index] = DefinitionId(2);
            }
        }
        let mut game = fixture.game();
        let mut chooser = seed + 1;
        for _ in 0..500 {
            assert_conservation(&game, 40);
            let before = game.snapshot().expect("snapshot");
            let restored =
                Game::restore(&before, fixture.definitions.clone()).expect("invariants restore");
            assert_eq!(restored.snapshot().expect("snapshot"), before);
            let view = game.view_for(A);
            if view.phase == Phase::Finished {
                for player in PlayerId::ALL {
                    assert!(game.legal_commands_for(player).is_empty());
                    assert_eq!(
                        game.apply(player, &Command::Pass),
                        Err(Error::GameAlreadyFinished)
                    );
                }
                break;
            }
            let player = view.active_player.unwrap_or_else(|| {
                if game.legal_commands_for(A).is_empty() {
                    A.opponent()
                } else {
                    A
                }
            });
            let legal = game.legal_commands_for(player);
            assert!(!legal.is_empty());
            chooser = chooser
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let index = usize::try_from(chooser % u64::try_from(legal.len()).expect("length"))
                .expect("index");
            let command = &legal[index];
            let mut restored = restored;
            let events = game
                .apply(player, command)
                .expect("discovered legal command");
            assert_eq!(
                events,
                restored
                    .apply(player, command)
                    .expect("restored legal command")
            );
            assert_eq!(
                game.snapshot().expect("snapshot"),
                restored.snapshot().expect("snapshot")
            );
            assert_conservation(&game, 40);
        }
        assert_eq!(
            game.view_for(A).phase,
            Phase::Finished,
            "bounded walk completes"
        );
    }
}
