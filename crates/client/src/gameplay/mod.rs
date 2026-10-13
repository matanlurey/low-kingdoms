//! Session ownership and command dispatch. Presentation receives the human projection only.

use bevy::prelude::*;
use low_kingdoms_battle::{
    content::{CardDefinition, Content},
    session::Session,
};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    event::Event,
    model::{Card, CardId, DefinitionId, Outcome, Phase, PlayerId},
    view::PlayerView,
};
use serde::{Deserialize, Serialize};

use crate::{
    platform::storage::{self, Store},
    screens::Screen,
    ui::Control,
};

#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) mod presentation;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) enum Speed {
    #[default]
    Normal,
    Fast,
    Instant,
}

impl Speed {
    const fn seconds(self) -> f32 {
        match self {
            Self::Normal => 0.75,
            Self::Fast => 0.15,
            Self::Instant => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) enum Panel {
    #[default]
    None,
    More,
    Hand,
    Log,
    Resources,
    Inspect,
    Replace,
    Error,
}

#[derive(Resource, Default)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) struct Battle {
    pub view: Option<PlayerView>,
    pub legal: Vec<Command>,
    pub catalog: Vec<CardDefinition>,
    pub history: Vec<String>,
    pub revision: u64,
    pub resume_available: bool,
    pub error: Option<String>,
    pub notification: String,
    pub panel: Panel,
    pub selected: Option<CardId>,
    pub resource_selection: Vec<CardId>,
    pub attacker: Option<CardId>,
    pub speed: Speed,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalSave {
    version: u32,
    speed: Speed,
    session: String,
}

#[derive(Default)]
struct AiClock {
    wait: f32,
    was_active: bool,
}

#[derive(Resource, Default)]
struct Runtime {
    content: Option<Content>,
    session: Option<Session>,
    store: Option<Box<dyn Store>>,
    loaded: bool,
    dirty: bool,
    suspended: bool,
    ai_clock: AiClock,
    last_seed: Option<u64>,
}

#[derive(Message, Clone, Copy)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) struct Activate(pub Control);

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
#[derive(Resource, Default)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) struct CapturePauseAI(pub bool);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) struct Dispatch;

#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Battle>()
            .init_resource::<Runtime>()
            .add_message::<Activate>()
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                (dispatch, advance_ai)
                    .chain()
                    .in_set(Dispatch)
                    .before(presentation::Reconcile),
            );
        presentation::configure(app);
    }
}

fn initialize(mut runtime: ResMut<Runtime>, mut battle: ResMut<Battle>) {
    if !load_content(&mut runtime, &mut battle) {
        return;
    }
    if runtime.store.is_none() {
        match storage::default_store() {
            Ok(store) => runtime.store = Some(store),
            Err(error) => {
                fail(&mut battle, error.to_string());
                return;
            }
        }
    }
    load(&mut runtime, &mut battle);
}

fn load_content(runtime: &mut Runtime, battle: &mut Battle) -> bool {
    match Content::load() {
        Ok(content) => {
            battle.catalog = content.cards().to_vec();
            runtime.content = Some(content);
            true
        }
        Err(error) => {
            fail(battle, error.to_string());
            false
        }
    }
}

fn fail(battle: &mut Battle, message: String) {
    battle.error = Some(message);
    battle.panel = Panel::Error;
    // The diagnostic is deliberately concise and never includes snapshots or private commands.
    warn!("Battle operation paused; a recoverable message is displayed.");
}

fn load(runtime: &mut Runtime, battle: &mut Battle) {
    let Some(store) = runtime.store.as_mut() else {
        fail(
            battle,
            "Local storage is unavailable. Retry loading before continuing.".to_owned(),
        );
        return;
    };
    let data = match store.read() {
        Ok(data) => data,
        Err(error) => {
            fail(battle, error.to_string());
            return;
        }
    };
    runtime.loaded = true;
    let Some(data) = data else {
        return;
    };
    let Some(content) = runtime.content.as_ref() else {
        return;
    };
    let saved = serde_json::from_str::<LocalSave>(&data)
        .map_err(|_| "The local battle save is damaged.".to_owned())
        .and_then(|save| {
            if save.version != 1 {
                return Err("The local battle save version is incompatible.".to_owned());
            }
            Session::restore(content, &save.session)
                .map(|session| (session, save.speed))
                .map_err(|error| error.to_string())
        });
    match saved {
        Ok((session, speed)) => {
            runtime.last_seed = Some(session.seed());
            if session.is_finished() {
                if let Err(error) = store.clear() {
                    runtime.session = Some(session);
                    runtime.dirty = true;
                    fail(battle, error.to_string());
                }
            } else {
                runtime.session = Some(session);
                battle.speed = speed;
                battle.resume_available = true;
            }
        }
        Err(message) => {
            // A corrupt slot is left intact until the user explicitly confirms replacement.
            battle.resume_available = false;
            battle.notification = message;
            runtime.suspended = true;
        }
    }
}

fn persist(runtime: &mut Runtime, battle: &mut Battle) {
    let Some(session) = runtime.session.as_ref() else {
        return;
    };
    let finished = session.is_finished();
    let data = session
        .save()
        .map_err(|error| error.to_string())
        .and_then(|session| {
            serde_json::to_string(&LocalSave {
                version: 1,
                speed: battle.speed,
                session,
            })
            .map_err(|_| "Could not encode the battle save.".to_owned())
        });
    let Some(store) = runtime.store.as_mut() else {
        runtime.dirty = true;
        fail(
            battle,
            "Local storage is unavailable. Retry before continuing.".to_owned(),
        );
        return;
    };
    let operation = if finished {
        match store.clear() {
            Ok(()) => Ok(()),
            Err(error) => {
                // Replace the old unfinished slot with a terminal marker where possible. Retry
                // repeats removal only; it must never replay the successful ending command.
                let mut message = error.to_string();
                if let Err(fallback) =
                    data.and_then(|data| store.write(&data).map_err(|error| error.to_string()))
                {
                    message.push_str(" The completed-result fallback also failed: ");
                    message.push_str(&fallback);
                }
                Err(message)
            }
        }
    } else {
        data.and_then(|data| store.write(&data).map_err(|error| error.to_string()))
    };
    match operation {
        Ok(()) => {
            runtime.dirty = false;
            battle.error = None;
            if battle.panel == Panel::Error {
                battle.panel = Panel::None;
            }
        }
        Err(message) => {
            runtime.dirty = true;
            fail(battle, message);
        }
    }
}

fn fresh(runtime: &mut Runtime, battle: &mut Battle) -> bool {
    let Some(content) = runtime.content.as_ref() else {
        return false;
    };
    let mut next_seed = None;
    for _ in 0..4 {
        match fresh_seed(runtime.last_seed) {
            Ok(seed) if Some(seed) != runtime.last_seed => {
                next_seed = Some(seed);
                break;
            }
            Ok(_) => {}
            Err(error) => {
                fail(battle, error.to_string());
                return false;
            }
        }
    }
    let Some(seed) = next_seed else {
        fail(
            battle,
            "Could not select a fresh match seed. Retry starting the battle.".to_owned(),
        );
        return false;
    };
    match Session::new(content, seed) {
        Ok(session) => {
            runtime.session = Some(session);
            runtime.last_seed = Some(seed);
            runtime.dirty = true;
            runtime.suspended = false;
            runtime.ai_clock.wait = 0.0;
            runtime.ai_clock.was_active = false;
            battle.error = None;
            battle.notification.clear();
            battle.selected = None;
            battle.attacker = None;
            battle.resource_selection.clear();
            battle.panel = Panel::None;
            battle.resume_available = true;
            project(runtime, battle);
            persist(runtime, battle);
            true
        }
        Err(error) => {
            fail(battle, error.to_string());
            false
        }
    }
}

fn fresh_seed(previous: Option<u64>) -> Result<u64, storage::StorageError> {
    #[cfg(feature = "test-support")]
    if previous.is_none() {
        return Ok(42);
    }
    let _ = previous;
    storage::fresh_seed()
}

fn project(runtime: &Runtime, battle: &mut Battle) {
    if let Some(session) = runtime.session.as_ref() {
        battle.view = Some(session.view());
        battle.legal = session.legal_commands();
        battle.revision = session.revision();
        battle.history = session
            .history()
            .iter()
            .filter_map(|event| describe(event, battle))
            .collect();
        battle.resume_available = !session.is_finished();
    }
}

fn describe(event: &Event, battle: &Battle) -> Option<String> {
    Some(match event {
        Event::RoundStarted { round, initiative } => {
            format!(
                "Round {round} began. {} hold initiative.",
                actor(*initiative)
            )
        }
        Event::RoundEnded { round } => format!("Round {round} ended."),
        Event::CardsDrawn { player, count, .. } => {
            format!("{} drew {count} cards.", actor(*player))
        }
        Event::CardsResourced { player, count, .. } => {
            format!("{} resourced {count} cards.", actor(*player))
        }
        Event::ResourcesExhausted { player, count, .. } => {
            format!("{} paid {count} resources.", actor(*player))
        }
        Event::ResourcesReadied { player, count } => {
            format!("{} readied {count} resources.", actor(*player))
        }
        Event::UnitsReadied { player, cards } => {
            format!("{} readied {} units.", actor(*player), cards.len())
        }
        Event::MulliganCompleted { player, replaced } => format!(
            "{} {} the opening hand.",
            actor(*player),
            if *replaced { "replaced" } else { "kept" }
        ),
        Event::UnitPlayed { unit } => {
            format!(
                "{} deployed {} exhausted.",
                actor(unit.owner),
                definition_name(battle, unit.card.definition)
            )
        }
        Event::AttackDeclared {
            player,
            attacker,
            target,
        } => format!(
            "{} {} attacked {}.",
            possessive(*player),
            visible_name(battle, *attacker),
            target_name(battle, *target),
        ),
        Event::CombatDamage { assignments } => {
            assignments
                .iter()
                .map(|assignment| {
                    let health =
                        target_health(battle, assignment.target).saturating_sub(assignment.total);
                    format!(
                        "{} took {} damage ({health} Health remaining)",
                        target_name(battle, assignment.target),
                        assignment.amount
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
                + "."
        }
        Event::UnitDestroyed { player, card } => {
            format!(
                "{} {} was destroyed.",
                possessive(*player),
                definition_name(battle, card.definition)
            )
        }
        Event::PlayerPassed { player } => format!("{} passed.", actor(*player)),
        Event::InitiativeClaimed { player } => format!("{} claimed initiative.", actor(*player)),
        Event::DeckExhaustionDamage {
            player,
            amount,
            total,
            ..
        } => {
            let health =
                target_health(battle, AttackTarget::Stronghold(*player)).saturating_sub(*total);
            format!(
                "{} stronghold took {amount} deck exhaustion damage ({health} Health remaining).",
                possessive(*player)
            )
        }
        Event::GameEnded { outcome } => match outcome {
            Outcome::Winner(PlayerId::Zero) => {
                "Victory. The opponent's stronghold was destroyed.".to_owned()
            }
            Outcome::Winner(PlayerId::One) => "Defeat. Your stronghold was destroyed.".to_owned(),
            Outcome::Draw => "Draw. Both strongholds were destroyed.".to_owned(),
        },
        _ => return None,
    })
}

const fn actor(player: PlayerId) -> &'static str {
    match player {
        PlayerId::Zero => "You",
        PlayerId::One => "Opponent",
    }
}

const fn possessive(player: PlayerId) -> &'static str {
    match player {
        PlayerId::Zero => "Your",
        PlayerId::One => "Opponent's",
    }
}

fn definition_name(battle: &Battle, id: DefinitionId) -> &str {
    battle
        .catalog
        .iter()
        .find(|card| card.id == id)
        .map_or("unit", |card| card.name.as_str())
}

fn visible_card(battle: &Battle, id: CardId) -> Option<(Card, PlayerId)> {
    let view = battle.view.as_ref()?;
    for (index, player) in view.players.iter().enumerate() {
        let owner = if index == 0 {
            PlayerId::Zero
        } else {
            PlayerId::One
        };
        if let Some(card) = player
            .units
            .iter()
            .map(|unit| unit.card)
            .chain(player.discard.iter().copied())
            .find(|card| card.id == id)
        {
            return Some((card, owner));
        }
    }
    view.hand
        .iter()
        .copied()
        .chain(view.resources.iter().map(|resource| resource.card))
        .find(|card| card.id == id)
        .map(|card| (card, view.player))
}

fn visible_name(battle: &Battle, id: CardId) -> &str {
    visible_card(battle, id).map_or("unit", |(card, _)| definition_name(battle, card.definition))
}

fn target_name(battle: &Battle, target: AttackTarget) -> String {
    match target {
        AttackTarget::Stronghold(player) => format!("{} stronghold", possessive(player)),
        AttackTarget::Unit(id) => visible_card(battle, id).map_or_else(
            || "Unit".to_owned(),
            |(card, owner)| {
                format!(
                    "{} {}",
                    possessive(owner),
                    definition_name(battle, card.definition)
                )
            },
        ),
    }
}

fn target_health(battle: &Battle, target: AttackTarget) -> u64 {
    match target {
        AttackTarget::Stronghold(player) => battle.view.as_ref().map_or(0, |view| {
            u64::from(match player {
                PlayerId::Zero => view.players[0].stronghold_health,
                PlayerId::One => view.players[1].stronghold_health,
            })
        }),
        AttackTarget::Unit(id) => visible_card(battle, id)
            .and_then(|(card, _)| {
                battle
                    .catalog
                    .iter()
                    .find(|definition| definition.id == card.definition)
            })
            .map_or(0, |definition| u64::from(definition.health)),
    }
}

fn screen_for(battle: &Battle) -> Screen {
    match battle.view.as_ref().map(|view| view.phase) {
        Some(Phase::Mulligan) => Screen::Mulligan,
        Some(Phase::Finished) => Screen::Results,
        _ => Screen::Battle,
    }
}

fn accept(runtime: &mut Runtime, battle: &mut Battle, command: &Command) {
    if battle.error.is_some() || !battle.legal.contains(command) {
        return;
    }
    let Some(session) = runtime.session.as_mut() else {
        return;
    };
    match session.apply_human(command) {
        Ok(events) => succeeded(runtime, battle, &events, true),
        Err(error) => fail(battle, error.to_string()),
    }
}

fn succeeded(runtime: &mut Runtime, battle: &mut Battle, events: &[Event], human: bool) {
    let previous_phase = battle.view.as_ref().map(|view| view.phase);
    runtime.ai_clock.wait = 0.0;
    runtime.ai_clock.was_active = false;
    runtime.dirty = true;
    project(runtime, battle);
    battle.notification = events
        .iter()
        .rev()
        .find_map(|event| describe(event, battle))
        .unwrap_or_else(|| "Decision submitted; waiting for opponent.".to_owned());
    if human || battle.view.as_ref().map(|view| view.phase) != previous_phase {
        battle.selected = None;
        battle.attacker = None;
        battle.resource_selection.clear();
        battle.panel = Panel::None;
    } else {
        preserve_choices(battle);
    }
    persist(runtime, battle);
}

fn preserve_choices(battle: &mut Battle) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    let visible = |id| {
        view.hand.iter().any(|card| card.id == id)
            || view.resources.iter().any(|resource| resource.card.id == id)
            || view.players.iter().any(|player| {
                player.units.iter().any(|unit| unit.card.id == id)
                    || player.discard.iter().any(|card| card.id == id)
            })
    };
    if battle.selected.is_some_and(|id| !visible(id)) {
        battle.selected = None;
        if battle.panel == Panel::Inspect {
            battle.panel = Panel::None;
        }
    }
    battle
        .resource_selection
        .retain(|id| !view.decision_submitted && view.hand.iter().any(|card| card.id == *id));
    if battle.attacker.is_some_and(|attacker| {
        !battle.legal.iter().any(
            |command| matches!(command, Command::Attack { attacker: id, .. } if *id == attacker),
        )
    }) {
        battle.attacker = None;
    }
}

fn dispatch(
    mut messages: MessageReader<Activate>,
    mut runtime: ResMut<Runtime>,
    mut battle: ResMut<Battle>,
    mut next: ResMut<NextState<Screen>>,
) {
    for Activate(control) in messages.read() {
        match *control {
            Control::Play | Control::PlayAgain => {
                if !runtime.loaded || runtime.content.is_none() { fail(&mut battle, "Retry loading local saves before starting a battle.".to_owned()); }
                else if battle.resume_available || runtime.suspended { battle.panel = Panel::Replace; }
                else if fresh(&mut runtime, &mut battle) { NextState::set_if_neq(&mut next, Screen::Mulligan); }
            }
            Control::ConfirmReplace => { if fresh(&mut runtime, &mut battle) { NextState::set_if_neq(&mut next, Screen::Mulligan); } }
            Control::Resume => { if runtime.session.as_ref().is_some_and(|session| !session.is_finished()) { project(&runtime, &mut battle); battle.panel = if runtime.dirty || battle.error.is_some() { Panel::Error } else { Panel::None }; runtime.ai_clock.wait = 0.0; runtime.ai_clock.was_active = false; NextState::set_if_neq(&mut next, screen_for(&battle)); } }
            Control::MainMenu => { battle.panel = Panel::None; battle.selected = None; battle.attacker = None; NextState::set_if_neq(&mut next, Screen::Title); }
            Control::Cancel | Control::Close => { battle.panel = Panel::None; battle.selected = None; battle.attacker = None; battle.resource_selection.clear(); }
            Control::Retry => {
                if runtime.dirty { persist(&mut runtime, &mut battle); }
                else if !runtime.loaded || runtime.content.is_none() {
                    if runtime.content.is_none() && !load_content(&mut runtime, &mut battle) { continue; }
                    if runtime.store.is_none() {
                        match storage::default_store() {
                            Ok(store) => runtime.store = Some(store),
                            Err(error) => { fail(&mut battle, error.to_string()); continue; }
                        }
                    }
                    battle.error = None;
                    battle.panel = Panel::None;
                    load(&mut runtime, &mut battle);
                }
                else { battle.error = None; battle.panel = Panel::None; runtime.ai_clock.wait = 0.0; }
            }
            Control::More => {
                battle.panel = if battle.error.is_some() {
                    Panel::Error
                } else {
                    Panel::More
                };
            }
            Control::Hand => battle.panel = Panel::Hand,
            Control::Log => battle.panel = Panel::Log,
            Control::Resources => battle.panel = Panel::Resources,
            Control::SpeedNormal | Control::SpeedFast | Control::SpeedInstant => {
                battle.speed = match control { Control::SpeedFast => Speed::Fast, Control::SpeedInstant => Speed::Instant, _ => Speed::Normal }; runtime.ai_clock.wait = 0.0; battle.panel = Panel::None;
                if runtime.session.is_some() { persist(&mut runtime, &mut battle); }
            }
            Control::KeepHand => accept(&mut runtime, &mut battle, &Command::KeepHand),
            Control::Mulligan => accept(&mut runtime, &mut battle, &Command::MulliganHand),
            Control::ConfirmResources => {
                battle.resource_selection.sort_unstable();
                let command = if battle.view.as_ref().is_some_and(|view| view.phase == Phase::InitialResourcing) { Command::ChooseInitialResources { cards: battle.resource_selection.clone() } } else { Command::ChooseRegroupResource { card: battle.resource_selection.first().copied() } };
                accept(&mut runtime, &mut battle, &command);
            }
            Control::Pass => accept(&mut runtime, &mut battle, &Command::Pass),
            Control::ClaimInitiative => accept(&mut runtime, &mut battle, &Command::ClaimInitiative),
            Control::PlayUnit => { if let Some(card) = battle.selected { accept(&mut runtime, &mut battle, &Command::PlayUnit { card }); } }
            Control::AttackUnit => {
                if let Some(attacker) = battle.selected && battle.legal.iter().any(|command| matches!(command, Command::Attack { attacker: id, .. } if *id == attacker)) { battle.attacker = Some(attacker); battle.panel = Panel::None; "Choose a highlighted enemy unit or stronghold. Scroll to find targets.".clone_into(&mut battle.notification); }
            }
            Control::EnemyStronghold => { if let Some(attacker) = battle.attacker { accept(&mut runtime, &mut battle, &Command::Attack { attacker, target: AttackTarget::Stronghold(PlayerId::One) }); } }
            Control::Card(card) => card_clicked(card, &mut runtime, &mut battle),
            Control::About | Control::Back | Control::Quit => {}
        }
        if matches!(
            control,
            Control::KeepHand
                | Control::Mulligan
                | Control::ConfirmResources
                | Control::Pass
                | Control::ClaimInitiative
                | Control::PlayUnit
                | Control::EnemyStronghold
                | Control::Card(_)
        ) && battle.view.is_some()
        {
            NextState::set_if_neq(&mut next, screen_for(&battle));
        }
    }
}

fn card_clicked(card: CardId, runtime: &mut Runtime, battle: &mut Battle) {
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    if let Some(attacker) = battle.attacker
        && view.players[1]
            .units
            .iter()
            .any(|unit| unit.card.id == card)
    {
        accept(
            runtime,
            battle,
            &Command::Attack {
                attacker,
                target: AttackTarget::Unit(card),
            },
        );
        return;
    }
    if matches!(view.phase, Phase::InitialResourcing | Phase::Regroup)
        && !view.decision_submitted
        && view.hand.iter().any(|held| held.id == card)
    {
        if battle.resource_selection.contains(&card) {
            battle.resource_selection.retain(|id| *id != card);
        } else if battle.resource_selection.len()
            < if view.phase == Phase::InitialResourcing {
                2
            } else {
                1
            }
        {
            battle.resource_selection.push(card);
        }
    } else {
        battle.selected = Some(card);
        battle.panel = Panel::Inspect;
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy owns injected read-only resources and queries."
)]
fn advance_ai(
    #[cfg(all(feature = "test-support", not(target_arch = "wasm32")))] capture_pause: Option<
        Res<CapturePauseAI>,
    >,
    time: Res<Time>,
    state: Res<State<Screen>>,
    windows: Query<&Window>,
    mut runtime: ResMut<Runtime>,
    mut battle: ResMut<Battle>,
    mut next: ResMut<NextState<Screen>>,
) {
    #[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
    if capture_pause.is_some_and(|pause| pause.0) {
        return;
    }
    if !matches!(state.get(), Screen::Mulligan | Screen::Battle) || battle.error.is_some() {
        runtime.ai_clock.wait = 0.0;
        return;
    }
    let active = storage::document_active() && windows.iter().all(|window| window.focused);
    if !active {
        runtime.ai_clock.was_active = false;
        runtime.ai_clock.wait = 0.0;
        return;
    }
    if !runtime.ai_clock.was_active {
        runtime.ai_clock.was_active = true;
        runtime.ai_clock.wait = 0.0;
        return;
    }
    if !runtime.session.as_ref().is_some_and(Session::needs_ai) {
        runtime.ai_clock.wait = 0.0;
        return;
    }
    runtime.ai_clock.wait += time.delta_secs();
    if runtime.ai_clock.wait < battle.speed.seconds() {
        return;
    }
    let Runtime {
        session, content, ..
    } = &mut *runtime;
    if let (Some(session), Some(content)) = (session.as_mut(), content.as_ref()) {
        match session.step_ai(content) {
            Ok(events) => {
                succeeded(&mut runtime, &mut battle, &events, false);
                NextState::set_if_neq(&mut next, screen_for(&battle));
            }
            Err(error) => fail(&mut battle, error.to_string()),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Private client modules share the battle controller contract without exposing a public API."
)]
pub(super) mod tests;
