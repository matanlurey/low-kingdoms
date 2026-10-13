//! RON validation, deterministic tactical decisions, and complete headless prototype matches.
#![cfg(test)]

use low_kingdoms_battle::ai::{AiError, choose};
use low_kingdoms_battle::content::{CardDefinition, Content, ContentError};
use low_kingdoms_rules::command::{AttackTarget, Command, LoggedCommand};
use low_kingdoms_rules::engine::Game;
use low_kingdoms_rules::model::{Card, CardId, DefinitionId, Phase, PlayerId};
use low_kingdoms_rules::view::{PlayerView, PublicPlayer, ResourceCounts, UnitView};

const A: PlayerId = PlayerId::Zero;
const B: PlayerId = PlayerId::One;
const CARDS: &str = include_str!("../../../assets/content/cards.ron");
const DECKS: &str = include_str!("../../../assets/content/decks.ron");

fn catalog() -> Vec<CardDefinition> {
    Content::load()
        .expect("embedded prototype content")
        .cards()
        .to_vec()
}

fn player() -> PublicPlayer {
    PublicPlayer {
        deck_size: 14,
        hand_size: 6,
        resources: ResourceCounts { total: 2, ready: 2 },
        units: Vec::new(),
        discard: Vec::new(),
        stronghold_health: 30,
        stronghold_damage: 0,
    }
}

fn fixture_view(definitions: &[u32], phase: Phase) -> PlayerView {
    PlayerView {
        player: A,
        phase,
        round: 1,
        initiative: A,
        next_initiative: A,
        active_player: (phase == Phase::Action).then_some(A),
        initiative_claimant: None,
        consecutive_passes: 0,
        outcome: None,
        players: [player(), player()],
        hand: definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| Card {
                id: CardId(u64::try_from(index + 1).expect("small fixture")),
                definition: DefinitionId(*definition),
            })
            .collect(),
        resources: Vec::new(),
        decision_submitted: false,
    }
}

fn unit(id: u64, power: u32, health: u32, damage: u64, owner: PlayerId) -> UnitView {
    UnitView {
        card: Card {
            id: CardId(id),
            definition: DefinitionId(1),
        },
        owner,
        power,
        health,
        damage,
        ready: owner == A,
    }
}

fn initial_commands(view: &PlayerView) -> Vec<Command> {
    let mut commands = vec![Command::ChooseInitialResources { cards: Vec::new() }];
    for (index, card) in view.hand.iter().enumerate() {
        commands.push(Command::ChooseInitialResources {
            cards: vec![card.id],
        });
        for other in view.hand.iter().skip(index + 1) {
            commands.push(Command::ChooseInitialResources {
                cards: vec![card.id, other.id],
            });
        }
    }
    commands
}

fn decks(references: &[u32], health: u32) -> String {
    let cards = references
        .iter()
        .map(|id| format!("({id})"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "[(cards:[{cards}],stronghold_health:{health}),(cards:[{cards}],stronghold_health:{health})]"
    )
}

#[test]
fn embedded_content_loads_two_varied_twenty_card_decks() {
    let content = Content::load().expect("valid embedded RON");
    let config = content.config(42);
    assert_eq!(content.cards().len(), 6);
    assert_eq!(config.seed, 42);
    assert_eq!(config.starting_player, None);
    assert!(config.players.iter().all(|player| player.deck.len() == 20));
    assert!(
        config
            .players
            .iter()
            .all(|player| player.stronghold_health == 30)
    );
    assert_ne!(config.players[0].deck, config.players[1].deck);
    assert!(content.cards().iter().any(|card| card.cost <= 1));
    assert!(content.cards().iter().any(|card| card.cost >= 5));
    Game::new(
        config,
        content
            .cards()
            .iter()
            .map(CardDefinition::unit_definition)
            .collect(),
    )
    .expect("validated content initializes the real engine");
}

#[test]
fn content_reports_malformed_missing_unknown_and_invalid_fields() {
    let valid_decks = decks(&[1; 20], 30);
    assert!(matches!(
        Content::parse("broken", &valid_decks),
        Err(ContentError::Syntax {
            source: "cards.ron",
            ..
        })
    ));
    assert!(matches!(
        Content::parse(CARDS, "broken"),
        Err(ContentError::Syntax {
            source: "decks.ron",
            ..
        })
    ));
    for cards in [
        "[(id:(1),name:\"Unit\",cost:1,power:1)]",
        "[(id:(1),name:\"Unit\",cost:-1,power:1,health:1)]",
        "[(id:(1),name:\"Unit\",cost:4294967296,power:1,health:1)]",
        "[(id:(1),name:\"Unit\",cost:1,power:4294967296,health:1)]",
        "[(id:(1),name:\"Unit\",cost:1,power:1,health:1,ability:\"Extra\")]",
    ] {
        assert!(matches!(
            Content::parse(cards, &valid_decks),
            Err(ContentError::Syntax { .. })
        ));
    }
    assert_eq!(
        Content::parse("[]", &valid_decks).unwrap_err(),
        ContentError::EmptyCatalog
    );
    let cards = "[(id:(1),name:\" \t\",cost:0,power:0,health:1)]";
    assert_eq!(
        Content::parse(cards, &valid_decks).unwrap_err(),
        ContentError::EmptyName(DefinitionId(1))
    );
    let cards = "[(id:(1),name:\"Unit\",cost:0,power:0,health:0)]";
    assert_eq!(
        Content::parse(cards, &valid_decks).unwrap_err(),
        ContentError::InvalidHealth(DefinitionId(1))
    );
    let cards = "[(id:(1),name:\"Unit\",cost:1,power:1,health:1),(id:(1),name:\"Other\",cost:1,power:1,health:1)]";
    assert_eq!(
        Content::parse(cards, &valid_decks).unwrap_err(),
        ContentError::DuplicateDefinition(DefinitionId(1))
    );
}

#[test]
fn content_validates_exact_decks_references_and_strongholds() {
    assert_eq!(
        Content::parse(CARDS, "[]").unwrap_err(),
        ContentError::DeckCount(0)
    );
    assert_eq!(
        Content::parse(CARDS, &decks(&[1; 19], 30)).unwrap_err(),
        ContentError::DeckLength {
            player: 0,
            actual: 19
        }
    );
    assert_eq!(
        Content::parse(CARDS, &decks(&[1; 21], 30)).unwrap_err(),
        ContentError::DeckLength {
            player: 0,
            actual: 21
        }
    );
    assert_eq!(
        Content::parse(CARDS, &decks(&[1; 20], 0)).unwrap_err(),
        ContentError::InvalidStronghold(0)
    );
    assert_eq!(
        Content::parse(CARDS, &decks(&[99; 20], 30)).unwrap_err(),
        ContentError::UnknownDefinition {
            player: 0,
            definition: DefinitionId(99)
        }
    );
    let boundary =
        "[(id:(1),name:\"Boundary\",cost:4294967295,power:4294967295,health:4294967295)]";
    assert!(Content::parse(boundary, &decks(&[1; 20], u32::MAX)).is_ok());
}

#[test]
fn compatibility_is_exact_but_ron_whitespace_and_catalog_order_are_irrelevant() {
    let content = Content::load().expect("content");
    let mut cards = content.cards().to_vec();
    cards.reverse();
    let reordered = ron::to_string(&cards).expect("RON encoding");
    assert_eq!(
        content.signature(),
        Content::parse(&reordered, DECKS)
            .expect("same data")
            .signature()
    );
    cards[0].name.push_str(" changed");
    assert_ne!(
        content.signature(),
        Content::parse(&ron::to_string(&cards).expect("RON"), DECKS)
            .expect("new name")
            .signature()
    );
    assert_ne!(
        content.signature(),
        Content::parse(CARDS, &DECKS.replace("30", "31"))
            .expect("new Health")
            .signature()
    );
    assert!(content.signature().contains("Infantry"));
}

#[test]
fn mulligan_keeps_an_early_curve_and_replaces_an_expensive_hand() {
    let legal = [Command::KeepHand, Command::MulliganHand];
    assert_eq!(
        choose(
            &fixture_view(&[1, 2, 3, 4, 5, 6], Phase::Mulligan),
            &legal,
            &catalog()
        )
        .expect("choice"),
        Command::KeepHand
    );
    assert_eq!(
        choose(
            &fixture_view(&[5, 5, 5, 6, 6, 6], Phase::Mulligan),
            &legal,
            &catalog()
        )
        .expect("choice"),
        Command::MulliganHand
    );
    assert_eq!(
        choose(&fixture_view(&[1; 6], Phase::Mulligan), &legal, &catalog()).expect("choice"),
        Command::KeepHand
    );
}

#[test]
fn initial_resources_prefer_expensive_duplicates_and_preserve_cheap_plays() {
    let view = fixture_view(&[1, 2, 3, 4, 6, 6], Phase::InitialResourcing);
    let legal = initial_commands(&view);
    assert_eq!(
        choose(&view, &legal, &catalog()).expect("choice"),
        Command::ChooseInitialResources {
            cards: vec![CardId(5), CardId(6)]
        }
    );
    let view = fixture_view(&[1, 6, 6, 6, 6, 6], Phase::InitialResourcing);
    let legal = initial_commands(&view);
    let Command::ChooseInitialResources { cards } =
        choose(&view, &legal, &catalog()).expect("choice")
    else {
        panic!("resource command")
    };
    assert_eq!(cards.len(), 2);
    assert!(!cards.contains(&CardId(1)));
}

#[test]
fn regroup_grows_toward_plays_and_preserves_the_last_affordable_card() {
    let view = fixture_view(&[1, 6], Phase::Regroup);
    let legal = [
        Command::ChooseRegroupResource { card: None },
        Command::ChooseRegroupResource {
            card: Some(CardId(1)),
        },
        Command::ChooseRegroupResource {
            card: Some(CardId(2)),
        },
    ];
    assert_eq!(
        choose(&view, &legal, &catalog()).expect("choice"),
        Command::ChooseRegroupResource {
            card: Some(CardId(2))
        }
    );
    let view = fixture_view(&[1], Phase::Regroup);
    assert_eq!(
        choose(&view, &legal[..2], &catalog()).expect("choice"),
        Command::ChooseRegroupResource { card: None }
    );
}

#[test]
fn immediate_winning_attack_precedes_a_trade_or_deployment() {
    let mut view = fixture_view(&[1], Phase::Action);
    view.players[0].units.push(unit(10, 4, 5, 0, A));
    view.players[1].units.push(unit(20, 1, 1, 0, B));
    view.players[1].stronghold_damage = 26;
    let winning = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Stronghold(B),
    };
    let legal = [
        Command::PlayUnit { card: CardId(1) },
        Command::Attack {
            attacker: CardId(10),
            target: AttackTarget::Unit(CardId(20)),
        },
        winning.clone(),
        Command::ClaimInitiative,
        Command::Pass,
    ];
    assert_eq!(choose(&view, &legal, &catalog()).expect("choice"), winning);
}

#[test]
fn favorable_trade_accounts_for_persistent_damage_and_exhausted_retaliation() {
    let mut view = fixture_view(&[1], Phase::Action);
    view.players[0].units.push(unit(10, 3, 5, 1, A));
    view.players[1].units.push(unit(20, 3, 5, 2, B));
    let trade = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Unit(CardId(20)),
    };
    let face = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Stronghold(B),
    };
    let legal = [
        Command::PlayUnit { card: CardId(1) },
        trade.clone(),
        face.clone(),
        Command::ClaimInitiative,
        Command::Pass,
    ];
    assert_eq!(
        choose(&view, &legal, &catalog()).expect("surviving kill"),
        trade
    );
    view.players[0].units[0].damage = 4;
    assert_eq!(
        choose(&view, &legal[1..], &catalog()).expect("unfavorable retaliation"),
        face
    );
}

#[test]
fn mutual_destruction_is_favorable_only_when_removing_a_more_valuable_unit() {
    let mut view = fixture_view(&[], Phase::Action);
    view.players[0].units.push(unit(10, 3, 2, 0, A));
    view.players[1].units.push(unit(20, 5, 3, 0, B));
    let trade = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Unit(CardId(20)),
    };
    let face = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Stronghold(B),
    };
    assert_eq!(
        choose(&view, &[face, trade.clone()], &catalog()).expect("valuable mutual trade"),
        trade
    );
}

#[test]
fn deployment_then_stronghold_damage_precede_claiming_and_passing() {
    let mut view = fixture_view(&[1, 2], Phase::Action);
    view.players[0].units.push(unit(10, 2, 2, 0, A));
    let face = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Stronghold(B),
    };
    let deploy = Command::PlayUnit { card: CardId(2) };
    let legal = [
        Command::Pass,
        Command::ClaimInitiative,
        face.clone(),
        Command::PlayUnit { card: CardId(1) },
        deploy.clone(),
    ];
    assert_eq!(
        choose(&view, &legal, &catalog()).expect("best deployment"),
        deploy
    );
    assert_eq!(
        choose(&view, &legal[..3], &catalog()).expect("direct attack"),
        face
    );
    assert_eq!(
        choose(&view, &legal[..2], &catalog()).expect("claim after useful actions"),
        Command::ClaimInitiative
    );
    assert_eq!(
        choose(&view, &legal[..1], &catalog()).expect("pass fallback"),
        Command::Pass
    );
}

#[test]
fn equal_choices_use_ids_independently_of_legal_or_catalog_order() {
    let view = fixture_view(&[2, 2], Phase::Action);
    let mut legal = vec![
        Command::PlayUnit { card: CardId(2) },
        Command::PlayUnit { card: CardId(1) },
        Command::Pass,
    ];
    let mut definitions = catalog();
    let first = choose(&view, &legal, &definitions).expect("choice");
    legal.reverse();
    definitions.reverse();
    assert_eq!(choose(&view, &legal, &definitions).expect("choice"), first);
    assert_eq!(first, Command::PlayUnit { card: CardId(1) });
}

#[test]
fn boundary_statistics_and_counts_do_not_overflow_ai_scores() {
    let definitions = vec![CardDefinition {
        id: DefinitionId(1),
        name: "Boundary".to_owned(),
        cost: u32::MAX,
        power: u32::MAX,
        health: u32::MAX,
    }];
    let mut view = fixture_view(&[1; 6], Phase::InitialResourcing);
    assert!(choose(&view, &initial_commands(&view), &definitions).is_ok());
    view.phase = Phase::Regroup;
    view.players[0].resources.total = usize::MAX;
    assert_eq!(
        choose(
            &view,
            &[Command::ChooseRegroupResource { card: None }],
            &definitions
        )
        .expect("boundary regroup"),
        Command::ChooseRegroupResource { card: None }
    );
    view.phase = Phase::Action;
    view.players[0]
        .units
        .push(unit(10, u32::MAX, u32::MAX, u64::from(u32::MAX) - 1, A));
    view.players[1]
        .units
        .push(unit(20, u32::MAX, u32::MAX, 0, B));
    let lethal = Command::Attack {
        attacker: CardId(10),
        target: AttackTarget::Stronghold(B),
    };
    assert_eq!(
        choose(
            &view,
            &[
                Command::PlayUnit { card: CardId(1) },
                Command::Attack {
                    attacker: CardId(10),
                    target: AttackTarget::Unit(CardId(20))
                },
                lethal.clone()
            ],
            &definitions
        )
        .expect("boundary attack"),
        lethal
    );
}

#[test]
fn ai_errors_are_explicit_and_do_not_require_authoritative_inspection() {
    assert_eq!(
        choose(&fixture_view(&[], Phase::Action), &[], &catalog()),
        Err(AiError::NoLegalCommand)
    );
    assert_eq!(
        choose(
            &fixture_view(&[99], Phase::Mulligan),
            &[Command::KeepHand],
            &catalog()
        ),
        Err(AiError::UnknownDefinition(DefinitionId(99)))
    );
    assert_eq!(
        choose(
            &fixture_view(&[], Phase::Action),
            &[Command::PlayUnit { card: CardId(99) }],
            &catalog()
        ),
        Err(AiError::MissingVisibleCard(CardId(99)))
    );
    assert_eq!(
        choose(
            &fixture_view(&[], Phase::Finished),
            &[Command::Pass],
            &catalog()
        ),
        Err(AiError::NoLegalCommand)
    );
}

#[test]
fn complete_ai_matches_are_legal_deterministic_and_terminate_across_seeds() {
    let content = Content::load().expect("content");
    let definitions = content
        .cards()
        .iter()
        .map(CardDefinition::unit_definition)
        .collect::<Vec<_>>();
    for seed in 0..32 {
        let (mut game, _) = Game::new(content.config(seed), definitions.clone()).expect("match");
        let mut log = Vec::new();
        let mut events = Vec::new();
        for _ in 0..512 {
            if game.view_for(A).phase == Phase::Finished {
                break;
            }
            let actor = PlayerId::ALL
                .into_iter()
                .find(|player| !game.legal_commands_for(*player).is_empty())
                .expect("unfinished game has a decision");
            let view = game.view_for(actor);
            let legal = game.legal_commands_for(actor);
            let command = choose(&view, &legal, content.cards()).expect("AI decision");
            let mut reversed = legal.clone();
            reversed.reverse();
            assert_eq!(
                choose(&view, &reversed, content.cards()).expect("reordered AI decision"),
                command
            );
            assert!(legal.contains(&command));
            let batch = game
                .apply(actor, &command)
                .expect("AI always returns a legal command");
            events.push(batch.for_player(A));
            log.push(LoggedCommand {
                player: actor,
                command,
            });
        }
        assert_eq!(game.view_for(A).phase, Phase::Finished, "seed {seed}");
        let (replayed, batches) =
            Game::replay(content.config(seed), definitions.clone(), &log).expect("replay");
        assert_eq!(
            game.snapshot().expect("snapshot"),
            replayed.snapshot().expect("replay snapshot")
        );
        assert_eq!(
            events,
            batches
                .iter()
                .skip(1)
                .map(|batch| batch.for_player(A))
                .collect::<Vec<_>>()
        );
    }
}
