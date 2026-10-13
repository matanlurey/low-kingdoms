use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
    input::touch::{Touch, TouchInput, TouchPhase},
    input_focus::{
        FocusCause, InputFocus, InputFocusVisible,
        tab_navigation::{TabIndex, TabNavigationPlugin},
    },
    prelude::*,
    ui::ComputedStackIndex,
};

use crate::screens::{Screen, ScreenPanel};

pub(crate) mod theme;

#[cfg(test)]
mod input_tests;

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
    empty: bool,
    pointer: Option<Pointer>,
    release_position: Option<Vec2>,
}

#[derive(Clone, Copy)]
enum Pointer {
    Mouse,
    Touch(u64),
}

impl Pointer {
    fn position(self, window: &Window, touches: &Touches) -> Option<Vec2> {
        match self {
            Self::Mouse => window.cursor_position(),
            Self::Touch(id) => touches
                .get_pressed(id)
                .or_else(|| touches.get_released(id))
                .map(Touch::position),
        }
    }

    fn started(self, mouse: &ButtonInput<MouseButton>, touches: &Touches) -> bool {
        match self {
            Self::Mouse => mouse.just_pressed(MouseButton::Left),
            Self::Touch(id) => touches.just_pressed(id),
        }
    }

    fn released(self, mouse: &ButtonInput<MouseButton>, touches: &Touches) -> bool {
        match self {
            Self::Mouse => mouse.just_released(MouseButton::Left),
            Self::Touch(id) => touches.just_released(id),
        }
    }
}

type OverlayNodes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static crate::gameplay::presentation::OverlayRoot,
        &'static ComputedNode,
        &'static UiGlobalTransform,
    ),
>;

type ScrollPanes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static crate::gameplay::presentation::ScrollPane,
        &'static mut ScrollPosition,
    ),
>;

#[derive(SystemParam)]
struct ScrollInput<'w, 's> {
    window: Single<'w, 's, &'static Window>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    wheel: MessageReader<'w, 's, bevy::input::mouse::MouseWheel>,
    touch_events: MessageReader<'w, 's, TouchInput>,
    gesture: ResMut<'w, Gesture>,
    panes: ScrollPanes<'w, 's>,
    parents: Query<'w, 's, &'static ChildOf>,
    overlays: OverlayNodes<'w, 's>,
}

fn scroll_input(data: ScrollInput) {
    let ScrollInput {
        window,
        mouse,
        touches,
        mut wheel,
        mut touch_events,
        mut gesture,
        mut panes,
        parents,
        overlays,
    } = data;
    if gesture.pointer.is_none() {
        let contact = touches
            .iter_just_pressed()
            .min_by_key(|touch| touch.id())
            .map(|touch| (Pointer::Touch(touch.id()), touch.start_position()))
            .or_else(|| {
                mouse
                    .just_pressed(MouseButton::Left)
                    .then(|| {
                        window
                            .cursor_position()
                            .map(|position| (Pointer::Mouse, position))
                    })
                    .flatten()
            });
        if let Some((pointer, origin)) = contact {
            gesture.pointer = Some(pointer);
            gesture.origin = Some(origin);
            gesture.previous = Some(origin);
            gesture.dragged = false;
            gesture.pending = None;
            gesture.release_position = None;
            gesture.pane = panes
                .iter()
                .filter(|(entity, node, transform, _, _)| {
                    active_surface(*entity, &parents, &overlays, origin)
                        && logical_bounds(node, transform).contains(origin)
                })
                .min_by_key(|(_, node, _, _, _)| (node.size().x * node.size().y).to_bits())
                .map(|(entity, ..)| entity);
        }
    }
    for event in touch_events.read() {
        if matches!(gesture.pointer, Some(Pointer::Touch(id)) if id == event.id)
            && event.phase == TouchPhase::Ended
        {
            gesture.release_position = Some(event.position);
        }
    }
    let position = gesture.pointer.map_or_else(
        || window.cursor_position(),
        |pointer| {
            gesture
                .release_position
                .or_else(|| pointer.position(&window, &touches))
        },
    );
    if gesture.pointer.is_some()
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
            && let Ok((_, node, _, pane, mut scroll)) = panes.get_mut(entity)
        {
            let delta = previous - position;
            if pane.horizontal {
                scroll_by(node, true, &mut scroll, delta.x);
            } else {
                scroll_by(node, false, &mut scroll, delta.y);
            }
        }
        gesture.previous = Some(position);
    }
    scroll_wheel(&mut wheel, position, &mut panes, &parents, &overlays);
}

fn scroll_wheel(
    wheel: &mut MessageReader<bevy::input::mouse::MouseWheel>,
    position: Option<Vec2>,
    panes: &mut ScrollPanes,
    parents: &Query<&ChildOf>,
    overlays: &OverlayNodes,
) {
    for event in wheel.read() {
        if let Some(position) = position {
            for (entity, node, transform, pane, mut scroll) in panes.iter_mut() {
                if !active_surface(entity, parents, overlays, position) {
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
                        scroll_by(node, true, &mut scroll, -(event.y + event.x) * factor);
                    } else {
                        scroll_by(node, false, &mut scroll, -event.y * factor);
                    }
                }
            }
        }
    }
}

fn scroll_by(node: &ComputedNode, horizontal: bool, scroll: &mut ScrollPosition, delta: f32) {
    // Bevy clamps its rendered offset but leaves ScrollPosition unchanged. Bound the input
    // value too, so reversing at an edge never has to consume invisible excess movement.
    let limit = (node.content_size() - node.size() + node.scrollbar_size).max(Vec2::ZERO)
        * node.inverse_scale_factor();
    if horizontal {
        scroll.x = (scroll.x.clamp(0.0, limit.x) + delta).clamp(0.0, limit.x);
    } else {
        scroll.y = (scroll.y.clamp(0.0, limit.y) + delta).clamp(0.0, limit.y);
    }
}

fn active_surface(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    overlays: &OverlayNodes,
    position: Vec2,
) -> bool {
    loop {
        if overlays.get(entity).is_ok() {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            break;
        };
        entity = parent.parent();
    }
    overlays.iter().all(|(_, overlay, node, transform)| {
        !overlay.modal && !logical_bounds(node, transform).contains(position)
    })
}

fn logical_bounds(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    Rect::from_center_size(
        transform.translation * node.inverse_scale_factor(),
        node.size() * node.inverse_scale_factor(),
    )
}

type PointerControls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Control,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        Option<&'static CalculatedClip>,
        &'static ComputedStackIndex,
        Option<&'static InheritedVisibility>,
        Option<&'static crate::gameplay::presentation::DisabledControl>,
    ),
>;

#[derive(SystemParam)]
struct PointerContext<'w, 's> {
    focus: ResMut<'w, InputFocus>,
    visible: ResMut<'w, InputFocusVisible>,
    next: ResMut<'w, NextState<Screen>>,
    exit: MessageWriter<'w, AppExit>,
    messages: MessageWriter<'w, crate::gameplay::Activate>,
    gesture: ResMut<'w, Gesture>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    window: Single<'w, 's, &'static Window>,
    parents: Query<'w, 's, &'static ChildOf>,
    overlays: OverlayNodes<'w, 's>,
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
        window,
        parents,
        overlays,
    } = context;
    let Some(pointer) = gesture.pointer else {
        return;
    };
    if let Pointer::Touch(id) = pointer
        && touches.just_canceled(id)
    {
        gesture.pointer = None;
        gesture.pending = None;
        return;
    }
    if pointer.started(&mouse, &touches)
        && let Some(origin) = gesture.origin
    {
        let hit = hit_control(&controls, &parents, &overlays, origin);
        gesture.empty = hit.is_none();
        gesture.pending = hit
            .filter(|(_, _, enabled)| *enabled)
            .map(|(entity, control, _)| (entity, control));
        if let Some((entity, _)) = gesture.pending {
            focus.set(entity, FocusCause::Pressed);
            visible.0 = false;
        }
    }
    if pointer.released(&mouse, &touches) {
        let hit = gesture
            .release_position
            .or_else(|| pointer.position(&window, &touches))
            .and_then(|position| hit_control(&controls, &parents, &overlays, position));
        if !gesture.dragged {
            if let (Some((pressed, control)), Some((released, _, true))) =
                (gesture.pending.take(), hit)
            {
                if pressed == released {
                    activate(control, &mut next, &mut exit, &mut messages);
                }
            } else if gesture.empty && hit.is_none() {
                messages.write(crate::gameplay::Activate(Control::Close));
            }
        }
        gesture.pointer = None;
        gesture.pending = None;
        gesture.origin = None;
    }
}

fn hit_control(
    controls: &PointerControls,
    parents: &Query<&ChildOf>,
    overlays: &OverlayNodes,
    point: Vec2,
) -> Option<(Entity, Control, bool)> {
    controls
        .iter()
        .filter_map(
            |(entity, control, node, transform, clip, stack, visibility, disabled)| {
                if visibility.is_some_and(|visibility| !visibility.get())
                    || !active_surface(entity, parents, overlays, point)
                {
                    return None;
                }
                let bounds = logical_bounds(node, transform);
                let clipped = clip.map_or(bounds, |clip| {
                    bounds.intersect(Rect::from_corners(
                        clip.clip.min * node.inverse_scale_factor(),
                        clip.clip.max * node.inverse_scale_factor(),
                    ))
                });
                (clipped.width() > 0.0 && clipped.height() > 0.0 && clipped.contains(point))
                    .then_some((stack.0, entity, *control, disabled.is_none()))
            },
        )
        .max_by_key(|(stack, entity, ..)| (*stack, entity.to_bits()))
        .map(|(_, entity, control, enabled)| (entity, control, enabled))
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
