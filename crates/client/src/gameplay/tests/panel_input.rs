use bevy::{
    input::touch::{TouchInput, TouchPhase},
    input_focus::InputFocus,
    math::Affine2,
    prelude::*,
    ui::ComputedStackIndex,
    window::PrimaryWindow,
};
use low_kingdoms_rules::{command::Command, model::Phase};

use super::{
    Activate, Battle, Control, Screen, Speed, activate, ai_action, focus, human_command,
    resume_fixture, revision, session_save, state,
};
use crate::gameplay::presentation::OverlayRoot;

fn control_entity(app: &mut App, desired: Control) -> Entity {
    app.world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find_map(|(entity, control)| (*control == desired).then_some(entity))
        .expect("rendered control")
}

fn menu_touch_geometry(app: &mut App, menu: Entity) -> Entity {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .expect("window");
    let overlay = app
        .world_mut()
        .query_filtered::<Entity, With<OverlayRoot>>()
        .single(app.world())
        .expect("menu overlay");
    for (entity, size) in [
        (overlay, Vec2::new(375.0, 667.0)),
        (menu, Vec2::new(240.0, 46.0)),
    ] {
        app.world_mut().entity_mut(entity).insert((
            ComputedNode {
                size,
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from(Affine2::from_translation(Vec2::new(187.5, 333.5))),
            ComputedStackIndex(100),
            InheritedVisibility::VISIBLE,
        ));
    }
    window
}

fn touch_menu(app: &mut App, window: Entity, phase: TouchPhase) {
    app.world_mut().write_message(TouchInput {
        window,
        phase,
        id: 1,
        position: Vec2::new(187.5, 333.5),
        force: None,
    });
    app.update();
}

#[test]
fn held_main_menu_touch_survives_a_private_ai_resource_decision() {
    let (mut app, _) = resume_fixture(42, None, Speed::Normal);
    human_command(&mut app, &Command::KeepHand);
    ai_action(&mut app);
    assert_eq!(state(&app), Screen::Battle);
    assert_eq!(
        app.world()
            .resource::<Battle>()
            .view
            .as_ref()
            .expect("view")
            .phase,
        Phase::InitialResourcing
    );
    activate(&mut app, Control::More);
    let menu = control_entity(&mut app, Control::MainMenu);
    let window = menu_touch_geometry(&mut app, menu);
    let before = {
        let battle = app.world().resource::<Battle>();
        (
            battle.view.clone(),
            battle.legal.clone(),
            battle.history.clone(),
        )
    };
    let saved = session_save(&app);
    touch_menu(&mut app, window, TouchPhase::Started);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(menu));
    ai_action(&mut app);
    let battle = app.world().resource::<Battle>();
    assert_eq!(
        (
            battle.view.clone(),
            battle.legal.clone(),
            battle.history.clone()
        ),
        before
    );
    assert!(
        session_save(&app) != saved,
        "accepted private command updates its save"
    );
    assert_eq!(control_entity(&mut app, Control::MainMenu), menu);
    touch_menu(&mut app, window, TouchPhase::Ended);
    app.update();
    assert_eq!(state(&app), Screen::Title);
}

#[test]
fn main_menu_navigation_wins_over_an_ai_action_due_in_the_same_update() {
    let (mut app, store) = resume_fixture(42, None, Speed::Instant);
    focus(&mut app, true);
    app.update();
    let before = revision(&app);
    let writes = store.0.lock().expect("slot").writes;
    // Dispatch queues the production PendingIfNeq navigation before AI runs.
    app.world_mut().write_message(Activate(Control::MainMenu));
    app.update();
    assert_eq!(revision(&app), before);
    app.update();
    assert_eq!(state(&app), Screen::Title);
    assert_eq!(store.0.lock().expect("slot").writes, writes);
}
