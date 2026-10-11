//! Complete scripted matches using only the public headless engine API.
#![cfg(test)]

use crate::{A, B, Fixture, deploy, end_round, regroup, submit};
use low_kingdoms_rules::command::{AttackTarget, Command};
use low_kingdoms_rules::error::Error;
use low_kingdoms_rules::event::Event;
use low_kingdoms_rules::model::{Outcome, Phase};

#[test]
fn complete_combat_victory() {
    let mut fixture = Fixture::new(1, 3, 4);
    for player in &mut fixture.config.players {
        player.stronghold_health = 6;
    }
    let mut game = fixture.start([2, 2]);
    let first_a = deploy(&mut game, A);
    let first_b = deploy(&mut game, B);
    end_round(&mut game);
    for player in [A, B] {
        let card = game.view_for(player).hand[0].id;
        submit(
            &mut game,
            player,
            &Command::ChooseRegroupResource { card: Some(card) },
        );
    }
    assert_eq!(
        game.view_for(A)
            .resources
            .iter()
            .filter(|resource| resource.ready)
            .count(),
        3
    );
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker: first_a,
            target: AttackTarget::Unit(first_b),
        },
    );
    let view = game.view_for(A);
    assert_eq!(view.players[0].units[0].damage, 3);
    assert_eq!(view.players[1].units[0].damage, 3);
    submit(
        &mut game,
        B,
        &Command::Attack {
            attacker: first_b,
            target: AttackTarget::Unit(first_a),
        },
    );
    assert_eq!(game.view_for(A).players[0].discard[0].id, first_a);
    assert_eq!(game.view_for(A).players[1].discard[0].id, first_b);
    let finisher = deploy(&mut game, A);
    deploy(&mut game, B);
    end_round(&mut game);
    regroup(&mut game);
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker: finisher,
            target: AttackTarget::Stronghold(B),
        },
    );
    assert_eq!(game.view_for(A).players[1].stronghold_damage, 3);
    assert_eq!(game.view_for(A).players[0].units[0].damage, 0);
    end_round(&mut game);
    regroup(&mut game);
    let events = submit(
        &mut game,
        A,
        &Command::Attack {
            attacker: finisher,
            target: AttackTarget::Stronghold(B),
        },
    );
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Winner(A)));
    assert_eq!(
        events.for_player(A).last(),
        Some(&Event::GameEnded {
            outcome: Outcome::Winner(A)
        })
    );
    assert_eq!(
        game.apply(B, &Command::Pass),
        Err(Error::GameAlreadyFinished)
    );
    assert!(game.legal_commands_for(A).is_empty());
}

#[test]
fn complete_deck_exhaustion_victory() {
    let mut fixture = Fixture::new(1, 1, 1);
    fixture.config.players[0].deck.truncate(6);
    fixture.config.players[0].stronghold_health = 7;
    let mut game = fixture.start([0, 0]);
    assert_eq!(game.view_for(A).players[0].deck_size, 0);
    assert_eq!(game.view_for(A).outcome, None);
    end_round(&mut game);
    assert_eq!(game.view_for(A).round, 2);
    assert_eq!(game.view_for(A).players[0].stronghold_damage, 6);
    assert_eq!(game.view_for(A).outcome, None);
    regroup(&mut game);
    let events = end_round(&mut game);
    assert_eq!(game.view_for(A).phase, Phase::Finished);
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Winner(B)));
    assert_eq!(game.view_for(A).players[1].hand_size, 10);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CardsDrawn {
            player: B,
            count: 2,
            ..
        }
    )));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ResourcesReadied { .. }))
    );
}

#[test]
fn complete_initiative_and_passing_match() {
    let mut fixture = Fixture::new(0, 4, 2);
    fixture.config.players[0].stronghold_health = 4;
    let mut game = fixture.start([1, 1]);
    submit(&mut game, A, &Command::Pass);
    let attacker = deploy(&mut game, B);
    assert_eq!(game.view_for(A).active_player, Some(A));
    assert_eq!(game.view_for(A).consecutive_passes, 0);
    submit(&mut game, A, &Command::ClaimInitiative);
    assert_eq!(game.view_for(A).initiative_claimant, Some(A));
    assert_eq!(game.apply(A, &Command::Pass), Err(Error::NotYourAction));
    assert_eq!(
        game.apply(B, &Command::ClaimInitiative),
        Err(Error::InitiativeUnavailable)
    );
    deploy(&mut game, B);
    deploy(&mut game, B);
    assert_eq!(game.view_for(A).active_player, Some(B));
    end_round(&mut game);
    regroup(&mut game);
    assert_eq!(game.view_for(A).active_player, Some(A));
    assert_eq!(game.view_for(A).initiative_claimant, None);
    submit(&mut game, A, &Command::Pass);
    submit(
        &mut game,
        B,
        &Command::Attack {
            attacker,
            target: AttackTarget::Stronghold(A),
        },
    );
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Winner(B)));
}
