use bevy::prelude::*;
use low_kingdoms_rules::view::PlayerView;

use crate::{
    screens::Screen,
    ui::{Control, theme},
};

use super::{CardFace, Label, Layout, Presentation, ScrollPane, Zone, panels};

#[derive(Component)]
struct Lane(Zone);

pub(super) fn build(
    world: &mut World,
    presentation: &mut Presentation,
    screen: Screen,
    layout: Layout,
) {
    let Some(root) = presentation.root else {
        return;
    };
    world
        .resource_mut::<bevy::input_focus::InputFocus>()
        .clear();
    let scroll = presentation
        .zones
        .iter()
        .filter_map(|(zone, entity)| {
            world
                .get::<ScrollPosition>(*entity)
                .map(|position| (*zone, position.0))
        })
        .collect::<Vec<_>>();
    world.entity_mut(root).despawn_children();
    presentation.zones.clear();
    presentation.cards.clear();
    presentation.labels.clear();
    presentation.controls.clear();
    presentation.log = None;
    presentation.log_lines.clear();
    presentation.overlay = None;
    presentation.overlay_content = None;
    presentation.overlay_panel = None;
    presentation.overlay_stamp.clear();
    presentation.layout = Some(layout);
    label(
        world,
        presentation,
        root,
        Label::Status,
        15.0,
        theme::ACCENT,
    );
    label(
        world,
        presentation,
        root,
        Label::Notification,
        13.0,
        theme::SECONDARY,
    );
    match screen {
        Screen::Mulligan => mulligan(world, presentation, root, layout),
        Screen::Battle => battlefield(world, presentation, root, layout),
        Screen::Results => results(world, presentation, root),
        Screen::Title | Screen::About => {}
    }
    for (zone, position) in scroll {
        if let Some(entity) = presentation.zones.get(&zone)
            && let Some(mut scroll) = world.get_mut::<ScrollPosition>(*entity)
        {
            scroll.0 = position;
        }
    }
}

fn mulligan(world: &mut World, presentation: &mut Presentation, root: Entity, layout: Layout) {
    text(
        world,
        root,
        "Keep these six cards or replace the whole hand once.",
        16.0,
        theme::FOREGROUND,
    );
    let pane = pane(
        world,
        root,
        false,
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            width: percent(100),
            // Wrapped rows must start inside the nonnegative scroll range.
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Center,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(8),
            column_gap: px(8),
            ..default()
        },
    );
    presentation.zones.insert(Zone::Hand, pane);
    let row = node(
        world,
        root,
        Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            min_height: px(48),
            flex_shrink: 0.0,
            ..default()
        },
    );
    add_control(world, presentation, row, Control::KeepHand);
    add_control(world, presentation, row, Control::Mulligan);
    let footer = node(
        world,
        root,
        Node {
            justify_content: JustifyContent::Center,
            height: px(44),
            flex_shrink: 0.0,
            ..default()
        },
    );
    add_control(world, presentation, footer, Control::MainMenu);
    if layout.landscape {
        world
            .entity_mut(pane)
            .insert(ScrollPane { horizontal: true });
        world.entity_mut(pane).insert(Node {
            flex_grow: 1.0,
            min_height: px(0),
            width: percent(100),
            align_items: AlignItems::Start,
            // The outer cards' auto margins center a fitting hand without
            // putting overflowing cards before the scroll origin.
            justify_content: JustifyContent::Start,
            column_gap: px(8),
            overflow: Overflow::scroll_x(),
            ..default()
        });
    }
}

fn battlefield(world: &mut World, presentation: &mut Presentation, root: Entity, layout: Layout) {
    let body = node(
        world,
        root,
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            width: percent(100),
            column_gap: px(12),
            ..default()
        },
    );
    let board = node(
        world,
        body,
        Node {
            flex_grow: 1.0,
            min_width: px(0),
            height: percent(100),
            flex_direction: if layout.landscape {
                FlexDirection::Row
            } else {
                FlexDirection::Column
            },
            row_gap: px(5),
            column_gap: px(5),
            ..default()
        },
    );
    lane(world, presentation, board, Zone::Enemy, layout);
    lane(world, presentation, board, Zone::Friendly, layout);
    if !layout.compact {
        text(
            world,
            board,
            "Your hand | scroll for more cards",
            13.0,
            theme::SECONDARY,
        );
        let hand = pane(
            world,
            board,
            true,
            Node {
                height: px(118),
                min_height: px(118),
                flex_shrink: 0.0,
                width: percent(100),
                column_gap: px(8),
                ..default()
            },
        );
        presentation.zones.insert(Zone::Hand, hand);
        sidebar(world, presentation, body);
    }
    if layout.resourcing {
        let prompt = node(
            world,
            root,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                flex_shrink: 0.0,
                ..default()
            },
        );
        label(
            world,
            presentation,
            prompt,
            Label::ResourcePrompt,
            14.0,
            theme::ACCENT,
        );
        add_control(world, presentation, prompt, Control::ConfirmResources);
    }
    footer(world, presentation, root);
}

fn lane(
    world: &mut World,
    presentation: &mut Presentation,
    board: Entity,
    zone: Zone,
    layout: Layout,
) {
    let own = zone == Zone::Friendly;
    let lane = node(
        world,
        board,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_height: px(0),
            min_width: px(0),
            width: if layout.landscape {
                percent(50)
            } else {
                percent(100)
            },
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border: UiRect::all(px(1)),
            padding: UiRect::all(px(3)),
            ..default()
        },
    );
    world.entity_mut(lane).insert(Lane(zone));
    if !own {
        info(world, presentation, lane, false);
    }
    let show_hand = own && layout.compact && layout.resourcing;
    text(
        world,
        lane,
        if show_hand {
            "Select cards to resource | scroll for more"
        } else if own {
            "Your units | scroll for more"
        } else {
            "Opponent units | scroll for more"
        },
        12.0,
        theme::SECONDARY,
    );
    let cards = pane(
        world,
        lane,
        layout.compact || show_hand,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_height: px(if layout.landscape { 95 } else { 104 }),
            min_width: px(0),
            width: percent(100),
            flex_wrap: if layout.compact {
                FlexWrap::NoWrap
            } else {
                FlexWrap::Wrap
            },
            row_gap: px(7),
            column_gap: px(7),
            align_content: AlignContent::Start,
            ..default()
        },
    );
    presentation
        .zones
        .insert(if show_hand { Zone::Hand } else { zone }, cards);
    if own {
        info(world, presentation, lane, true);
    }
}

fn info(world: &mut World, presentation: &mut Presentation, parent: Entity, own: bool) {
    let row = node(
        world,
        parent,
        Node {
            width: percent(100),
            min_height: px(40),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(6),
            padding: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
    );
    world
        .entity_mut(row)
        .insert(BackgroundColor(Color::srgb(0.085, 0.105, 0.135)));
    if own {
        label(
            world,
            presentation,
            row,
            Label::OwnInfo,
            13.0,
            theme::FOREGROUND,
        );
    } else {
        let button = panels::button(
            world,
            row,
            Control::EnemyStronghold,
            "Opponent stronghold",
            false,
        );
        world.entity_mut(button).insert(CardFace);
        presentation
            .controls
            .push((Control::EnemyStronghold, button));
        let text = panels::button_text(world, button);
        if let Some(text) = text {
            presentation.labels.insert(Label::EnemyInfo, text);
        }
        if let Some(mut node) = world.get_mut::<Node>(button) {
            node.width = percent(100);
            node.height = px(44);
            node.min_height = px(44);
        }
    }
}

fn sidebar(world: &mut World, presentation: &mut Presentation, body: Entity) {
    let sidebar = node(
        world,
        body,
        Node {
            width: px(260),
            flex_shrink: 0.0,
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(8)),
            ..default()
        },
    );
    world
        .entity_mut(sidebar)
        .insert(BackgroundColor(Color::srgb(0.075, 0.090, 0.115)));
    text(world, sidebar, "Event log", 17.0, theme::FOREGROUND);
    presentation.log = Some(pane(
        world,
        sidebar,
        false,
        Node {
            flex_grow: 1.0,
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            width: percent(100),
            ..default()
        },
    ));
}

fn footer(world: &mut World, presentation: &mut Presentation, root: Entity) {
    let footer = node(
        world,
        root,
        Node {
            width: percent(100),
            height: px(46),
            min_height: px(46),
            column_gap: px(4),
            flex_shrink: 0.0,
            ..default()
        },
    );
    for control in [
        Control::Pass,
        Control::More,
        Control::Hand,
        Control::Log,
        Control::Resources,
    ] {
        let button = add_control(world, presentation, footer, control);
        if control == Control::Resources
            && let Some(text) = panels::button_text(world, button)
            && let Some(mut text) = world.get_mut::<Text>(text)
        {
            "Res.".clone_into(&mut text.0);
        }
        if let Some(mut node) = world.get_mut::<Node>(button) {
            node.min_width = px(0);
            node.flex_grow = 1.0;
            node.flex_basis = px(0);
        }
    }
}

fn results(world: &mut World, presentation: &mut Presentation, root: Entity) {
    let center = node(
        world,
        root,
        Node {
            flex_grow: 1.0,
            width: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(20),
            ..default()
        },
    );
    label(
        world,
        presentation,
        center,
        Label::Result,
        48.0,
        theme::FOREGROUND,
    );
    label(
        world,
        presentation,
        center,
        Label::Round,
        18.0,
        theme::SECONDARY,
    );
    add_control(world, presentation, center, Control::PlayAgain);
    add_control(world, presentation, center, Control::MainMenu);
}

fn add_control(
    world: &mut World,
    presentation: &mut Presentation,
    parent: Entity,
    control: Control,
) -> Entity {
    let entity = panels::button(world, parent, control, control.label(), true);
    presentation.controls.push((control, entity));
    entity
}

pub(super) fn label(
    world: &mut World,
    presentation: &mut Presentation,
    parent: Entity,
    label: Label,
    size: f32,
    color: Color,
) {
    let entity = text(world, parent, " ", size, color);
    presentation.labels.insert(label, entity);
}

pub(super) fn text(
    world: &mut World,
    parent: Entity,
    value: impl Into<String>,
    size: f32,
    color: Color,
) -> Entity {
    let entity = world
        .spawn((
            Text::new(value),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Node {
                max_width: percent(100),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .id();
    world.entity_mut(parent).add_child(entity);
    entity
}

pub(super) fn node(world: &mut World, parent: Entity, node: Node) -> Entity {
    let entity = world.spawn(node).id();
    world.entity_mut(parent).add_child(entity);
    entity
}

pub(super) fn pane(world: &mut World, parent: Entity, horizontal: bool, mut node: Node) -> Entity {
    node.overflow = if horizontal {
        Overflow {
            x: OverflowAxis::Scroll,
            y: OverflowAxis::Clip,
        }
    } else {
        Overflow {
            x: OverflowAxis::Clip,
            y: OverflowAxis::Scroll,
        }
    };
    let entity = world
        .spawn((node, ScrollPane { horizontal }, ScrollPosition::default()))
        .id();
    world.entity_mut(parent).add_child(entity);
    entity
}

pub(super) fn highlight(world: &mut World, view: &PlayerView) {
    for (lane, mut border, mut background) in world
        .query::<(&Lane, &mut BorderColor, &mut BackgroundColor)>()
        .iter_mut(world)
    {
        let own = lane.0 == Zone::Friendly;
        let active = view
            .active_player
            .is_some_and(|player| (player == view.player) == own);
        *border = BorderColor::all(if active { theme::ACCENT } else { Color::NONE });
        *background = if active {
            Color::srgb(0.085, 0.10, 0.125)
        } else {
            theme::BACKGROUND
        }
        .into();
    }
}
