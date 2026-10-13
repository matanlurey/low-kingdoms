use bevy::{
    input_focus::{
        FocusCause, InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    prelude::*,
};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    model::{Card, Phase},
};

use crate::ui::{Control, theme};

use super::super::{Battle, Panel, Speed};
use super::{DisabledControl, Layout, OverlayRoot, Presentation, layout, visible_card};

pub(super) fn reconcile(
    world: &mut World,
    presentation: &mut Presentation,
    battle: &Battle,
    layout: Layout,
) {
    let stamp = format!(
        "{:?}/{:?}/{:?}/{:?}/{}/{:?}/{}",
        battle.panel,
        battle.selected,
        battle.resource_selection,
        battle.speed,
        battle.revision,
        battle.error,
        layout.compact
    );
    if presentation.overlay_stamp == stamp {
        return;
    }
    let position = if presentation.overlay_panel == Some(battle.panel) {
        presentation
            .overlay_content
            .and_then(|entity| world.get::<ScrollPosition>(entity))
            .map(|position| position.0)
    } else {
        None
    };
    if let Some(entity) = presentation.overlay.take() {
        world.despawn(entity);
    }
    presentation.overlay_content = None;
    presentation.overlay_panel = None;
    presentation.overlay_stamp = stamp;
    if battle.panel == Panel::None {
        return;
    }
    let Some(root) = presentation.root else {
        return;
    };
    let (overlay, content, close) = surface(world, root, battle.panel);
    if let Some(position) = position
        && let Some(mut scroll) = world.get_mut::<ScrollPosition>(content)
    {
        scroll.0 = position;
    }
    match battle.panel {
        Panel::More => more(world, content, battle),
        Panel::Hand => hand(world, content, battle),
        Panel::Log => log(world, content, battle),
        Panel::Resources => resources(world, content, battle),
        Panel::Inspect => inspect(world, content, battle),
        Panel::Replace => replace(world, content, battle),
        Panel::Error => error(world, content, battle),
        Panel::None => {}
    }
    presentation.overlay = Some(overlay);
    presentation.overlay_content = Some(content);
    presentation.overlay_panel = Some(battle.panel);
    if let Some(mut focus) = world.get_resource_mut::<InputFocus>() {
        focus.set(close, FocusCause::Navigated);
    }
}

fn surface(world: &mut World, root: Entity, kind: Panel) -> (Entity, Entity, Entity) {
    let overlay = layout::node(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    );
    world.entity_mut(overlay).insert((
        OverlayRoot,
        bevy::ui::FocusPolicy::Block,
        GlobalZIndex(20),
        BackgroundColor(Color::srgba(0.025, 0.035, 0.05, 0.97)),
        Name::new("Battle overlay"),
    ));
    let panel = layout::node(
        world,
        overlay,
        Node {
            width: percent(100),
            max_width: px(600),
            max_height: percent(96),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
    );
    world.entity_mut(panel).insert((
        TabGroup::modal(),
        BackgroundColor(theme::BACKGROUND),
        BorderColor::all(theme::BORDER),
    ));
    let close = header(world, panel, kind);
    let content = layout::pane(
        world,
        panel,
        false,
        Node {
            width: percent(100),
            min_height: px(0),
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
    );
    (overlay, content, close)
}

fn header(world: &mut World, parent: Entity, panel: Panel) -> Entity {
    let row = layout::node(
        world,
        parent,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(8),
            flex_shrink: 0.0,
            ..default()
        },
    );
    let title = match panel {
        Panel::More => "Battle menu",
        Panel::Hand => "Your hand",
        Panel::Log => "Event log",
        Panel::Resources => "Resources and discard",
        Panel::Inspect => "Card inspection",
        Panel::Replace => "Replace saved battle?",
        Panel::Error => "Battle paused",
        Panel::None => "Battle",
    };
    let title = layout::text(world, row, title, 18.0, theme::FOREGROUND);
    if let Some(mut node) = world.get_mut::<Node>(title) {
        node.flex_grow = 1.0;
    }
    button(world, row, Control::Close, "Close", true)
}

fn more(world: &mut World, parent: Entity, battle: &Battle) {
    contextual_button(world, parent, Control::ClaimInitiative, battle);
    layout::text(
        world,
        parent,
        "Claiming initiative ends your actions this round.",
        14.0,
        theme::SECONDARY,
    );
    layout::text(
        world,
        parent,
        format!("AI speed: {:?}", battle.speed),
        16.0,
        theme::FOREGROUND,
    );
    let row = layout::node(
        world,
        parent,
        Node {
            width: percent(100),
            column_gap: px(5),
            ..default()
        },
    );
    for (control, speed, label) in [
        (Control::SpeedNormal, Speed::Normal, "Normal\n750 ms"),
        (Control::SpeedFast, Speed::Fast, "Fast\n150 ms"),
        (Control::SpeedInstant, Speed::Instant, "Instant"),
    ] {
        let entity = button(world, row, control, label, true);
        if let Some(mut node) = world.get_mut::<Node>(entity) {
            node.flex_grow = 1.0;
            node.flex_basis = px(0);
            node.min_width = px(44);
        }
        if battle.speed == speed {
            world
                .entity_mut(entity)
                .insert(BorderColor::all(theme::ACCENT));
        }
    }
    button(world, parent, Control::MainMenu, "Main Menu", true);
}

fn hand(world: &mut World, parent: Entity, battle: &Battle) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    if view.hand.is_empty() {
        layout::text(world, parent, "Your hand is empty.", 16.0, theme::SECONDARY);
    }
    for card in &view.hand {
        card_row(world, parent, *card, battle, "Inspect");
    }
    if matches!(view.phase, Phase::InitialResourcing | Phase::Regroup) {
        contextual_button(world, parent, Control::ConfirmResources, battle);
    }
}

fn log(world: &mut World, parent: Entity, battle: &Battle) {
    if battle.history.is_empty() {
        layout::text(
            world,
            parent,
            "Battle events will appear here.",
            15.0,
            theme::SECONDARY,
        );
    }
    for entry in &battle.history {
        layout::text(world, parent, entry.clone(), 14.0, theme::SECONDARY);
    }
}

fn resources(world: &mut World, parent: Entity, battle: &Battle) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    let own = &view.players[0].resources;
    let enemy = &view.players[1].resources;
    layout::text(
        world,
        parent,
        format!(
            "Your resources: {}/{} ready\nOpponent resources: {}/{} ready",
            own.ready, own.total, enemy.ready, enemy.total
        ),
        16.0,
        theme::FOREGROUND,
    );
    for resource in &view.resources {
        card_row(
            world,
            parent,
            resource.card,
            battle,
            if resource.ready {
                "Ready resource"
            } else {
                "Exhausted resource"
            },
        );
    }
    for (index, player) in view.players.iter().enumerate() {
        layout::text(
            world,
            parent,
            if index == 0 {
                "Your discard"
            } else {
                "Opponent discard"
            },
            16.0,
            theme::FOREGROUND,
        );
        if player.discard.is_empty() {
            layout::text(world, parent, "Empty", 14.0, theme::SECONDARY);
        }
        for card in &player.discard {
            card_row(world, parent, *card, battle, "Discarded unit");
        }
    }
}

fn inspect(world: &mut World, parent: Entity, battle: &Battle) {
    let Some(id) = battle.selected else {
        return;
    };
    let Some((card, unit)) = visible_card(battle, id) else {
        return;
    };
    let Some(definition) = battle
        .catalog
        .iter()
        .find(|definition| definition.id == card.definition)
    else {
        return;
    };
    layout::text(
        world,
        parent,
        definition.name.clone(),
        26.0,
        theme::FOREGROUND,
    );
    layout::text(
        world,
        parent,
        format!(
            "Cost {}\nPower {} | Maximum Health {}",
            definition.cost, definition.power, definition.health
        ),
        18.0,
        theme::FOREGROUND,
    );
    if let Some(unit) = unit {
        let health = u64::from(unit.health).saturating_sub(unit.damage);
        layout::text(
            world,
            parent,
            format!(
                "Remaining Health {health} | Damage {}\n{}",
                unit.damage,
                if unit.ready { "Ready" } else { "Exhausted" }
            ),
            17.0,
            theme::SECONDARY,
        );
        if battle
            .view
            .as_ref()
            .is_some_and(|view| unit.owner == view.player && view.phase == Phase::Action)
        {
            contextual_button(world, parent, Control::AttackUnit, battle);
        }
    } else if battle.view.as_ref().is_some_and(|view| {
        view.phase == Phase::Action && view.hand.iter().any(|held| held.id == id)
    }) {
        contextual_button(world, parent, Control::PlayUnit, battle);
    } else {
        layout::text(
            world,
            parent,
            "This card is in a resource or discard zone.",
            15.0,
            theme::SECONDARY,
        );
    }
}

fn replace(world: &mut World, parent: Entity, battle: &Battle) {
    if !battle.resume_available && !battle.notification.is_empty() {
        layout::text(
            world,
            parent,
            battle.notification.clone(),
            16.0,
            theme::ACCENT,
        );
    }
    layout::text(
        world,
        parent,
        "Starting a new battle replaces your unfinished saved match.",
        17.0,
        theme::FOREGROUND,
    );
    button(
        world,
        parent,
        Control::ConfirmReplace,
        "Start New Battle",
        true,
    );
    button(world, parent, Control::Cancel, "Keep Saved Battle", true);
}

fn error(world: &mut World, parent: Entity, battle: &Battle) {
    layout::text(
        world,
        parent,
        battle
            .error
            .clone()
            .unwrap_or_else(|| "A battle operation could not finish.".to_owned()),
        17.0,
        theme::FOREGROUND,
    );
    button(world, parent, Control::Retry, "Retry", true);
    button(world, parent, Control::MainMenu, "Main Menu", true);
}

fn card_row(world: &mut World, parent: Entity, card: Card, battle: &Battle, caption: &str) {
    let Some(definition) = battle
        .catalog
        .iter()
        .find(|definition| definition.id == card.definition)
    else {
        return;
    };
    let selected = battle.resource_selection.contains(&card.id);
    let value = format!(
        "{} | Cost {} | {} / {}\n{}",
        definition.name,
        definition.cost,
        definition.power,
        definition.health,
        if selected {
            "Selected resource"
        } else {
            caption
        }
    );
    let entity = button(world, parent, Control::Card(card.id), &value, true);
    if selected {
        world
            .entity_mut(entity)
            .insert(BorderColor::all(theme::ACCENT));
    }
}

fn contextual_button(world: &mut World, parent: Entity, control: Control, battle: &Battle) {
    button(
        world,
        parent,
        control,
        control.label(),
        enabled(control, battle),
    );
}

pub(super) fn enabled(control: Control, battle: &Battle) -> bool {
    if battle.error.is_some()
        && matches!(
            control,
            Control::KeepHand
                | Control::Mulligan
                | Control::ConfirmResources
                | Control::Pass
                | Control::ClaimInitiative
                | Control::PlayUnit
                | Control::AttackUnit
                | Control::EnemyStronghold
        )
    {
        return false;
    }
    match control {
        Control::KeepHand => battle.legal.contains(&Command::KeepHand),
        Control::Mulligan => battle.legal.contains(&Command::MulliganHand),
        Control::Pass => battle.legal.contains(&Command::Pass),
        Control::ClaimInitiative => battle.legal.contains(&Command::ClaimInitiative),
        Control::PlayUnit => battle
            .selected
            .is_some_and(|card| battle.legal.contains(&Command::PlayUnit { card })),
        Control::AttackUnit => battle.selected.is_some_and(|card| {
            battle.legal.iter().any(
                |command| matches!(command, Command::Attack { attacker, .. } if *attacker == card),
            )
        }),
        Control::EnemyStronghold => battle.attacker.is_some_and(|attacker| {
            battle.view.as_ref().is_some_and(|view| {
                battle.legal.contains(&Command::Attack {
                    attacker,
                    target: AttackTarget::Stronghold(view.player.opponent()),
                })
            })
        }),
        Control::ConfirmResources => battle.view.as_ref().is_some_and(|view| {
            let mut cards = battle.resource_selection.clone();
            cards.sort_unstable();
            match view.phase {
                Phase::InitialResourcing => battle
                    .legal
                    .contains(&Command::ChooseInitialResources { cards }),
                Phase::Regroup => battle.legal.contains(&Command::ChooseRegroupResource {
                    card: cards.first().copied(),
                }),
                _ => false,
            }
        }),
        _ => true,
    }
}

pub(super) fn set_enabled(world: &mut World, entity: Entity, enabled: bool) {
    if enabled {
        world
            .entity_mut(entity)
            .remove::<DisabledControl>()
            .insert(TabIndex(0));
    } else {
        world
            .entity_mut(entity)
            .insert((DisabledControl, TabIndex(-1)));
    }
}

pub(super) fn button(
    world: &mut World,
    parent: Entity,
    control: Control,
    label: &str,
    enabled: bool,
) -> Entity {
    let entity = world
        .spawn((
            Button,
            control,
            TabIndex(if enabled { 0 } else { -1 }),
            Name::new(format!("{} control", control.label())),
            Node {
                min_height: px(44),
                height: px(46),
                min_width: px(44),
                flex_shrink: 0.0,
                padding: UiRect::axes(px(8), px(3)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(4)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme::BUTTON),
            BorderColor::all(theme::BORDER),
        ))
        .id();
    if !enabled {
        world.entity_mut(entity).insert(DisabledControl);
    }
    world.entity_mut(parent).add_child(entity);
    let text = layout::text(world, entity, label.to_owned(), 14.0, theme::FOREGROUND);
    world
        .entity_mut(text)
        .insert(TextLayout::justify(Justify::Center));
    entity
}

pub(super) fn button_text(world: &World, button: Entity) -> Option<Entity> {
    world
        .get::<Children>(button)?
        .iter()
        .find(|entity| world.get::<Text>(*entity).is_some())
}
