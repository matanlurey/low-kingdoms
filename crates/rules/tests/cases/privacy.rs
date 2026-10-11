//! Player projections, event redaction, error privacy, and legal-action discovery.

use low_kingdoms_rules::command::{AttackTarget, Command};
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::error::Error;
use low_kingdoms_rules::event::Event;
use low_kingdoms_rules::model::{CardId, DefinitionId, UnitDefinition};

use crate::{A, B, Fixture, deploy, end_round, keep, regroup, submit};

#[test]
fn views_and_events_omit_opposing_private_identifiers_and_both_decks() {
    let mut fixture = Fixture::new(1, 2, 4);
    fixture.definitions.push(UnitDefinition {
        id: DefinitionId(91),
        cost: 1,
        power: 2,
        health: 4,
    });
    fixture.config.players[1].deck.fill(DefinitionId(91));
    let (mut game, initial) =
        Game::new(fixture.config.clone(), fixture.definitions.clone()).expect("fixture");
    let opposing_hand = game.view_for(B).hand;
    let initial_public = initial.for_player(A);
    assert!(initial_public.iter().any(|event| matches!(
        event,
        Event::CardsDrawn {
            player: B,
            count: 6,
            cards: None
        }
    )));
    assert!(initial_public.iter().any(|event| matches!(event, Event::CardsDrawn { player: A, cards: Some(cards), .. } if cards.len()==6)));
    keep(&mut game);
    let a_resources = game.view_for(A).hand[..2]
        .iter()
        .map(|card| card.id)
        .collect();
    let b_resources = opposing_hand[..2].iter().map(|card| card.id).collect();
    submit(
        &mut game,
        A,
        &Command::ChooseInitialResources { cards: a_resources },
    );
    let batch = submit(
        &mut game,
        B,
        &Command::ChooseInitialResources { cards: b_resources },
    );
    assert!(batch.for_player(A).iter().any(|event| matches!(
        event,
        Event::CardsResourced {
            player: B,
            count: 2,
            cards: None
        }
    )));
    let public_json = serde_json::to_value(game.view_for(A)).expect("view");
    assert_eq!(
        public_json["players"][1]["resources"],
        serde_json::json!({"total":2,"ready":2})
    );
    assert!(public_json["players"][1].get("hand").is_none());
    assert!(public_json["players"][1].get("deck").is_none());
    assert!(public_json.get("pending").is_none());
    assert!(public_json.get("rng").is_none());
    assert!(public_json.get("definitions").is_none());
    assert!(
        !serde_json::to_string(&public_json)
            .expect("view")
            .contains("\"definition\":91")
    );
    let invalid_hand = opposing_hand[3].id;
    for card in [invalid_hand, opposing_hand[0].id, CardId(u64::MAX)] {
        assert_eq!(
            game.apply(A, &Command::PlayUnit { card }),
            Err(Error::InvalidCard)
        );
    }
    deploy(&mut game, A);
    let opposing = game.view_for(B).hand[0].id;
    let payment = submit(&mut game, B, &Command::PlayUnit { card: opposing });
    assert!(payment.for_player(A).iter().any(|event| matches!(
        event,
        Event::ResourcesExhausted {
            player: B,
            count: 1,
            cards: None
        }
    )));
    assert_eq!(
        game.view_for(A).players[1].units[0].card.definition,
        DefinitionId(91)
    );
    assert!(format!("{payment:?}").starts_with("EventBatch { count:"));
    let restored_events: Vec<Event> =
        serde_json::from_str(&serde_json::to_string(&payment.for_player(A)).expect("events"))
            .expect("events round trip");
    assert_eq!(restored_events, payment.for_player(A));
}

#[test]
fn private_pending_decisions_do_not_change_the_opponents_view_or_options() {
    let fixture = Fixture::new(0, 1, 1);
    let mut game = fixture.game();
    let opponent_before = game.view_for(B);
    let legal_before = game.legal_commands_for(B);
    submit(&mut game, A, &Command::MulliganHand);
    assert_eq!(game.view_for(B), opponent_before);
    assert_eq!(game.legal_commands_for(B), legal_before);
    assert!(game.legal_commands_for(A).is_empty());
    submit(&mut game, B, &Command::KeepHand);
    let opponent_before = game.view_for(B);
    let legal_before = game.legal_commands_for(B);
    let resource = game.view_for(A).hand[0].id;
    submit(
        &mut game,
        A,
        &Command::ChooseInitialResources {
            cards: vec![resource],
        },
    );
    assert_eq!(game.view_for(B), opponent_before);
    assert_eq!(game.legal_commands_for(B), legal_before);
    submit(
        &mut game,
        B,
        &Command::ChooseInitialResources { cards: vec![] },
    );
    end_round(&mut game);
    let opponent_before = game.view_for(B);
    let legal_before = game.legal_commands_for(B);
    submit(&mut game, A, &Command::ChooseRegroupResource { card: None });
    assert_eq!(game.view_for(B), opponent_before);
    assert_eq!(game.legal_commands_for(B), legal_before);
}

#[test]
fn legal_discovery_is_complete_safe_and_every_suggestion_executes() {
    let fixture = Fixture::new(1, 2, 3);
    let mut game = fixture.game();
    assert_eq!(
        game.legal_commands_for(A),
        vec![Command::KeepHand, Command::MulliganHand]
    );
    keep(&mut game);
    assert_eq!(game.legal_commands_for(A).len(), 22); // empty, six singles, fifteen pairs
    for command in game.legal_commands_for(A) {
        let mut copy = Game::restore(
            &game.snapshot().expect("snapshot"),
            fixture.definitions.clone(),
        )
        .expect("restore");
        assert!(copy.apply(A, &command).is_ok());
    }
    let cards = game.view_for(A).hand[..2]
        .iter()
        .map(|card| card.id)
        .collect();
    submit(&mut game, A, &Command::ChooseInitialResources { cards });
    let cards = game.view_for(B).hand[..2]
        .iter()
        .map(|card| card.id)
        .collect();
    submit(&mut game, B, &Command::ChooseInitialResources { cards });
    assert!(game.legal_commands_for(B).is_empty());
    assert_eq!(game.legal_commands_for(A).len(), 6); // pass, claim, four payable units
    let attacker = deploy(&mut game, A);
    let defender = deploy(&mut game, B);
    end_round(&mut game);
    assert_eq!(
        game.legal_commands_for(A).len(),
        game.view_for(A).hand.len() + 1
    );
    regroup(&mut game);
    let actions = game.legal_commands_for(A);
    assert!(actions.contains(&Command::Attack {
        attacker,
        target: AttackTarget::Unit(defender)
    }));
    assert!(actions.contains(&Command::Attack {
        attacker,
        target: AttackTarget::Stronghold(B)
    }));
    assert!(!actions.contains(&Command::Attack {
        attacker,
        target: AttackTarget::Stronghold(A)
    }));
    let before = game.snapshot().expect("snapshot");
    assert_eq!(actions, game.legal_commands_for(A));
    assert_eq!(before, game.snapshot().expect("snapshot"));
    for command in actions {
        let mut copy = Game::restore(&before, fixture.definitions.clone()).expect("restore");
        assert!(copy.apply(A, &command).is_ok());
    }
    submit(&mut game, A, &Command::ClaimInitiative);
    assert!(game.legal_commands_for(A).is_empty());
    assert!(
        !game
            .legal_commands_for(B)
            .contains(&Command::ClaimInitiative)
    );
}
