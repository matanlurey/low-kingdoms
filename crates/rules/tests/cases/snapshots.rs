//! Exact continuation, replay, compatibility, malformed inputs, and counter boundaries.

use low_kingdoms_rules::command::{AttackTarget, Command, LoggedCommand};
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::error::Error;
use low_kingdoms_rules::model::{Outcome, Phase};

use crate::{A, B, Fixture, deploy, end_round, keep, regroup, submit};

fn assert_restores(game: &Game, fixture: &Fixture) {
    let encoded = game.snapshot().expect("snapshot");
    let restored = Game::restore(&encoded, fixture.definitions.clone()).expect("restore");
    assert_eq!(encoded, restored.snapshot().expect("snapshot"));
    for player in [A, B] {
        assert_eq!(game.view_for(player), restored.view_for(player));
        assert_eq!(
            game.legal_commands_for(player),
            restored.legal_commands_for(player)
        );
    }
}

#[test]
fn snapshots_cover_pending_setup_resources_regroup_claims_damage_and_terminal_state() {
    let mut fixture = Fixture::new(0, 1, 5);
    fixture.config.players[1].stronghold_health = 1;
    let mut game = fixture.game();
    assert_restores(&game, &fixture);
    submit(&mut game, B, &Command::MulliganHand);
    assert_restores(&game, &fixture);
    submit(&mut game, A, &Command::KeepHand);
    let card = game.view_for(A).hand[0].id;
    submit(
        &mut game,
        A,
        &Command::ChooseInitialResources { cards: vec![card] },
    );
    assert_restores(&game, &fixture);
    submit(
        &mut game,
        B,
        &Command::ChooseInitialResources { cards: vec![] },
    );
    let attacker = deploy(&mut game, A);
    let defender = deploy(&mut game, B);
    submit(&mut game, A, &Command::ClaimInitiative);
    assert_restores(&game, &fixture);
    end_round(&mut game);
    submit(&mut game, B, &Command::ChooseRegroupResource { card: None });
    assert_restores(&game, &fixture);
    submit(&mut game, A, &Command::ChooseRegroupResource { card: None });
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Unit(defender),
        },
    );
    assert_restores(&game, &fixture);
    end_round(&mut game);
    regroup(&mut game);
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Stronghold(B),
        },
    );
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Winner(A)));
    assert_restores(&game, &fixture);
    let mut restored =
        Game::restore(&game.snapshot().expect("snapshot"), fixture.definitions).expect("restore");
    assert_eq!(
        restored.apply(A, &Command::Pass),
        Err(Error::GameAlreadyFinished)
    );
}

#[test]
fn pending_random_decision_restores_to_identical_future_events_and_outcome() {
    let mut fixture = Fixture::new(0, 3, 2);
    fixture.config.players[1].stronghold_health = 3;
    let mut uninterrupted = fixture.game();
    submit(&mut uninterrupted, B, &Command::MulliganHand);
    let mut restored = Game::restore(
        &uninterrupted.snapshot().expect("snapshot"),
        fixture.definitions,
    )
    .expect("restore");
    let steps = vec![
        (A, Command::MulliganHand),
        (A, Command::ChooseInitialResources { cards: vec![] }),
        (B, Command::ChooseInitialResources { cards: vec![] }),
    ];
    for (player, command) in steps {
        assert_eq!(
            uninterrupted.apply(player, &command),
            restored.apply(player, &command)
        );
    }
    let attacker = uninterrupted.view_for(A).hand[0].id;
    for (player, command) in [
        (A, Command::PlayUnit { card: attacker }),
        (B, Command::Pass),
        (A, Command::Pass),
        (B, Command::ChooseRegroupResource { card: None }),
        (A, Command::ChooseRegroupResource { card: None }),
        (
            A,
            Command::Attack {
                attacker,
                target: AttackTarget::Stronghold(B),
            },
        ),
    ] {
        let left = uninterrupted
            .apply(player, &command)
            .expect("legal continuation");
        let right = restored
            .apply(player, &command)
            .expect("legal continuation");
        for viewer in [A, B] {
            assert_eq!(left.for_player(viewer), right.for_player(viewer));
        }
        assert_eq!(
            uninterrupted.snapshot().expect("snapshot"),
            restored.snapshot().expect("snapshot")
        );
    }
    assert_eq!(restored.view_for(A).outcome, Some(Outcome::Winner(A)));
}

#[test]
fn replay_matches_configuration_command_log_events_and_terminal_snapshot() {
    for seed in [0, 1, 42, u64::MAX] {
        for starting in [None, Some(A), Some(B)] {
            let mut fixture = Fixture::new(0, 2, 3);
            fixture.config.seed = seed;
            fixture.config.starting_player = starting;
            for player in &mut fixture.config.players {
                player.deck.truncate(6);
                player.stronghold_health = 7;
            }
            let (mut original, first) =
                Game::new(fixture.config.clone(), fixture.definitions.clone()).expect("fixture");
            let mut events = vec![first];
            let mut log = vec![];
            for (player, command) in [
                (B, Command::MulliganHand),
                (A, Command::KeepHand),
                (A, Command::ChooseInitialResources { cards: vec![] }),
                (B, Command::ChooseInitialResources { cards: vec![] }),
            ] {
                events.push(original.apply(player, &command).expect("setup"));
                log.push(LoggedCommand { player, command });
            }
            while original.view_for(A).phase != Phase::Finished {
                let view = original.view_for(A);
                let (player, command) = if view.phase == Phase::Action {
                    (view.active_player.expect("active"), Command::Pass)
                } else if !view.decision_submitted {
                    (A, Command::ChooseRegroupResource { card: None })
                } else {
                    (B, Command::ChooseRegroupResource { card: None })
                };
                events.push(original.apply(player, &command).expect("legal"));
                log.push(LoggedCommand { player, command });
            }
            let decoded_log: Vec<LoggedCommand> =
                serde_json::from_str(&serde_json::to_string(&log).expect("log"))
                    .expect("log round trip");
            let (replayed, replay_events) = Game::replay(
                fixture.config.clone(),
                fixture.definitions.clone(),
                &decoded_log,
            )
            .expect("replay");
            assert_eq!(
                original.snapshot().expect("snapshot"),
                replayed.snapshot().expect("snapshot")
            );
            assert_eq!(events, replay_events);
            assert_eq!(original.view_for(A).outcome, Some(Outcome::Draw));
            log.push(LoggedCommand {
                player: A,
                command: Command::Pass,
            });
            assert!(matches!(
                Game::replay(fixture.config, fixture.definitions, &log),
                Err(Error::GameAlreadyFinished)
            ));
        }
    }
}

#[test]
fn snapshot_version_catalog_and_malformed_invariants_are_rejected() {
    let fixture = Fixture::new(1, 2, 3);
    let game = fixture.game();
    let encoded = game.snapshot().expect("snapshot");
    let original: serde_json::Value = serde_json::from_str(&encoded).expect("json");
    let mut changed = fixture.definitions.clone();
    changed[0].power += 1;
    assert!(matches!(
        Game::restore(&encoded, changed),
        Err(Error::SnapshotDefinitions)
    ));
    assert!(matches!(
        Game::restore("invalid", fixture.definitions.clone()),
        Err(Error::SnapshotFormat)
    ));
    let mut version = original.clone();
    version["version"] = serde_json::json!(2);
    assert!(matches!(
        Game::restore(&version.to_string(), fixture.definitions.clone()),
        Err(Error::SnapshotVersion)
    ));
    for mutate in [
        |value: &mut serde_json::Value| value["state"]["next_id"] = serde_json::json!(1),
        |value: &mut serde_json::Value| value["state"]["round"] = serde_json::json!(0),
        |value: &mut serde_json::Value| value["state"]["passes"] = serde_json::json!(2),
        |value: &mut serde_json::Value| value["state"]["active"] = serde_json::json!("Zero"),
        |value: &mut serde_json::Value| {
            value["state"]["pending"][0] = serde_json::json!({"Resources":[]});
        },
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["hand"][0] =
                value["state"]["players"][0]["hand"][1].clone();
        },
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["hand"][0]["definition"] = serde_json::json!(999);
        },
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["stronghold_damage"] = serde_json::json!(u64::MAX);
        },
    ] {
        let mut value = original.clone();
        mutate(&mut value);
        assert!(matches!(
            Game::restore(&value.to_string(), fixture.definitions.clone()),
            Err(Error::SnapshotInvariant)
        ));
    }
    let mut unknown = original;
    unknown["state"]["extra"] = serde_json::json!(true);
    assert!(matches!(
        Game::restore(&unknown.to_string(), fixture.definitions),
        Err(Error::SnapshotFormat)
    ));
}

#[test]
fn restored_battlefield_and_decision_corruption_cannot_break_command_assumptions() {
    let fixture = Fixture::new(1, 2, 3);
    let mut game = fixture.start([2, 2]);
    deploy(&mut game, A);
    let original: serde_json::Value =
        serde_json::from_str(&game.snapshot().expect("snapshot")).expect("json");
    for mutate in [
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["units"][0]["health"] = serde_json::json!(4);
        },
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["units"][0]["damage"] = serde_json::json!(3);
        },
        |value: &mut serde_json::Value| value["state"]["claimant"] = serde_json::json!("One"),
        |value: &mut serde_json::Value| {
            value["state"]["pending"][0] = serde_json::json!({"Resources":[]});
        },
        |value: &mut serde_json::Value| {
            value["state"]["players"][0]["resources"]
                .as_array_mut()
                .expect("resources")
                .reverse();
        },
        |value: &mut serde_json::Value| {
            value["state"]["outcome"] = serde_json::json!({"Winner":"Zero"});
        },
    ] {
        let mut value = original.clone();
        mutate(&mut value);
        assert!(matches!(
            Game::restore(&value.to_string(), fixture.definitions.clone()),
            Err(Error::SnapshotInvariant)
        ));
    }
    let mut pending = fixture.game();
    keep(&mut pending);
    let mut value: serde_json::Value =
        serde_json::from_str(&pending.snapshot().expect("snapshot")).expect("json");
    value["state"]["pending"][0] = serde_json::json!({"Resources":[999]});
    assert!(matches!(
        Game::restore(&value.to_string(), fixture.definitions),
        Err(Error::SnapshotInvariant)
    ));
}

#[test]
fn round_counter_overflow_rejects_only_the_ending_command_atomically() {
    let fixture = Fixture::new(0, 0, 1);
    let game = fixture.start([0, 0]);
    let mut value: serde_json::Value =
        serde_json::from_str(&game.snapshot().expect("snapshot")).expect("json");
    value["state"]["round"] = serde_json::json!(u64::MAX);
    let mut restored =
        Game::restore(&value.to_string(), fixture.definitions).expect("representable state");
    submit(&mut restored, A, &Command::Pass);
    let before = restored.snapshot().expect("snapshot");
    assert_eq!(
        restored.apply(B, &Command::Pass),
        Err(Error::ArithmeticLimit)
    );
    assert_eq!(restored.snapshot().expect("snapshot"), before);
    assert!(!restored.legal_commands_for(B).contains(&Command::Pass));
    assert!(
        restored
            .legal_commands_for(B)
            .contains(&Command::ClaimInitiative)
    );
}
