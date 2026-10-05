use bevy::{
    app::AppExit,
    input_focus::{
        FocusCause, InputFocus, InputFocusVisible,
        tab_navigation::{TabIndex, TabNavigationPlugin},
    },
    prelude::*,
};

use crate::screens::{Screen, ScreenPanel};

pub(crate) mod theme;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Controls are shared with screens, diagnostics, and tests inside the client."
)]
pub(crate) enum Control {
    About,
    Back,
    Quit,
}

impl Control {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::About => "About",
            Self::Back => "Back",
            Self::Quit => "Quit",
        }
    }

    #[cfg(all(feature = "test-support", target_arch = "wasm32"))]
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::About => "about",
            Self::Back => "back",
            Self::Quit => "quit",
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Screen text roles are consumed by the crate-local responsive layout system."
)]
pub(crate) enum TextRole {
    Heading,
    Body,
    Caption,
}

impl TextRole {
    pub(crate) fn font_size(self, compact: bool, width: f32) -> f32 {
        match self {
            Self::Heading => (width * theme::HEADING_WIDTH_RATIO).clamp(
                theme::HEADING_MIN,
                if compact {
                    theme::HEADING_COMPACT
                } else {
                    theme::HEADING_MAX
                },
            ),
            Self::Body => {
                if compact {
                    theme::BODY_COMPACT_SIZE
                } else {
                    theme::BODY_SIZE
                }
            }
            Self::Caption => theme::CAPTION_SIZE,
        }
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "The application composes this private UI plugin."
)]
pub(crate) struct UiInteractionPlugin;

impl Plugin for UiInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .add_systems(
                Update,
                (activate_pointer, activate_keyboard, style_controls).chain(),
            )
            .add_systems(Update, responsive_layout);
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "The screen builders share this crate-local control construction."
)]
pub(crate) fn control(control: Control, tab_index: i32) -> impl Bundle {
    (
        Button,
        control,
        TabIndex(tab_index),
        Name::new(format!("{} control", control.label())),
        Node {
            width: percent(theme::CONTROL_WIDTH_PERCENT),
            max_width: px(theme::CONTROL_MAX_WIDTH),
            height: px(theme::CONTROL_HEIGHT),
            min_height: px(theme::CONTROL_HEIGHT),
            border: UiRect::all(px(theme::BORDER_WIDTH)),
            border_radius: BorderRadius::all(px(theme::CORNER_RADIUS)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(theme::BUTTON),
        BorderColor::all(theme::BORDER),
        children![(
            Text::new(control.label()),
            TextFont {
                font_size: FontSize::Px(theme::CONTROL_TEXT_SIZE),
                ..default()
            },
            TextColor(theme::FOREGROUND)
        )],
    )
}

fn activate_pointer(
    controls: Query<(Entity, &Control, &Interaction), Changed<Interaction>>,
    mut focus: ResMut<InputFocus>,
    mut visible: ResMut<InputFocusVisible>,
    mut next: ResMut<NextState<Screen>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, control, interaction) in &controls {
        if *interaction == Interaction::Pressed {
            focus.set(entity, FocusCause::Pressed);
            visible.0 = false;
            activate(*control, &mut next, &mut exit);
        }
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned Res system parameters."
)]
fn activate_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    controls: Query<&Control>,
    state: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) && *state.get() == Screen::About {
        next.set(Screen::Title);
    } else if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
        && let Some(entity) = focus.get()
        && let Ok(control) = controls.get(entity)
    {
        activate(*control, &mut next, &mut exit);
    }
}

fn activate(control: Control, next: &mut NextState<Screen>, exit: &mut MessageWriter<AppExit>) {
    match control {
        Control::About => next.set(Screen::About),
        Control::Back => next.set(Screen::Title),
        Control::Quit => {
            exit.write(AppExit::Success);
        }
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned Res system parameters."
)]
fn style_controls(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    mut controls: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        With<Control>,
    >,
) {
    for (entity, interaction, mut background, mut border) in &mut controls {
        *background = match interaction {
            Interaction::Pressed => theme::PRESSED,
            Interaction::Hovered => theme::HOVER,
            Interaction::None => theme::BUTTON,
        }
        .into();
        *border = BorderColor::all(if focus.get() == Some(entity) && visible.0 {
            theme::ACCENT
        } else {
            theme::BORDER
        });
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects the owned Single system parameter."
)]
fn responsive_layout(
    window: Single<&Window>,
    mut panels: Query<&mut Node, With<ScreenPanel>>,
    mut text: Query<(&TextRole, &mut TextFont)>,
) {
    let compact = window.height() < theme::COMPACT_HEIGHT || window.width() < theme::COMPACT_WIDTH;
    for mut panel in &mut panels {
        let gap = px(if compact {
            theme::COMPACT_GAP
        } else {
            theme::GAP
        });
        if panel.row_gap != gap {
            panel.row_gap = gap;
        }
    }
    for (role, mut font) in &mut text {
        let size = FontSize::Px(role.font_size(compact, window.width()));
        if font.font_size != size {
            font.font_size = size;
        }
    }
}
