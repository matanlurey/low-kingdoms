use std::collections::BTreeSet;

use bevy::{input_focus::tab_navigation::TabIndex, prelude::*};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    model::{Card, CardId},
    view::UnitView,
};

use crate::ui::{Control, theme};

use super::super::Battle;
use super::{CardFace, Layout, Presentation, Zone, friendly, layout, panels};

pub(super) struct Face {
    entity: Entity,
    text: Entity,
}

impl Face {
    pub(super) fn exists(&self, world: &World) -> bool {
        world.get::<CardFace>(self.entity).is_some() && world.get::<Text>(self.text).is_some()
    }
}

#[derive(Component)]
struct EmptyZone(Zone);

pub(super) fn reconcile(
    world: &mut World,
    presentation: &mut Presentation,
    battle: &Battle,
    layout: Layout,
) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    let mut visible = Vec::new();
    for card in &view.hand {
        visible.push((Zone::Hand, *card, None));
    }
    for player in &view.players {
        for unit in &player.units {
            let zone = if friendly(view, unit.owner) {
                Zone::Friendly
            } else {
                Zone::Enemy
            };
            visible.push((zone, unit.card, Some(unit)));
        }
    }
    let desired = visible
        .iter()
        .map(|(zone, card, _)| (*zone, card.id))
        .collect::<BTreeSet<_>>();
    presentation.cards.retain(|key, face| {
        if desired.contains(key) {
            true
        } else {
            world.despawn(face.entity);
            false
        }
    });
    for (zone, card, unit) in visible {
        let Some(parent) = presentation.zones.get(&zone).copied() else {
            continue;
        };
        let face = presentation
            .cards
            .entry((zone, card.id))
            .or_insert_with(|| spawn(world, parent, card.id, layout));
        update(world, face, battle, card, unit, layout);
    }
    reconcile_empty(world, presentation);
}

fn spawn(world: &mut World, parent: Entity, id: CardId, layout: Layout) -> Face {
    let entity = world
        .spawn((
            Button,
            Control::Card(id),
            CardFace,
            TabIndex(0),
            Name::new(format!("Visible card {}", id.0)),
            Node {
                width: px(if layout.compact { 128 } else { 136 }),
                height: px(if layout.landscape { 98 } else { 110 }),
                min_height: px(98),
                flex_shrink: 0.0,
                padding: UiRect::all(px(7)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::BUTTON),
            BorderColor::all(theme::BORDER),
        ))
        .id();
    world.entity_mut(parent).add_child(entity);
    let text = layout::text(world, entity, "Card", 14.0, theme::FOREGROUND);
    Face { entity, text }
}

fn update(
    world: &mut World,
    face: &Face,
    battle: &Battle,
    card: Card,
    unit: Option<&UnitView>,
    layout: Layout,
) {
    let Some(definition) = battle
        .catalog
        .iter()
        .find(|definition| definition.id == card.definition)
    else {
        return;
    };
    let selected = battle.selected == Some(card.id)
        || battle.resource_selection.contains(&card.id)
        || battle.attacker == Some(card.id);
    let target = battle.attacker.is_some_and(|attacker| {
        battle.legal.contains(&Command::Attack {
            attacker,
            target: AttackTarget::Unit(card.id),
        })
    });
    let value = unit.map_or_else(
        || {
            format!(
                "{}\nCost {} | {} / {}\n{}",
                definition.name,
                definition.cost,
                definition.power,
                definition.health,
                if battle.resource_selection.contains(&card.id) {
                    "Selected resource"
                } else {
                    "Tap to inspect"
                }
            )
        },
        |unit| {
            let health = u64::from(unit.health).saturating_sub(unit.damage);
            format!(
                "{} | Cost {}\n{} / {health}\nDamage: {}\n{}",
                definition.name,
                definition.cost,
                unit.power,
                unit.damage,
                if unit.ready { "Ready" } else { "Exhausted" }
            )
        },
    );
    if let Some(mut text) = world.get_mut::<Text>(face.text)
        && text.0 != value
    {
        text.0 = value;
    }
    let border = if selected || target {
        theme::ACCENT
    } else {
        theme::BORDER
    };
    if let Some(mut color) = world.get_mut::<BorderColor>(face.entity) {
        *color = BorderColor::all(border);
    }
    let color = if target {
        Color::srgb(0.25, 0.22, 0.14)
    } else if selected {
        theme::PRESSED
    } else if unit.is_some_and(|unit| !unit.ready) {
        Color::srgb(0.085, 0.10, 0.13)
    } else {
        theme::BUTTON
    };
    if let Some(mut background) = world.get_mut::<BackgroundColor>(face.entity) {
        *background = color.into();
    }
    if let Some(mut color) = world.get_mut::<TextColor>(face.text) {
        *color = if unit.is_some_and(|unit| !unit.ready) {
            theme::SECONDARY
        } else {
            theme::FOREGROUND
        }
        .into();
    }
    panels::set_enabled(
        world,
        face.entity,
        battle.error.is_none() && (battle.attacker.is_none() || target),
    );
    if let Some(mut font) = world.get_mut::<TextFont>(face.text) {
        font.font_size = FontSize::Px(if layout.landscape { 13.0 } else { 14.0 });
    }
}

fn reconcile_empty(world: &mut World, presentation: &Presentation) {
    let empty = world
        .query::<(Entity, &EmptyZone)>()
        .iter(world)
        .map(|(entity, zone)| (entity, zone.0))
        .collect::<Vec<_>>();
    for (zone, pane) in &presentation.zones {
        let has_cards = presentation
            .cards
            .keys()
            .any(|(card_zone, _)| card_zone == zone);
        let placeholder = empty
            .iter()
            .find(|(_, empty_zone)| empty_zone == zone)
            .map(|(entity, _)| *entity);
        if has_cards {
            if let Some(entity) = placeholder {
                world.despawn(entity);
            }
        } else if placeholder.is_none() {
            let entity = layout::text(
                world,
                *pane,
                if *zone == Zone::Hand {
                    "Your hand is empty"
                } else {
                    "No units deployed"
                },
                14.0,
                theme::SECONDARY,
            );
            world.entity_mut(entity).insert(EmptyZone(*zone));
        }
    }
}
