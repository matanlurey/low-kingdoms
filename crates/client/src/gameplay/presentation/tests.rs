use bevy::prelude::*;
use low_kingdoms_battle::content::{CardDefinition, Content};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    engine::Game,
    model::{CardId, DefinitionId, MatchConfig, PlayerConfig, PlayerId, UnitDefinition},
};

use crate::{gameplay::Battle, screens::Screen, ui::Control};

use super::{CardFace, DisabledControl, Presentation, Zone, configure};

mod geometry;

fn fixture() -> (App, Game) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<Screen>()
        .init_resource::<Battle>();
    configure(&mut app);
    app.world_mut().spawn(Window::default());
    let definition = UnitDefinition {
        id: DefinitionId(1),
        cost: 0,
        power: 2,
        health: 3,
    };
    let player = PlayerConfig {
        deck: vec![definition.id; 20],
        stronghold_health: 30,
    };
    let (mut game, _) = Game::new(
        MatchConfig {
            players: [player.clone(), player],
            seed: 42,
            starting_player: Some(PlayerId::Zero),
        },
        vec![definition],
    )
    .expect("valid fixture");
    for player in PlayerId::ALL {
        game.apply(player, &Command::KeepHand).expect("keep");
    }
    for player in PlayerId::ALL {
        game.apply(
            player,
            &Command::ChooseInitialResources { cards: Vec::new() },
        )
        .expect("resources");
    }
    app.world_mut().resource_mut::<Battle>().catalog = vec![CardDefinition {
        id: definition.id,
        name: "Infantry".to_owned(),
        cost: definition.cost,
        power: definition.power,
        health: definition.health,
    }];
    project(&mut app, &game);
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Battle);
    app.update();
    app.update();
    (app, game)
}

fn project(app: &mut App, game: &Game) {
    let mut battle = app.world_mut().resource_mut::<Battle>();
    battle.view = Some(game.view_for(PlayerId::Zero));
    battle.legal = game.legal_commands_for(PlayerId::Zero);
    battle.revision += 1;
}

fn apply(app: &mut App, game: &mut Game, player: PlayerId, command: &Command) {
    game.apply(player, command).expect("legal fixture command");
    project(app, game);
    app.update();
}

fn face(app: &mut App, id: CardId) -> Entity {
    app.world_mut()
        .query_filtered::<(Entity, &Control), With<CardFace>>()
        .iter(app.world())
        .find_map(|(entity, control)| (*control == Control::Card(id)).then_some(entity))
        .expect("visible card")
}

fn ready_combat(app: &mut App, game: &mut Game) -> (CardId, CardId) {
    let own = game.view_for(PlayerId::Zero).hand[0].id;
    let enemy = game.view_for(PlayerId::One).hand[0].id;
    apply(app, game, PlayerId::Zero, &Command::PlayUnit { card: own });
    apply(app, game, PlayerId::One, &Command::PlayUnit { card: enemy });
    apply(app, game, PlayerId::Zero, &Command::Pass);
    apply(app, game, PlayerId::One, &Command::Pass);
    for player in PlayerId::ALL {
        apply(
            app,
            game,
            player,
            &Command::ChooseRegroupResource { card: None },
        );
    }
    (own, enemy)
}

#[test]
fn damage_reconciliation_preserves_entities_and_scroll() {
    let (mut app, mut game) = fixture();
    let (own, enemy) = ready_combat(&mut app, &mut game);
    let own_entity = face(&mut app, own);
    let enemy_entity = face(&mut app, enemy);
    let pane = app.world().resource::<Presentation>().zones[&Zone::Friendly];
    app.world_mut()
        .get_mut::<ScrollPosition>(pane)
        .expect("pane")
        .0 = Vec2::new(25.0, 60.0);
    apply(
        &mut app,
        &mut game,
        PlayerId::Zero,
        &Command::Attack {
            attacker: own,
            target: AttackTarget::Unit(enemy),
        },
    );
    assert_eq!(face(&mut app, own), own_entity);
    assert_eq!(face(&mut app, enemy), enemy_entity);
    assert_eq!(
        app.world().get::<ScrollPosition>(pane).expect("pane").0,
        Vec2::new(25.0, 60.0)
    );
    let texts = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>();
    assert!(
        texts
            .iter()
            .any(|text| text.contains("2 / 1\nDamage: 2\nExhausted"))
    );
    apply(
        &mut app,
        &mut game,
        PlayerId::One,
        &Command::Attack {
            attacker: enemy,
            target: AttackTarget::Unit(own),
        },
    );
    assert!(app.world().get_entity(own_entity).is_err());
    assert!(app.world().get_entity(enemy_entity).is_err());
}

#[test]
fn targeting_enables_only_actual_legal_targets() {
    let (mut app, mut game) = fixture();
    let (own, enemy) = ready_combat(&mut app, &mut game);
    app.world_mut().resource_mut::<Battle>().attacker = Some(own);
    app.update();
    let own = face(&mut app, own);
    let enemy = face(&mut app, enemy);
    assert!(app.world().get::<DisabledControl>(own).is_some());
    assert!(app.world().get::<DisabledControl>(enemy).is_none());
    let stronghold = app
        .world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find_map(|(entity, control)| (*control == Control::EnemyStronghold).then_some(entity))
        .expect("stronghold");
    assert!(app.world().get::<DisabledControl>(stronghold).is_none());
}

#[test]
fn orientation_rebuild_keeps_one_root_and_restores_scroll() {
    let (mut app, mut game) = fixture();
    let (own, _) = ready_combat(&mut app, &mut game);
    let pane = app.world().resource::<Presentation>().zones[&Zone::Friendly];
    app.world_mut()
        .get_mut::<ScrollPosition>(pane)
        .expect("pane")
        .0
        .x = 40.0;
    app.world_mut()
        .query::<&mut Window>()
        .single_mut(app.world_mut())
        .expect("window")
        .resolution
        .set(375.0, 667.0);
    app.update();
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<crate::screens::ScreenRoot>>()
            .iter(app.world())
            .count(),
        1
    );
    let new_pane = app.world().resource::<Presentation>().zones[&Zone::Friendly];
    assert_ne!(new_pane, pane);
    assert_eq!(
        app.world()
            .get::<ScrollPosition>(new_pane)
            .expect("pane")
            .0
            .x
            .to_bits(),
        40.0_f32.to_bits()
    );
    let own_entity = face(&mut app, own);
    assert!(app.world().get::<CardFace>(own_entity).is_some());
    assert!(
        !app.world()
            .resource::<Presentation>()
            .zones
            .contains_key(&Zone::Hand)
    );
}

#[test]
fn missing_presentation_entity_is_reconciled_without_a_game_change() {
    let (mut app, mut game) = fixture();
    let (own, _) = ready_combat(&mut app, &mut game);
    let old = face(&mut app, own);
    let expected = app.world().resource::<Battle>().view.clone();
    app.world_mut().despawn(old);
    app.update();
    assert_ne!(face(&mut app, own), old);
    assert_eq!(app.world().resource::<Battle>().view, expected);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<crate::screens::ScreenRoot>>()
            .iter(app.world())
            .count(),
        1
    );
}

#[test]
fn same_panel_reconciliation_preserves_overlay_scroll() {
    let (mut app, mut game) = fixture();
    app.world_mut().resource_mut::<Battle>().panel = crate::gameplay::Panel::Log;
    app.update();
    let pane = app
        .world()
        .resource::<Presentation>()
        .overlay_content
        .expect("log pane");
    app.world_mut()
        .get_mut::<ScrollPosition>(pane)
        .expect("pane")
        .0
        .y = 60.0;
    let own = game.view_for(PlayerId::Zero).hand[0].id;
    apply(
        &mut app,
        &mut game,
        PlayerId::Zero,
        &Command::PlayUnit { card: own },
    );
    let pane = app
        .world()
        .resource::<Presentation>()
        .overlay_content
        .expect("log pane");
    assert_eq!(
        app.world()
            .get::<ScrollPosition>(pane)
            .expect("pane")
            .0
            .y
            .to_bits(),
        60.0_f32.to_bits()
    );
}

#[test]
fn battle_menu_controls_survive_public_actions_and_refresh_claim_eligibility() {
    let (mut app, mut game) = fixture();
    app.world_mut().resource_mut::<Battle>().panel = crate::gameplay::Panel::More;
    app.update();
    let controls = |app: &mut App| {
        let mut controls = app
            .world_mut()
            .query::<(Entity, &Control)>()
            .iter(app.world())
            .filter(|(_, control)| {
                matches!(
                    control,
                    Control::MainMenu | Control::ClaimInitiative | Control::Close
                )
            })
            .map(|(entity, control)| (*control, entity))
            .collect::<Vec<_>>();
        controls.sort_unstable_by_key(|(_, entity)| entity.to_bits());
        controls
    };
    let original = controls(&mut app);
    let claim = original
        .iter()
        .find_map(|(control, entity)| (*control == Control::ClaimInitiative).then_some(*entity))
        .expect("claim control");
    assert!(app.world().get::<DisabledControl>(claim).is_none());
    apply(&mut app, &mut game, PlayerId::Zero, &Command::Pass);
    assert_eq!(controls(&mut app), original);
    assert!(app.world().get::<DisabledControl>(claim).is_some());
    let card = game.view_for(PlayerId::One).hand[0].id;
    apply(
        &mut app,
        &mut game,
        PlayerId::One,
        &Command::PlayUnit { card },
    );
    assert_eq!(controls(&mut app), original);
    assert!(app.world().get::<DisabledControl>(claim).is_none());
    app.world_mut().resource_mut::<Battle>().speed = crate::gameplay::Speed::Fast;
    app.update();
    assert_eq!(controls(&mut app), original);
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0 == "AI speed: Fast")
    );
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0 == "Claiming initiative ends your actions this round.")
    );
}

#[test]
fn opening_hand_inspection_has_correct_stats_without_a_play_control_or_wrong_zone() {
    let (mut app, _) = fixture();
    let content = Content::load().expect("embedded content");
    let definitions = content
        .cards()
        .iter()
        .map(CardDefinition::unit_definition)
        .collect();
    let (game, _) = Game::new(content.config(42), definitions).expect("opening hand");
    let card = game.view_for(PlayerId::Zero).hand[0];
    let definition = content
        .cards()
        .iter()
        .find(|definition| definition.id == card.definition)
        .expect("definition");
    project(&mut app, &game);
    {
        let mut battle = app.world_mut().resource_mut::<Battle>();
        battle.catalog = content.cards().to_vec();
        battle.selected = Some(card.id);
        battle.panel = crate::gameplay::Panel::Inspect;
    }
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Mulligan);
    app.update();
    app.update();
    let texts = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.clone())
        .collect::<Vec<_>>();
    assert!(texts.iter().any(|text| text == &definition.name));
    assert!(
        texts
            .iter()
            .any(|text| text.contains(&format!("Maximum Health {}", definition.health)))
    );
    assert!(
        !texts
            .iter()
            .any(|text| text.contains("resource or discard zone"))
    );
    assert!(
        !app.world_mut()
            .query::<&Control>()
            .iter(app.world())
            .any(|control| *control == Control::PlayUnit)
    );
}

#[test]
fn hand_captions_switch_from_resourcing_to_inspection_after_the_decision_is_submitted() {
    let (mut app, _) = fixture();
    let content = Content::load().expect("embedded content");
    let definitions = content
        .cards()
        .iter()
        .map(CardDefinition::unit_definition)
        .collect();
    let (mut game, _) = Game::new(content.config(42), definitions).expect("opening hand");
    for player in PlayerId::ALL {
        game.apply(player, &Command::KeepHand).expect("keep hand");
    }
    project(&mut app, &game);
    {
        let mut battle = app.world_mut().resource_mut::<Battle>();
        battle.catalog = content.cards().to_vec();
        battle.panel = crate::gameplay::Panel::Hand;
    }
    app.update();
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Tap to resource"))
    );
    let card = game.view_for(PlayerId::Zero).hand[0].id;
    app.world_mut().resource_mut::<Battle>().resource_selection = vec![card];
    app.update();
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Selected resource"))
    );
    game.apply(
        PlayerId::Zero,
        &Command::ChooseInitialResources { cards: vec![card] },
    )
    .expect("resource decision");
    project(&mut app, &game);
    app.world_mut()
        .resource_mut::<Battle>()
        .resource_selection
        .clear();
    app.update();
    assert!(
        !app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Tap to resource"))
    );
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Tap to inspect"))
    );
}
