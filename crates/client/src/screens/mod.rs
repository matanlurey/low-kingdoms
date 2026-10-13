use bevy::{
    input_focus::{InputFocus, tab_navigation::TabGroup},
    prelude::*,
};

use crate::{
    platform::PresentationPlatform,
    ui::{Control, TextRole, theme},
};

#[derive(States, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Screen state is shared within the client, never a public domain API."
)]
pub(crate) enum Screen {
    #[default]
    Title,
    About,
    Mulligan,
    Battle,
    Results,
}

#[derive(Component)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Screen ownership is inspected by crate-local lifecycle tests."
)]
pub(crate) struct ScreenRoot;

#[derive(Component)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The shared UI layout system uses this crate-local marker."
)]
pub(crate) struct ScreenPanel;

#[expect(
    clippy::redundant_pub_crate,
    reason = "The application composes this private screen plugin."
)]
pub(crate) struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .add_systems(OnEnter(Screen::Title), (spawn_title, clear_focus).chain())
            .add_systems(OnEnter(Screen::About), (spawn_about, clear_focus).chain())
            .add_systems(
                PostStartup,
                clear_focus.after(bevy::input_focus::set_initial_focus),
            );
    }
}

fn clear_focus(mut focus: ResMut<InputFocus>) {
    // Tabbing begins with the first control, without retaining a despawned screen's focus.
    focus.clear();
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned Res system parameters."
)]
fn spawn_title(
    mut commands: Commands,
    platform: Res<PresentationPlatform>,
    battle: Res<crate::gameplay::Battle>,
) {
    let root = root(&mut commands, Screen::Title);
    commands.entity(root).with_children(|parent| {
        parent.spawn(panel()).with_children(|panel| {
            panel.spawn(text("Low Kingdoms", TextRole::Heading));
            panel.spawn(text("Early development build", TextRole::Body));
            panel.spawn(divider());
            panel.spawn(crate::ui::control(Control::Play, 0));
            if battle.resume_available {
                panel.spawn(crate::ui::control(Control::Resume, 1));
            }
            panel.spawn(crate::ui::control(Control::About, 2));
            if platform.supports_quit {
                panel.spawn(crate::ui::control(Control::Quit, 3));
            }
            panel.spawn(text("Tab to move / Enter to select", TextRole::Caption));
        });
    });
}

fn spawn_about(mut commands: Commands) {
    let root = root(&mut commands, Screen::About);
    commands.entity(root).with_children(|parent| {
        parent.spawn(panel()).with_children(|panel| {
            panel.spawn(text("Low Kingdoms", TextRole::Heading));
            panel.spawn(text(
                format!("Version {}", env!("CARGO_PKG_VERSION")),
                TextRole::Caption,
            ));
            panel.spawn(divider());
            panel.spawn(text(
                "Playable battle prototype against a basic AI.",
                TextRole::Body,
            ));
            panel.spawn(crate::ui::control(Control::Back, 0));
            panel.spawn(text("Escape to return", TextRole::Caption));
        });
    });
}

fn root(commands: &mut Commands, screen: Screen) -> Entity {
    commands
        .spawn((
            ScreenRoot,
            DespawnOnExit(screen),
            Name::new(format!("{screen:?} screen root")),
            TabGroup::new(0),
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(theme::SCREEN_PADDING)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .id()
}

fn panel() -> impl Bundle {
    (
        ScreenPanel,
        Name::new("Screen content"),
        Node {
            width: percent(100),
            max_width: px(theme::PANEL_MAX_WIDTH),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(theme::GAP),
            ..default()
        },
    )
}

fn divider() -> impl Bundle {
    (
        Name::new("Section divider"),
        Node {
            width: px(theme::DIVIDER_WIDTH),
            height: px(theme::DIVIDER_HEIGHT),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(theme::ACCENT),
    )
}

fn text(value: impl Into<String>, role: TextRole) -> impl Bundle {
    (
        Text::new(value),
        role,
        TextFont {
            font_size: FontSize::Px(role.font_size(false, theme::PANEL_MAX_WIDTH)),
            ..default()
        },
        TextColor(if matches!(role, TextRole::Heading) {
            theme::FOREGROUND
        } else {
            theme::SECONDARY
        }),
        TextLayout::justify(Justify::Center),
        Node {
            max_width: percent(100),
            flex_shrink: 0.0,
            ..default()
        },
        Name::new(format!("{role:?} text")),
    )
}
