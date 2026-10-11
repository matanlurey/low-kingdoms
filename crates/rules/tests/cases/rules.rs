//! Setup, action economy, payment, combat, fatigue, and arithmetic boundaries.
#![cfg(test)]

use crate::{A, B, Fixture, deploy, end_round, keep, regroup, submit};
use low_kingdoms_rules::command::{AttackTarget, Command};
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::error::Error;
use low_kingdoms_rules::event::Event;
use low_kingdoms_rules::model::{CardId, DefinitionId, Outcome, Phase, PlayerId, UnitDefinition};

fn reject(game: &mut Game, player: PlayerId, command: &Command, error: Error) {
    let before = game.snapshot().expect("snapshot");
    assert_eq!(game.apply(player, command), Err(error));
    assert_eq!(game.snapshot().expect("snapshot"), before);
}

#[test]
fn configuration_validation_and_seeded_setup() {
    let mut fixture = Fixture::new(0, 0, 1);
    fixture.config.starting_player = None;
    let first = fixture.game();
    assert_eq!(
        first.snapshot().expect("snapshot"),
        fixture.game().snapshot().expect("snapshot")
    );
    for player in [A, B] {
        let view = first.view_for(player);
        assert_eq!(view.phase, Phase::Mulligan);
        assert_eq!(view.hand.len(), 6);
        assert_eq!(view.players[0].deck_size, 14);
        assert!(view.active_player.is_none());
    }
    let mut starts = Vec::new();
    for seed in 0..32 {
        fixture.config.seed = seed;
        starts.push(fixture.game().view_for(A).initiative);
    }
    assert!(starts.contains(&A) && starts.contains(&B));
    fixture.config.starting_player = Some(B);
    assert_eq!(fixture.game().view_for(A).initiative, B);
    fixture.config.players[0].deck.truncate(5);
    assert!(matches!(
        Game::new(fixture.config.clone(), fixture.definitions.clone()),
        Err(Error::InvalidConfiguration)
    ));
    fixture.config.players[0].deck.push(DefinitionId(999));
    assert!(matches!(
        Game::new(fixture.config.clone(), fixture.definitions.clone()),
        Err(Error::UnknownDefinition(DefinitionId(999)))
    ));
    fixture.config.players[0].deck[5] = DefinitionId(1);
    fixture.config.players[1].stronghold_health = 0;
    assert!(matches!(
        Game::new(fixture.config.clone(), fixture.definitions.clone()),
        Err(Error::InvalidConfiguration)
    ));
    fixture.config.players[1].stronghold_health = 1;
    fixture.definitions[0].health = 0;
    assert!(matches!(
        Game::new(fixture.config.clone(), fixture.definitions.clone()),
        Err(Error::InvalidDefinition(DefinitionId(1)))
    ));
    fixture.definitions[0].health = 1;
    fixture.definitions.push(fixture.definitions[0]);
    assert!(matches!(
        Game::new(fixture.config, fixture.definitions),
        Err(Error::DuplicateDefinition(DefinitionId(1)))
    ));
    assert!(serde_json::from_str::<PlayerId>("2").is_err());
}

#[test]
fn buffered_mulligans_and_resources_ignore_arrival_and_selection_order() {
    let fixture = Fixture::new(1, 2, 3);
    let mut left = fixture.game();
    let mut right = fixture.game();
    let original = left.view_for(A).hand;
    assert!(
        submit(&mut left, A, &Command::MulliganHand)
            .for_player(A)
            .is_empty()
    );
    assert_eq!(left.view_for(A).hand, original);
    assert_eq!(left.view_for(B), right.view_for(B));
    reject(
        &mut left,
        A,
        &Command::MulliganHand,
        Error::DecisionAlreadySubmitted,
    );
    reject(
        &mut left,
        B,
        &Command::ChooseInitialResources { cards: vec![] },
        Error::WrongPhase,
    );
    let left_events = submit(&mut left, B, &Command::MulliganHand);
    submit(&mut right, B, &Command::MulliganHand);
    let right_events = submit(&mut right, A, &Command::MulliganHand);
    assert_eq!(left_events, right_events);
    assert_eq!(
        left.snapshot().expect("snapshot"),
        right.snapshot().expect("snapshot")
    );
    assert_eq!(left.view_for(A).hand.len(), 6);
    assert_ne!(left.view_for(A).hand, original);
    assert_eq!(left.view_for(A).players[0].deck_size, 14);
    reject(&mut left, A, &Command::MulliganHand, Error::WrongPhase);
    let a: Vec<_> = left.view_for(A).hand[..2]
        .iter()
        .map(|card| card.id)
        .collect();
    let b: Vec<_> = left.view_for(B).hand[..2]
        .iter()
        .map(|card| card.id)
        .collect();
    submit(
        &mut left,
        A,
        &Command::ChooseInitialResources { cards: a.clone() },
    );
    assert_eq!(left.view_for(A).resources.len(), 0);
    assert_eq!(left.view_for(A).hand.len(), 6);
    reject(
        &mut left,
        A,
        &Command::ChooseInitialResources { cards: vec![] },
        Error::DecisionAlreadySubmitted,
    );
    let left_events = submit(
        &mut left,
        B,
        &Command::ChooseInitialResources { cards: b.clone() },
    );
    submit(
        &mut right,
        B,
        &Command::ChooseInitialResources {
            cards: b.into_iter().rev().collect(),
        },
    );
    let right_events = submit(
        &mut right,
        A,
        &Command::ChooseInitialResources {
            cards: a.into_iter().rev().collect(),
        },
    );
    assert_eq!(left_events, right_events);
    assert_eq!(
        left.snapshot().expect("snapshot"),
        right.snapshot().expect("snapshot")
    );
    assert_eq!(left.view_for(A).hand.len(), 4);
    assert!(
        left.view_for(A)
            .resources
            .iter()
            .all(|resource| resource.ready)
    );
    assert_eq!(left.view_for(A).round, 1);
    assert_eq!(left.view_for(A).players[0].deck_size, 14);
}

#[test]
fn keep_and_initial_resource_zero_one_two_boundaries() {
    for count in 0..=2 {
        let fixture = Fixture::new(0, 0, 1);
        let mut game = fixture.game();
        let hand = game.view_for(A).hand;
        keep(&mut game);
        assert_eq!(game.view_for(A).hand, hand);
        let cards: Vec<_> = hand.iter().take(count).map(|card| card.id).collect();
        submit(&mut game, A, &Command::ChooseInitialResources { cards });
        submit(
            &mut game,
            B,
            &Command::ChooseInitialResources { cards: vec![] },
        );
        assert_eq!(game.view_for(A).hand.len(), 6 - count);
        assert_eq!(game.view_for(A).resources.len(), count);
    }
    let mut game = Fixture::new(0, 0, 1).game();
    keep(&mut game);
    let ids: Vec<_> = game.view_for(A).hand.iter().map(|card| card.id).collect();
    reject(
        &mut game,
        A,
        &Command::ChooseInitialResources {
            cards: ids[..3].to_vec(),
        },
        Error::TooManyResources,
    );
    reject(
        &mut game,
        A,
        &Command::ChooseInitialResources {
            cards: vec![ids[0], ids[0]],
        },
        Error::DuplicateResource,
    );
    reject(
        &mut game,
        A,
        &Command::ChooseInitialResources {
            cards: vec![CardId(0)],
        },
        Error::InvalidCard,
    );
}

#[test]
fn exact_deterministic_payment_and_regroup_readiness() {
    let fixture = Fixture::new(1, 1, 9);
    let mut game = fixture.start([2, 0]);
    let mut ids: Vec<_> = game
        .view_for(A)
        .resources
        .iter()
        .map(|resource| resource.card.id)
        .collect();
    ids.sort_unstable();
    let first = game.view_for(A).hand[0].id;
    let events = submit(&mut game, A, &Command::PlayUnit { card: first }).for_player(A);
    assert!(
        matches!(&events[0], Event::ResourcesExhausted { count: 1, cards: Some(cards), .. } if cards == &vec![ids[0]])
    );
    assert!(
        matches!(&events[1], Event::UnitPlayed { unit } if unit.card.id == first && !unit.ready)
    );
    reject(
        &mut game,
        B,
        &Command::PlayUnit {
            card: game_hand_id(&fixture, B),
        },
        Error::InsufficientResources,
    );
    submit(&mut game, B, &Command::Pass);
    reject(
        &mut game,
        A,
        &Command::Attack {
            attacker: first,
            target: AttackTarget::Stronghold(B),
        },
        Error::UnitExhausted,
    );
    deploy(&mut game, A);
    submit(&mut game, B, &Command::Pass);
    let third = game.view_for(A).hand[0].id;
    reject(
        &mut game,
        A,
        &Command::PlayUnit { card: third },
        Error::InsufficientResources,
    );
    end_round(&mut game);
    let before = game.view_for(A);
    assert_eq!(before.hand.len(), 4);
    assert!(before.resources.iter().all(|resource| !resource.ready));
    assert!(before.players[0].units.iter().all(|unit| !unit.ready));
    let card = before.hand[0].id;
    submit(&mut game, B, &Command::ChooseRegroupResource { card: None });
    assert_eq!(game.view_for(A).resources.len(), 2);
    submit(
        &mut game,
        A,
        &Command::ChooseRegroupResource { card: Some(card) },
    );
    assert_eq!(game.view_for(A).resources.len(), 3);
    assert!(
        game.view_for(A)
            .resources
            .iter()
            .all(|resource| resource.ready)
    );
    assert!(
        game.view_for(A).players[0]
            .units
            .iter()
            .all(|unit| unit.ready)
    );
    assert_eq!(game.view_for(A).active_player, Some(A));
}

fn game_hand_id(fixture: &Fixture, player: PlayerId) -> CardId {
    fixture.start([2, 0]).view_for(player).hand[0].id
}

#[test]
fn passing_is_temporary_and_claiming_moves_next_round_initiative() {
    let mut game = Fixture::new(0, 1, 2).start([0, 0]);
    submit(&mut game, A, &Command::Pass);
    assert_eq!(game.view_for(A).consecutive_passes, 1);
    deploy(&mut game, B);
    assert_eq!(game.view_for(A).consecutive_passes, 0);
    deploy(&mut game, A);
    submit(&mut game, B, &Command::ClaimInitiative);
    assert_eq!(game.view_for(A).initiative, A);
    assert_eq!(game.view_for(A).next_initiative, B);
    deploy(&mut game, A);
    assert_eq!(game.view_for(A).active_player, Some(A));
    end_round(&mut game);
    regroup(&mut game);
    assert_eq!(game.view_for(A).active_player, Some(B));
    end_round(&mut game);
    regroup(&mut game);
    assert_eq!(game.view_for(A).active_player, Some(B));
}

#[test]
fn zero_cost_duplicates_and_unrestricted_battlefield() {
    let mut fixture = Fixture::new(0, 0, 1);
    for player in &mut fixture.config.players {
        player.deck.resize(100, DefinitionId(1));
    }
    let mut game = fixture.start([0, 0]);
    submit(&mut game, A, &Command::ClaimInitiative);
    for _ in 0..6 {
        deploy(&mut game, B);
    }
    assert_eq!(game.view_for(A).players[1].units.len(), 6);
    assert!(game.view_for(A).resources.is_empty());
    for _ in 0..3 {
        end_round(&mut game);
        regroup(&mut game);
        submit(&mut game, A, &Command::ClaimInitiative);
        deploy(&mut game, B);
        deploy(&mut game, B);
    }
    assert_eq!(game.view_for(A).players[1].units.len(), 12);
    assert_eq!(game.view_for(A).players[1].discard.len(), 0);
}

#[test]
fn simultaneous_mutual_destruction_exhausted_retaliation_and_event_order() {
    let mut game = Fixture::new(0, 3, 2).start([0, 0]);
    let attacker = deploy(&mut game, A);
    let defender = deploy(&mut game, B);
    end_round(&mut game);
    regroup(&mut game);
    submit(&mut game, A, &Command::Pass);
    submit(
        &mut game,
        B,
        &Command::Attack {
            attacker: defender,
            target: AttackTarget::Stronghold(A),
        },
    );
    let events = submit(
        &mut game,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Unit(defender),
        },
    )
    .for_player(A);
    assert!(matches!(events[0], Event::AttackDeclared { .. }));
    assert_eq!(events[1], Event::UnitExhausted { card: attacker });
    assert!(
        matches!(&events[2], Event::CombatDamage { assignments } if assignments.len()==2 && assignments.iter().all(|damage| damage.amount==3 && damage.total==3))
    );
    let destroyed: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::UnitDestroyed { card, .. } => Some(card.id),
            _ => None,
        })
        .collect();
    assert_eq!(destroyed, vec![attacker, defender]);
    assert!(
        game.view_for(A)
            .players
            .iter()
            .all(|player| player.units.is_empty())
    );
    assert_eq!(game.view_for(A).players[0].discard[0].id, attacker);
    assert_eq!(game.view_for(A).players[1].discard[0].id, defender);
}

#[test]
fn defending_preserves_readiness_and_damage_persists_through_regroup() {
    let mut game = Fixture::new(0, 1, 5).start([0, 0]);
    let attacker = deploy(&mut game, A);
    let defender = deploy(&mut game, B);
    end_round(&mut game);
    regroup(&mut game);
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Unit(defender),
        },
    );
    assert!(!game.view_for(A).players[0].units[0].ready);
    assert!(game.view_for(A).players[1].units[0].ready);
    end_round(&mut game);
    regroup(&mut game);
    for player in &game.view_for(A).players {
        assert_eq!(player.units[0].damage, 1);
        assert_eq!(player.units[0].health, 5);
        assert_eq!(player.units[0].power, 1);
        assert!(player.units[0].ready);
    }
}

#[test]
fn partial_draw_empty_deck_and_simultaneous_lethal_fatigue() {
    let mut fixture = Fixture::new(0, 0, 1);
    fixture.config.players[0].deck.truncate(7);
    fixture.config.players[0].stronghold_health = 9;
    fixture.config.players[1].deck.truncate(6);
    fixture.config.players[1].stronghold_health = 12;
    let mut game = fixture.start([0, 0]);
    let events = end_round(&mut game);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::DeckExhaustionDamage {
            player: A,
            missing: 1,
            amount: 3,
            ..
        }
    )));
    assert_eq!(game.view_for(A).players[0].hand_size, 7);
    assert_eq!(game.view_for(A).players[0].stronghold_damage, 3);
    assert_eq!(game.view_for(A).players[1].stronghold_damage, 6);
    regroup(&mut game);
    let events = end_round(&mut game);
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Draw));
    assert_eq!(game.view_for(A).players[0].stronghold_damage, 9);
    assert_eq!(game.view_for(A).players[1].stronghold_damage, 12);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded {
            outcome: Outcome::Draw
        })
    );
}

#[test]
fn invalid_action_categories_are_atomic_and_do_not_leak_other_zones() {
    let fixture = Fixture::new(1, 2, 5);
    let mut game = fixture.start([2, 2]);
    let own = game.view_for(A).hand[0].id;
    let opposing = game.view_for(B).hand[0].id;
    reject(&mut game, B, &Command::Pass, Error::NotYourAction);
    reject(&mut game, A, &Command::KeepHand, Error::WrongPhase);
    for card in [CardId(0), opposing, game.view_for(A).resources[0].card.id] {
        reject(
            &mut game,
            A,
            &Command::PlayUnit { card },
            Error::InvalidCard,
        );
        reject(
            &mut game,
            A,
            &Command::Attack {
                attacker: card,
                target: AttackTarget::Stronghold(B),
            },
            Error::InvalidCard,
        );
    }
    submit(&mut game, A, &Command::PlayUnit { card: own });
    let defender = deploy(&mut game, B);
    end_round(&mut game);
    regroup(&mut game);
    let inaccessible = game.view_for(B).hand[0].id;
    for target in [
        AttackTarget::Unit(own),
        AttackTarget::Unit(inaccessible),
        AttackTarget::Unit(CardId(u64::MAX)),
        AttackTarget::Stronghold(A),
    ] {
        reject(
            &mut game,
            A,
            &Command::Attack {
                attacker: own,
                target,
            },
            Error::InvalidTarget,
        );
    }
    submit(&mut game, A, &Command::ClaimInitiative);
    reject(
        &mut game,
        A,
        &Command::Attack {
            attacker: own,
            target: AttackTarget::Unit(defender),
        },
        Error::NotYourAction,
    );
    reject(
        &mut game,
        B,
        &Command::ClaimInitiative,
        Error::InitiativeUnavailable,
    );
}

#[test]
fn maximum_stats_accumulate_without_overflow_and_zero_power_is_legal() {
    let mut fixture = Fixture::new(0, u32::MAX - 1, u32::MAX);
    for player in &mut fixture.config.players {
        player.stronghold_health = u32::MAX;
    }
    let mut game = fixture.start([0, 0]);
    let attacker = deploy(&mut game, A);
    let defender = deploy(&mut game, B);
    end_round(&mut game);
    regroup(&mut game);
    submit(
        &mut game,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Unit(defender),
        },
    );
    assert_eq!(
        game.view_for(A).players[0].units[0].damage,
        u64::from(u32::MAX - 1)
    );
    let events = submit(
        &mut game,
        B,
        &Command::Attack {
            attacker: defender,
            target: AttackTarget::Unit(attacker),
        },
    )
    .for_player(A);
    assert!(events.iter().any(|event| matches!(event, Event::CombatDamage { assignments } if assignments.iter().all(|damage| damage.total==2*u64::from(u32::MAX-1)))));
    fixture.definitions[0].cost = u32::MAX;
    let mut unaffordable = fixture.start([2, 2]);
    let card = unaffordable.view_for(A).hand[0].id;
    reject(
        &mut unaffordable,
        A,
        &Command::PlayUnit { card },
        Error::InsufficientResources,
    );
    let mut zero = Fixture::new(0, 0, 1).start([0, 0]);
    let attacker = deploy(&mut zero, A);
    submit(&mut zero, B, &Command::Pass);
    end_round(&mut zero);
    regroup(&mut zero);
    submit(
        &mut zero,
        A,
        &Command::Attack {
            attacker,
            target: AttackTarget::Stronghold(B),
        },
    );
    assert_eq!(zero.view_for(A).players[1].stronghold_damage, 0);
    assert!(!zero.view_for(A).players[0].units[0].ready);
}

#[test]
fn catalog_order_is_not_a_randomness_input() {
    let mut fixture = Fixture::new(0, 1, 2);
    fixture.definitions.push(UnitDefinition {
        id: DefinitionId(2),
        cost: 2,
        power: 0,
        health: 1,
    });
    fixture.config.players[0].deck[0] = DefinitionId(2);
    let first = fixture.game().snapshot().expect("snapshot");
    fixture.definitions.reverse();
    assert_eq!(first, fixture.game().snapshot().expect("snapshot"));
}

#[test]
fn regroup_buffering_and_rejection_are_atomic_in_both_arrival_orders() {
    let fixture = Fixture::new(0, 1, 3);
    let mut left = fixture.start([0, 0]);
    end_round(&mut left);
    let mut right =
        Game::restore(&left.snapshot().expect("snapshot"), fixture.definitions).expect("restore");
    let a = left.view_for(A).hand[0].id;
    let b = left.view_for(B).hand[0].id;
    reject(
        &mut left,
        A,
        &Command::ChooseRegroupResource { card: Some(b) },
        Error::InvalidCard,
    );
    reject(
        &mut left,
        A,
        &Command::PlayUnit { card: a },
        Error::WrongPhase,
    );
    submit(
        &mut left,
        A,
        &Command::ChooseRegroupResource { card: Some(a) },
    );
    reject(
        &mut left,
        A,
        &Command::ChooseRegroupResource { card: None },
        Error::DecisionAlreadySubmitted,
    );
    let left_events = submit(
        &mut left,
        B,
        &Command::ChooseRegroupResource { card: Some(b) },
    );
    submit(
        &mut right,
        B,
        &Command::ChooseRegroupResource { card: Some(b) },
    );
    let right_events = submit(
        &mut right,
        A,
        &Command::ChooseRegroupResource { card: Some(a) },
    );
    assert_eq!(left_events, right_events);
    assert_eq!(
        left.snapshot().expect("snapshot"),
        right.snapshot().expect("snapshot")
    );
}

#[test]
fn maximum_stronghold_health_supports_persistent_overkill_damage() {
    let mut fixture = Fixture::new(0, u32::MAX - 1, 1);
    fixture.config.players[1].stronghold_health = u32::MAX;
    let mut game = fixture.start([0, 0]);
    let attacker = deploy(&mut game, A);
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
    assert_eq!(game.view_for(A).outcome, None);
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
    assert_eq!(
        game.view_for(A).players[1].stronghold_damage,
        2 * u64::from(u32::MAX - 1)
    );
    assert_eq!(game.view_for(A).outcome, Some(Outcome::Winner(A)));
    let restored = Game::restore(&game.snapshot().expect("snapshot"), fixture.definitions)
        .expect("restore terminal overkill");
    assert_eq!(restored.view_for(A), game.view_for(A));
}
