use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
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
    Play,
    Resume,
    ConfirmReplace,
    Cancel,
    KeepHand,
    Mulligan,
    ConfirmResources,
    Pass,
    More,
    Hand,
    Log,
    Resources,
    Close,
    PlayUnit,
    AttackUnit,
    ClaimInitiative,
    SpeedNormal,
    SpeedFast,
    SpeedInstant,
    PlayAgain,
    MainMenu,
    Retry,
    Card(low_kingdoms_rules::model::CardId),
    EnemyStronghold,
}

impl Control {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::About => "About",
            Self::Back => "Back",
            Self::Quit => "Quit",
            Self::Play => "Play",
            Self::Resume => "Resume Battle",
            Self::ConfirmReplace => "Replace saved battle",
            Self::Cancel => "Cancel",
            Self::KeepHand => "Keep Hand",
            Self::Mulligan => "Mulligan",
            Self::ConfirmResources => "Confirm Resources",
            Self::Pass => "Pass",
            Self::More => "More",
            Self::Hand => "Hand",
            Self::Log => "Log",
            Self::Resources => "Resources",
            Self::Close => "Close",
            Self::PlayUnit => "Play Unit",
            Self::AttackUnit => "Attack",
            Self::ClaimInitiative => "Claim Initiative",
            Self::SpeedNormal => "Normal (750 ms)",
            Self::SpeedFast => "Fast (150 ms)",
            Self::SpeedInstant => "Instant",
            Self::PlayAgain => "Play Again",
            Self::MainMenu => "Main Menu",
            Self::Retry => "Retry",
            Self::Card(_) => "Inspect card",
            Self::EnemyStronghold => "Enemy Stronghold",
        }
    }

    #[cfg(all(feature = "test-support", target_arch = "wasm32"))]
    pub(crate) fn id(self) -> String {
        match self {
            Self::Card(id) => return format!("card-{}", id.0),
            Self::About => "about",
            Self::Back => "back",
            Self::Quit => "quit",
            Self::Play => "play",
            Self::Resume => "resume",
            Self::ConfirmReplace => "confirm-replace",
            Self::Cancel => "cancel",
            Self::KeepHand => "keep-hand",
            Self::Mulligan => "mulligan",
            Self::ConfirmResources => "confirm-resources",
            Self::Pass => "pass",
            Self::More => "more",
            Self::Hand => "hand",
            Self::Log => "log",
            Self::Resources => "resources",
            Self::Close => "close",
            Self::PlayUnit => "play-unit",
            Self::AttackUnit => "attack-unit",
            Self::ClaimInitiative => "claim-initiative",
            Self::SpeedNormal => "speed-normal",
            Self::SpeedFast => "speed-fast",
            Self::SpeedInstant => "speed-instant",
            Self::PlayAgain => "play-again",
            Self::MainMenu => "main-menu",
            Self::Retry => "retry",
            Self::EnemyStronghold => "enemy-stronghold",
        }
        .to_owned()
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
            .init_resource::<Gesture>()
            .add_systems(
                Update,
                (
                    scroll_input,
                    activate_pointer,
                    activate_keyboard,
                    style_controls,
                )
                    .chain(),
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

#[derive(Resource, Default)]
struct Gesture {
    pending: Option<(Entity, Control)>,
    origin: Option<Vec2>,
    previous: Option<Vec2>,
    pane: Option<Entity>,
    dragged: bool,
}

#[derive(SystemParam)]
struct ScrollInput<'w, 's> {
    window: Single<'w, 's, &'static Window>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    wheel: MessageReader<'w, 's, bevy::input::mouse::MouseWheel>,
    gesture: ResMut<'w, Gesture>,
    panes: Query<
        'w,
        's,
        (
            Entity,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static crate::gameplay::presentation::ScrollPane,
            &'static mut ScrollPosition,
        ),
    >,
    parents: Query<'w, 's, &'static ChildOf>,
    overlays: Query<'w, 's, Entity, With<crate::gameplay::presentation::OverlayRoot>>,
}

fn scroll_input(data: ScrollInput) {
    let ScrollInput {
        window,
        mouse,
        touches,
        mut wheel,
        mut gesture,
        mut panes,
        parents,
        overlays,
    } = data;
    let position = touches
        .first_pressed_position()
        .or_else(|| window.cursor_position());
    let started = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    if started {
        gesture.origin = position;
        gesture.previous = position;
        gesture.dragged = false;
        gesture.pane = position.and_then(|position| {
            panes
                .iter()
                .filter(|(entity, node, transform, _, _)| {
                    if !active_surface(*entity, &parents, &overlays) {
                        return false;
                    }
                    let center = transform.translation * node.inverse_scale_factor();
                    Rect::from_center_size(center, node.size() * node.inverse_scale_factor())
                        .contains(position)
                })
                .min_by_key(|(_, node, _, _, _)| (node.size().x * node.size().y).to_bits())
                .map(|(entity, ..)| entity)
        });
    }
    if (mouse.pressed(MouseButton::Left) || touches.iter().next().is_some())
        && let Some(position) = position
    {
        if gesture
            .origin
            .is_some_and(|origin| origin.distance(position) > 8.0)
        {
            gesture.dragged = true;
        }
        if gesture.dragged
            && let (Some(entity), Some(previous)) = (gesture.pane, gesture.previous)
            && let Ok((_, _, _, pane, mut scroll)) = panes.get_mut(entity)
        {
            let delta = previous - position;
            if pane.horizontal {
                scroll.x = (scroll.x + delta.x).max(0.0);
            } else {
                scroll.y = (scroll.y + delta.y).max(0.0);
            }
        }
        gesture.previous = Some(position);
    }
    for event in wheel.read() {
        if let Some(position) = position {
            for (entity, node, transform, pane, mut scroll) in &mut panes {
                if !active_surface(entity, &parents, &overlays) {
                    continue;
                }
                let center = transform.translation * node.inverse_scale_factor();
                if Rect::from_center_size(center, node.size() * node.inverse_scale_factor())
                    .contains(position)
                {
                    let factor = if matches!(event.unit, bevy::input::mouse::MouseScrollUnit::Line)
                    {
                        36.0
                    } else {
                        1.0
                    };
                    if pane.horizontal {
                        scroll.x = (event.y + event.x).mul_add(-factor, scroll.x).max(0.0);
                    } else {
                        scroll.y = event.y.mul_add(-factor, scroll.y).max(0.0);
                    }
                }
            }
        }
    }
}

fn active_surface(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    overlays: &Query<Entity, With<crate::gameplay::presentation::OverlayRoot>>,
) -> bool {
    if overlays.is_empty() {
        return true;
    }
    loop {
        if overlays.get(entity).is_ok() {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
}

type PointerControls<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Control, &'static Interaction),
    (
        Changed<Interaction>,
        Without<crate::gameplay::presentation::DisabledControl>,
    ),
>;

#[derive(SystemParam)]
struct PointerContext<'w> {
    focus: ResMut<'w, InputFocus>,
    visible: ResMut<'w, InputFocusVisible>,
    next: ResMut<'w, NextState<Screen>>,
    exit: MessageWriter<'w, AppExit>,
    messages: MessageWriter<'w, crate::gameplay::Activate>,
    gesture: ResMut<'w, Gesture>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
}

fn activate_pointer(controls: PointerControls, context: PointerContext) {
    let PointerContext {
        mut focus,
        mut visible,
        mut next,
        mut exit,
        mut messages,
        mut gesture,
        mouse,
        touches,
    } = context;
    for (entity, control, interaction) in &controls {
        if *interaction == Interaction::Pressed {
            focus.set(entity, FocusCause::Pressed);
            visible.0 = false;
            if matches!(control, Control::About | Control::Back | Control::Quit) {
                activate(*control, &mut next, &mut exit, &mut messages);
            } else {
                gesture.pending = Some((entity, *control));
            }
        }
    }
    if mouse.just_released(MouseButton::Left) || touches.iter_just_released().next().is_some() {
        if !gesture.dragged {
            if let Some((_, control)) = gesture.pending.take() {
                activate(control, &mut next, &mut exit, &mut messages);
            } else {
                messages.write(crate::gameplay::Activate(Control::Close));
            }
        }
        gesture.pending = None;
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned Res system parameters."
)]
fn activate_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    controls: Query<&Control, Without<crate::gameplay::presentation::DisabledControl>>,
    state: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
    mut exit: MessageWriter<AppExit>,
    mut messages: MessageWriter<crate::gameplay::Activate>,
) {
    if keys.just_pressed(KeyCode::Escape) && *state.get() == Screen::About {
        next.set(Screen::Title);
    } else if keys.just_pressed(KeyCode::Escape)
        && matches!(
            state.get(),
            Screen::Battle | Screen::Mulligan | Screen::Results
        )
    {
        messages.write(crate::gameplay::Activate(Control::Close));
    } else if (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space))
        && let Some(entity) = focus.get()
        && let Ok(control) = controls.get(entity)
    {
        activate(*control, &mut next, &mut exit, &mut messages);
    }
}

fn activate(
    control: Control,
    next: &mut NextState<Screen>,
    exit: &mut MessageWriter<AppExit>,
    messages: &mut MessageWriter<crate::gameplay::Activate>,
) {
    match control {
        Control::About => next.set(Screen::About),
        Control::Back => next.set(Screen::Title),
        Control::Quit => {
            exit.write(AppExit::Success);
        }
        _ => {
            messages.write(crate::gameplay::Activate(control));
        }
    }
}

type StyledControls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Interaction,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
        Option<&'static crate::gameplay::presentation::DisabledControl>,
    ),
    (
        With<Control>,
        Without<crate::gameplay::presentation::CardFace>,
    ),
>;

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned resource borrows."
)]
fn style_controls(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    mut controls: StyledControls,
) {
    for (entity, interaction, mut background, mut border, disabled) in &mut controls {
        *background = if disabled.is_some() {
            theme::BACKGROUND
        } else {
            match interaction {
                Interaction::Pressed => theme::PRESSED,
                Interaction::Hovered => theme::HOVER,
                Interaction::None => theme::BUTTON,
            }
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
