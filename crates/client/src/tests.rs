use bevy::{
    app::AppExit,
    input::{
        ButtonState, InputPlugin,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{InputDispatchPlugin, InputFocus, InputFocusPlugin, InputFocusVisible},
    prelude::*,
    state::app::StatesPlugin,
    window::PrimaryWindow,
};

use crate::{
    app::ClientPlugin,
    platform::PresentationPlatform,
    screens::{Screen, ScreenRoot},
    ui::Control,
};

fn test_app(supports_quit: bool) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        InputPlugin,
        InputFocusPlugin,
        InputDispatchPlugin,
    ))
    .init_resource::<Assets<Font>>()
    .insert_resource(PresentationPlatform { supports_quit })
    .add_plugins(ClientPlugin);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app.update();
    app.update();
    app
}

fn control_entity(app: &mut App, control: Control) -> Entity {
    app.world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find(|(_, found)| **found == control)
        .map(|(entity, _)| entity)
        .unwrap()
}

fn press_control(app: &mut App, control: Control) {
    let entity = control_entity(app, control);
    *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
    app.update();
    app.update();
}

fn key(app: &mut App, code: KeyCode, state: ButtonState) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .unwrap();
    app.world_mut().write_message(KeyboardInput {
        key_code: code,
        logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window,
    });
    app.update();
}

fn tap(app: &mut App, code: KeyCode) {
    key(app, code, ButtonState::Pressed);
    key(app, code, ButtonState::Released);
    app.update();
}

fn assert_screen(app: &mut App, screen: Screen) {
    assert_eq!(*app.world().resource::<State<Screen>>().get(), screen);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<ScreenRoot>>()
            .iter(app.world())
            .count(),
        1
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Camera2d>>()
            .iter(app.world())
            .count(),
        1
    );
}

#[test]
fn initializes_one_title_with_real_controls_and_text() {
    let mut app = test_app(true);
    assert_screen(&mut app, Screen::Title);
    let texts: Vec<_> = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.as_str())
        .collect();
    assert!(texts.contains(&"Low Kingdoms"));
    assert!(texts.contains(&"Early development build"));
    assert!(texts.contains(&"About"));
    assert!(texts.contains(&"Quit"));
    assert_eq!(
        app.world_mut()
            .query::<&Control>()
            .iter(app.world())
            .count(),
        2
    );
}

#[test]
fn production_navigation_repeatedly_cleans_screen_descendants() {
    let mut app = test_app(true);
    let nodes = |app: &mut App| {
        app.world_mut()
            .query_filtered::<Entity, With<Node>>()
            .iter(app.world())
            .collect::<Vec<_>>()
    };
    let title_nodes = nodes(&mut app).len();
    // Bevy resources use entities; PreviousState is created on the first exit.
    let live_entities = |app: &mut App| {
        app.world_mut()
            .query_filtered::<Entity, Without<bevy::ecs::resource::IsResource>>()
            .iter(app.world())
            .count()
    };
    let title_entities = live_entities(&mut app);
    for _ in 0..12 {
        let old_title = nodes(&mut app);
        press_control(&mut app, Control::About);
        assert_screen(&mut app, Screen::About);
        assert!(
            old_title
                .iter()
                .all(|entity| app.world().get_entity(*entity).is_err())
        );
        assert_eq!(
            app.world_mut()
                .query::<&Control>()
                .iter(app.world())
                .count(),
            1
        );
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .any(|text| text.0 == "Early development build. Gameplay is not implemented.")
        );
        let old_about = nodes(&mut app);
        press_control(&mut app, Control::Back);
        assert_screen(&mut app, Screen::Title);
        assert!(
            old_about
                .iter()
                .all(|entity| app.world().get_entity(*entity).is_err())
        );
        assert_eq!(nodes(&mut app).len(), title_nodes);
        assert_eq!(live_entities(&mut app), title_entities);
    }
}

#[test]
fn browser_configuration_has_no_quit_control() {
    let mut app = test_app(false);
    assert!(
        app.world_mut()
            .query::<&Control>()
            .iter(app.world())
            .all(|control| *control != Control::Quit)
    );
    press_control(&mut app, Control::About);
    press_control(&mut app, Control::Back);
    assert!(
        app.world_mut()
            .query::<&Control>()
            .iter(app.world())
            .all(|control| *control != Control::Quit)
    );
}

#[test]
fn actual_keyboard_tab_shift_tab_and_activation_navigate() {
    let mut app = test_app(true);
    tap(&mut app, KeyCode::Tab);
    let about = control_entity(&mut app, Control::About);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(about));
    assert!(app.world().resource::<InputFocusVisible>().0);
    key(&mut app, KeyCode::ShiftLeft, ButtonState::Pressed);
    tap(&mut app, KeyCode::Tab);
    key(&mut app, KeyCode::ShiftLeft, ButtonState::Released);
    let quit = control_entity(&mut app, Control::Quit);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(quit));
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Enter);
    assert_screen(&mut app, Screen::About);
    tap(&mut app, KeyCode::Tab);
    tap(&mut app, KeyCode::Space);
    assert_screen(&mut app, Screen::Title);
}

#[test]
fn escape_returns_about_without_quitting_title_and_quit_requests_exit() {
    let mut app = test_app(true);
    tap(&mut app, KeyCode::Escape);
    assert_screen(&mut app, Screen::Title);
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    press_control(&mut app, Control::About);
    tap(&mut app, KeyCode::Escape);
    assert_screen(&mut app, Screen::Title);
    press_control(&mut app, Control::Quit);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}
