//! Reconciles the human projection into stable native Bevy UI entities.

use std::collections::BTreeMap;

use bevy::{
    input_focus::{InputFocus, InputFocusVisible, tab_navigation::TabGroup},
    prelude::*,
};
use low_kingdoms_rules::{
    model::{CardId, Phase, PlayerId},
    view::PlayerView,
};

use crate::{
    screens::{Screen, ScreenRoot},
    ui::{Control, theme},
};

use super::Battle;

mod cards;
mod layout;
mod panels;

#[cfg(test)]
mod tests;

/// A scrollable viewport; input changes its logical-pixel `ScrollPosition`.
#[derive(Component, Clone, Copy)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Sibling input and diagnostics modules share this crate-local viewport marker."
)]
pub(crate) struct ScrollPane {
    pub(crate) horizontal: bool,
}

/// Card styling is reconciled from public state rather than generic button colors.
#[derive(Component)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The sibling UI module excludes reconciled cards from generic button styling."
)]
pub(crate) struct CardFace;

/// Controls retain their explanation while unavailable and are excluded from activation.
#[derive(Component)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Sibling input and diagnostics modules inspect this crate-local activation marker."
)]
pub(crate) struct DisabledControl;

/// A modal surface; scrolling and empty-space gestures stay inside this subtree.
#[derive(Component)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The sibling input module restricts gestures to the currently open overlay."
)]
pub(crate) struct OverlayRoot {
    pub(crate) modal: bool,
}

#[derive(SystemSet, Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct Reconcile;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Zone {
    Enemy,
    Friendly,
    Hand,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Label {
    Status,
    Notification,
    EnemyInfo,
    OwnInfo,
    ResourcePrompt,
    Result,
    Round,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Layout {
    compact: bool,
    landscape: bool,
    resourcing: bool,
}

impl Layout {
    fn for_window(size: Vec2, view: Option<&PlayerView>) -> Self {
        Self {
            compact: size.x < 900.0 || size.y < 600.0,
            landscape: size.y < 480.0 && size.x > size.y,
            resourcing: view.is_some_and(|view| {
                matches!(view.phase, Phase::InitialResourcing | Phase::Regroup)
            }),
        }
    }
}

#[derive(Resource, Default)]
struct Presentation {
    root: Option<Entity>,
    screen: Option<Screen>,
    layout: Option<Layout>,
    size: Vec2,
    zones: BTreeMap<Zone, Entity>,
    cards: BTreeMap<(Zone, CardId), cards::Face>,
    labels: BTreeMap<Label, Entity>,
    controls: Vec<(Control, Entity)>,
    log: Option<Entity>,
    overlay: Option<Entity>,
    overlay_content: Option<Entity>,
    overlay_panel: Option<super::Panel>,
    overlay_stamp: String,
    log_lines: Vec<String>,
}

pub(super) fn configure(app: &mut App) {
    app.init_resource::<Presentation>()
        .init_resource::<InputFocus>()
        .init_resource::<InputFocusVisible>()
        .add_systems(OnEnter(Screen::Mulligan), spawn)
        .add_systems(OnEnter(Screen::Battle), spawn)
        .add_systems(OnEnter(Screen::Results), spawn)
        .add_systems(
            Update,
            (synchronize, style_card_focus).chain().in_set(Reconcile),
        );
}

fn spawn(world: &mut World) {
    world.resource_mut::<InputFocus>().clear();
    let screen = *world.resource::<State<Screen>>().get();
    let root = spawn_root(world, screen);
    let mut presentation = world.resource_mut::<Presentation>();
    *presentation = Presentation {
        root: Some(root),
        screen: Some(screen),
        ..default()
    };
}

fn spawn_root(world: &mut World, screen: Screen) -> Entity {
    world
        .spawn((
            ScreenRoot,
            DespawnOnExit(screen),
            Name::new(format!("{screen:?} screen root")),
            TabGroup::new(0),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(8)),
                row_gap: px(5),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .id()
}

fn synchronize(world: &mut World) {
    let screen = *world.resource::<State<Screen>>().get();
    let size = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map_or(Vec2::new(1280.0, 720.0), |window| {
            Vec2::new(window.width(), window.height())
        });
    let changed = world
        .get_resource_ref::<Battle>()
        .is_some_and(|battle| battle.is_changed());
    let mut presentation = world.remove_resource::<Presentation>().unwrap_or_default();
    recover_missing_entities(world, &mut presentation, screen);
    let layout = Layout::for_window(size, world.resource::<Battle>().view.as_ref());
    let needs_layout = presentation.layout != Some(layout) || presentation.screen != Some(screen);
    if changed || needs_layout || presentation.size != size {
        synchronize_changed(world, &mut presentation, screen, size, layout);
    }
    world.insert_resource(presentation);
}

fn recover_missing_entities(world: &mut World, presentation: &mut Presentation, screen: Screen) {
    if matches!(screen, Screen::Title | Screen::About) {
        return;
    }
    if presentation
        .root
        .is_none_or(|root| world.get_entity(root).is_err())
    {
        *presentation = Presentation {
            root: Some(spawn_root(world, screen)),
            screen: Some(screen),
            ..default()
        };
        warn!("Battle presentation root was missing and has been reconciled.");
    } else if presentation.cards.values().any(|face| !face.exists(world))
        || presentation
            .labels
            .values()
            .any(|entity| world.get::<Text>(*entity).is_none())
        || presentation
            .zones
            .values()
            .any(|entity| world.get::<ScrollPosition>(*entity).is_none())
    {
        presentation.layout = None;
        warn!("Battle presentation entities were missing and have been reconciled.");
    }
}

fn synchronize_changed(
    world: &mut World,
    presentation: &mut Presentation,
    screen: Screen,
    size: Vec2,
    layout: Layout,
) {
    world.resource_scope(|world, battle: Mut<Battle>| {
        if matches!(screen, Screen::Title | Screen::About) {
            if screen == Screen::Title {
                crate::screens::reconcile_title_resume(world, battle.resume_available);
            }
            let root = world
                .query_filtered::<Entity, With<ScreenRoot>>()
                .iter(world)
                .next();
            if presentation.root != root || presentation.screen != Some(screen) {
                presentation.overlay = None;
                presentation.overlay_stamp.clear();
            }
            presentation.root = root;
            presentation.screen = Some(screen);
            presentation.layout = Some(layout);
            presentation.size = size;
            panels::reconcile(world, presentation, &battle, layout);
            return;
        }
        if presentation.layout != Some(layout) {
            layout::build(world, presentation, screen, layout);
        }
        presentation.size = size;
        reconcile_labels(world, presentation, &battle);
        cards::reconcile(world, presentation, &battle, layout);
        reconcile_controls(world, presentation, &battle);
        reconcile_log(world, presentation, &battle);
        panels::reconcile(world, presentation, &battle, layout);
    });
}

fn reconcile_labels(world: &mut World, presentation: &Presentation, battle: &Battle) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    layout::highlight(world, view);
    let status = if battle.attacker.is_some() {
        "Choose an attack target".to_owned()
    } else {
        match view.phase {
            Phase::Mulligan if !view.decision_submitted => "Choose your opening hand".to_owned(),
            Phase::InitialResourcing if !view.decision_submitted => {
                "Choose starting resources".to_owned()
            }
            Phase::Regroup if !view.decision_submitted => "Choose resources".to_owned(),
            Phase::Finished => "Game over".to_owned(),
            Phase::Action if view.active_player == Some(view.player) => "Your action".to_owned(),
            _ => "Opponent acting".to_owned(),
        }
    };
    let initiative = if view.initiative == view.player {
        "You"
    } else {
        "Opponent"
    };
    set_label(
        world,
        presentation,
        Label::Status,
        format!(
            "{status}\nRound {} | Initiative: {initiative} | AI {:?}",
            view.round, battle.speed
        ),
    );
    set_label(
        world,
        presentation,
        Label::Notification,
        if battle.notification.is_empty() {
            "Select a card to inspect. Swipe or scroll crowded rows.".to_owned()
        } else {
            battle.notification.clone()
        },
    );
    set_label(
        world,
        presentation,
        Label::EnemyInfo,
        player_info(view, false),
    );
    set_label(world, presentation, Label::OwnInfo, player_info(view, true));
    let limit = if view.phase == Phase::InitialResourcing {
        2
    } else {
        1
    };
    set_label(
        world,
        presentation,
        Label::ResourcePrompt,
        if view.decision_submitted {
            "Resource choice submitted. Waiting for opponent.".to_owned()
        } else {
            format!(
                "Choose up to {limit} cards: {} selected",
                battle.resource_selection.len()
            )
        },
    );
    set_label(
        world,
        presentation,
        Label::Round,
        format!("Battle ended in round {}", view.round),
    );
    set_label(
        world,
        presentation,
        Label::Result,
        match view.outcome {
            Some(low_kingdoms_rules::model::Outcome::Winner(player)) if player == view.player => {
                "Victory"
            }
            Some(low_kingdoms_rules::model::Outcome::Winner(_)) => "Defeat",
            Some(low_kingdoms_rules::model::Outcome::Draw) => "Draw",
            None => "Battle ended",
        }
        .to_owned(),
    );
}

fn player_info(view: &PlayerView, own: bool) -> String {
    let index = usize::from(!own);
    let player = &view.players[index];
    let health = u64::from(player.stronghold_health).saturating_sub(player.stronghold_damage);
    let name = if own {
        "Your stronghold"
    } else {
        "Opponent stronghold"
    };
    format!(
        "{name}: {health} HP\nResources: {}/{} ready | Deck: {} | Hand: {}",
        player.resources.ready, player.resources.total, player.deck_size, player.hand_size
    )
}

fn set_label(world: &mut World, presentation: &Presentation, label: Label, value: String) {
    if let Some(entity) = presentation.labels.get(&label)
        && let Some(mut text) = world.get_mut::<Text>(*entity)
        && text.0 != value
    {
        text.0 = value;
    }
}

fn reconcile_controls(world: &mut World, presentation: &Presentation, battle: &Battle) {
    for (control, entity) in &presentation.controls {
        let enabled = panels::enabled(*control, battle);
        panels::set_enabled(world, *entity, enabled);
    }
}

fn reconcile_log(world: &mut World, presentation: &mut Presentation, battle: &Battle) {
    let Some(log) = presentation.log else {
        return;
    };
    if presentation.log_lines == battle.history {
        return;
    }
    let prefix = presentation
        .log_lines
        .iter()
        .zip(&battle.history)
        .take_while(|(a, b)| a == b)
        .count();
    if prefix != presentation.log_lines.len() {
        world.entity_mut(log).despawn_children();
        presentation.log_lines.clear();
    }
    for line in &battle.history[presentation.log_lines.len()..] {
        layout::text(world, log, line.clone(), 13.0, theme::SECONDARY);
    }
    presentation.log_lines.clone_from(&battle.history);
}

fn visible_card(
    battle: &Battle,
    id: CardId,
) -> Option<(
    low_kingdoms_rules::model::Card,
    Option<&low_kingdoms_rules::view::UnitView>,
)> {
    let view = battle.view.as_ref()?;
    if let Some(unit) = view
        .players
        .iter()
        .flat_map(|player| &player.units)
        .find(|unit| unit.card.id == id)
    {
        return Some((unit.card, Some(unit)));
    }
    view.hand
        .iter()
        .chain(view.resources.iter().map(|resource| &resource.card))
        .chain(view.players.iter().flat_map(|player| &player.discard))
        .find(|card| card.id == id)
        .copied()
        .map(|card| (card, None))
}

fn friendly(view: &PlayerView, player: PlayerId) -> bool {
    view.player == player
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects owned resource system parameters."
)]
fn style_card_focus(
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    battle: Res<Battle>,
    mut cards: Query<(Entity, &Control, &mut BorderColor), With<CardFace>>,
) {
    if !focus.is_changed() && !visible.is_changed() && !battle.is_changed() {
        return;
    }
    for (entity, control, mut border) in &mut cards {
        let emphasized = match control {
            Control::Card(id) => {
                battle.selected == Some(*id)
                    || battle.resource_selection.contains(id)
                    || battle.attacker == Some(*id)
                    || battle.attacker.is_some_and(|attacker| {
                        battle
                            .legal
                            .contains(&low_kingdoms_rules::command::Command::Attack {
                                attacker,
                                target: low_kingdoms_rules::command::AttackTarget::Unit(*id),
                            })
                    })
            }
            Control::EnemyStronghold => panels::enabled(*control, &battle),
            _ => false,
        };
        *border = BorderColor::all(
            if emphasized || (focus.get() == Some(entity) && visible.0) {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        );
    }
}
