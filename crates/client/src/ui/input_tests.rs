//! Raw-input regressions use production input systems with explicit logical hit regions.

use bevy::{
    ecs::message::MessageCursor,
    input::{
        ButtonState, InputPlugin,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    input_focus::{InputDispatchPlugin, InputFocusPlugin},
    math::Affine2,
    prelude::*,
    state::app::StatesPlugin,
    ui::ComputedStackIndex,
};

use crate::{
    gameplay::{
        Activate,
        presentation::{DisabledControl, OverlayRoot},
    },
    screens::Screen,
};

use super::{Control, UiInteractionPlugin};

#[derive(Resource, Default)]
struct ReceivedActivations(MessageCursor<Activate>);

fn fixture() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        InputPlugin,
        InputFocusPlugin,
        InputDispatchPlugin,
    ))
    .init_state::<Screen>()
    .add_message::<Activate>()
    .init_resource::<ReceivedActivations>()
    .add_plugins(UiInteractionPlugin);
    let window = app
        .world_mut()
        .spawn(Window {
            resolution: (375, 667).into(),
            ..default()
        })
        .id();
    app.update();
    app.update();
    (app, window)
}

fn hit_region(app: &mut App, control: Control, center: Vec2, size: Vec2, stack: u32) -> Entity {
    app.world_mut()
        .spawn((
            Button,
            control,
            Node::default(),
            ComputedNode {
                size,
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from(Affine2::from_translation(center)),
            ComputedStackIndex(stack),
            InheritedVisibility::VISIBLE,
        ))
        .id()
}

fn mouse(app: &mut App, window: Entity, position: Vec2, states: &[ButtonState]) {
    app.world_mut()
        .get_mut::<Window>(window)
        .expect("window")
        .set_cursor_position(Some(position));
    for state in states {
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: *state,
            window,
        });
    }
    app.update();
}

fn touch(app: &mut App, window: Entity, id: u64, phase: TouchPhase, position: Vec2) {
    app.world_mut().write_message(TouchInput {
        phase,
        position,
        window,
        force: None,
        id,
    });
}

fn commands(app: &mut App) -> Vec<Control> {
    app.world_mut()
        .resource_scope(|world, mut received: Mut<ReceivedActivations>| {
            received
                .0
                .read(world.resource::<Messages<Activate>>())
                .map(|message| message.0)
                .collect()
        })
}

#[test]
fn same_frame_mouse_down_and_up_emit_exactly_one_activation() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        1,
    );
    mouse(
        &mut app,
        window,
        Vec2::new(100.0, 100.0),
        &[ButtonState::Pressed, ButtonState::Released],
    );
    assert_eq!(commands(&mut app), vec![Control::Pass]);
    app.update();
    assert!(commands(&mut app).is_empty());
}

#[test]
fn release_over_another_control_does_not_activate_the_pressed_control() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        1,
    );
    hit_region(
        &mut app,
        Control::More,
        Vec2::new(200.0, 100.0),
        Vec2::new(100.0, 44.0),
        2,
    );
    mouse(
        &mut app,
        window,
        Vec2::new(148.0, 100.0),
        &[ButtonState::Pressed],
    );
    mouse(
        &mut app,
        window,
        Vec2::new(154.0, 100.0),
        &[ButtonState::Released],
    );
    assert!(commands(&mut app).is_empty());
}

#[test]
fn same_frame_touch_taps_activate_but_swipes_and_cancellations_do_not() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        1,
    );
    touch(
        &mut app,
        window,
        1,
        TouchPhase::Started,
        Vec2::new(100.0, 100.0),
    );
    touch(
        &mut app,
        window,
        1,
        TouchPhase::Ended,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    assert_eq!(commands(&mut app), vec![Control::Pass]);
    app.update();
    app.update();
    touch(
        &mut app,
        window,
        2,
        TouchPhase::Started,
        Vec2::new(100.0, 100.0),
    );
    touch(
        &mut app,
        window,
        2,
        TouchPhase::Ended,
        Vec2::new(160.0, 100.0),
    );
    app.update();
    assert!(
        app.world().resource::<super::Gesture>().dragged,
        "swipe must be classified as a drag"
    );
    assert_eq!(commands(&mut app), Vec::new());
    touch(
        &mut app,
        window,
        3,
        TouchPhase::Started,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    touch(
        &mut app,
        window,
        3,
        TouchPhase::Canceled,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    assert!(commands(&mut app).is_empty());
}

#[test]
fn another_finger_cannot_release_the_primary_pending_control() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        1,
    );
    touch(
        &mut app,
        window,
        1,
        TouchPhase::Started,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    touch(
        &mut app,
        window,
        2,
        TouchPhase::Started,
        Vec2::new(100.0, 100.0),
    );
    touch(
        &mut app,
        window,
        2,
        TouchPhase::Ended,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    assert!(commands(&mut app).is_empty());
    touch(
        &mut app,
        window,
        1,
        TouchPhase::Ended,
        Vec2::new(100.0, 100.0),
    );
    app.update();
    assert_eq!(commands(&mut app), vec![Control::Pass]);
}

#[test]
fn disabled_foreground_controls_block_underlying_activation_without_canceling() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        1,
    );
    let disabled = hit_region(
        &mut app,
        Control::PlayUnit,
        Vec2::new(100.0, 100.0),
        Vec2::new(100.0, 44.0),
        2,
    );
    app.world_mut().entity_mut(disabled).insert(DisabledControl);
    mouse(
        &mut app,
        window,
        Vec2::new(100.0, 100.0),
        &[ButtonState::Pressed, ButtonState::Released],
    );
    assert!(commands(&mut app).is_empty());
}

#[test]
fn footer_pass_stays_active_outside_nonmodal_hand_tray() {
    let (mut app, window) = fixture();
    hit_region(
        &mut app,
        Control::Pass,
        Vec2::new(100.0, 645.0),
        Vec2::new(70.0, 44.0),
        1,
    );
    app.world_mut().spawn((
        Node::default(),
        OverlayRoot { modal: false },
        ComputedNode {
            size: Vec2::new(375.0, 360.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation(Vec2::new(187.5, 430.0))),
    ));
    mouse(
        &mut app,
        window,
        Vec2::new(100.0, 645.0),
        &[ButtonState::Pressed, ButtonState::Released],
    );
    assert_eq!(commands(&mut app), vec![Control::Pass]);
}
