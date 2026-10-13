//! Headless acceptance of the actual production controller, screens, and reconciliation systems.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::{
    input::InputPlugin,
    input_focus::{InputDispatchPlugin, InputFocusPlugin},
    prelude::*,
    state::app::StatesPlugin,
    time::TimeUpdateStrategy,
    window::PrimaryWindow,
};
use low_kingdoms_battle::{ai, content::Content, session::Session};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    model::{CardId, Outcome, Phase, PlayerId},
};

use super::{Activate, Battle, LocalSave, Panel, Runtime, Speed};
use crate::{
    app::ClientPlugin,
    platform::{
        PresentationPlatform,
        storage::{StorageError, Store},
    },
    screens::{Screen, ScreenRoot},
    ui::Control,
};

#[derive(Default)]
struct Slot {
    data: Option<String>,
    reads: usize,
    writes: usize,
    clears: usize,
    fail_read: bool,
    write_failure: Option<StorageError>,
    fail_clear: bool,
}

#[derive(Clone, Default)]
struct MemoryStore(Arc<Mutex<Slot>>);

impl Store for MemoryStore {
    fn read(&mut self) -> Result<Option<String>, StorageError> {
        let mut slot = self.0.lock().expect("test storage lock");
        slot.reads += 1;
        if slot.fail_read {
            return Err(StorageError::Read("test read failure".to_owned()));
        }
        Ok(slot.data.clone())
    }

    fn write(&mut self, data: &str) -> Result<(), StorageError> {
        let mut slot = self.0.lock().expect("test storage lock");
        slot.writes += 1;
        if let Some(error) = &slot.write_failure {
            return Err(error.clone());
        }
        slot.data = Some(data.to_owned());
        drop(slot);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), StorageError> {
        let mut slot = self.0.lock().expect("test storage lock");
        slot.clears += 1;
        if slot.fail_clear {
            return Err(StorageError::Clear("test cleanup failure".to_owned()));
        }
        slot.data = None;
        drop(slot);
        Ok(())
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Sibling navigation tests inject storage before production plugin startup."
)]
pub(crate) fn install_empty_store(app: &mut App) {
    app.insert_resource(Runtime {
        store: Some(Box::new(MemoryStore::default())),
        ..default()
    });
}

fn application(store: MemoryStore) -> App {
    application_with_store(Box::new(store))
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Native storage tests exercise corrupt physical files through the production controller."
)]
pub(crate) fn application_with_store(store: Box<dyn Store>) -> App {
    let mut app = App::new();
    app.insert_resource(Runtime {
        store: Some(store),
        ..default()
    })
    .add_plugins((
        MinimalPlugins,
        StatesPlugin,
        InputPlugin,
        InputFocusPlugin,
        InputDispatchPlugin,
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )))
    .insert_resource(PresentationPlatform {
        supports_quit: true,
    })
    .init_resource::<Assets<Font>>()
    .add_plugins(ClientPlugin);
    app.world_mut().spawn((
        Window {
            focused: false,
            resolution: (1280, 800).into(),
            ..default()
        },
        PrimaryWindow,
    ));
    app.update();
    app.update();
    app
}

fn empty_application() -> (App, MemoryStore) {
    let store = MemoryStore::default();
    (application(store.clone()), store)
}

fn activate(app: &mut App, control: Control) {
    app.world_mut().write_message(Activate(control));
    app.update();
    app.update();
}

fn focus(app: &mut App, active: bool) {
    app.world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>()
        .single_mut(app.world_mut())
        .expect("single test window")
        .focused = active;
}

fn revision(app: &App) -> u64 {
    app.world().resource::<Battle>().revision
}

fn state(app: &App) -> Screen {
    *app.world().resource::<State<Screen>>().get()
}

fn session_save(app: &App) -> String {
    app.world()
        .resource::<Runtime>()
        .session
        .as_ref()
        .expect("active session")
        .save()
        .expect("save")
}

fn assert_unique_screen(app: &mut App, screen: Screen) {
    assert_eq!(state(app), screen);
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

fn resume_fixture(seed: u64, human_decision: Option<Command>, pace: Speed) -> (App, MemoryStore) {
    let content = Content::load().expect("embedded content");
    let mut session = Session::new(&content, seed).expect("fixture session");
    if let Some(command) = human_decision {
        session.apply_human(&command).expect("fixture decision");
    }
    let store = MemoryStore::default();
    store.0.lock().expect("slot").data = Some(
        serde_json::to_string(&LocalSave {
            version: 1,
            speed: pace,
            session: session.save().expect("fixture save"),
        })
        .expect("local envelope"),
    );
    let mut app = application(store.clone());
    assert!(app.world().resource::<Battle>().resume_available);
    assert!(
        app.world_mut()
            .query::<&Control>()
            .iter(app.world())
            .any(|control| *control == Control::Resume),
        "a loaded battle must have a visible Resume control"
    );
    activate(&mut app, Control::Resume);
    (app, store)
}

fn human_command(app: &mut App, command: &Command) {
    focus(app, false);
    match command {
        Command::KeepHand => activate(app, Control::KeepHand),
        Command::MulliganHand => activate(app, Control::Mulligan),
        Command::ChooseInitialResources { cards } => {
            for &card in cards {
                activate(app, Control::Card(card));
            }
            activate(app, Control::ConfirmResources);
        }
        Command::ChooseRegroupResource { card } => {
            if let Some(card) = card {
                activate(app, Control::Card(*card));
            }
            activate(app, Control::ConfirmResources);
        }
        Command::PlayUnit { card } => {
            activate(app, Control::Card(*card));
            activate(app, Control::PlayUnit);
        }
        Command::Attack { attacker, target } => {
            activate(app, Control::Card(*attacker));
            activate(app, Control::AttackUnit);
            assert_eq!(app.world().resource::<Battle>().attacker, Some(*attacker));
            match target {
                AttackTarget::Unit(card) => activate(app, Control::Card(*card)),
                AttackTarget::Stronghold(_) => activate(app, Control::EnemyStronghold),
            }
        }
        Command::Pass => activate(app, Control::Pass),
        Command::ClaimInitiative => {
            activate(app, Control::More);
            activate(app, Control::ClaimInitiative);
        }
    }
    if !app
        .world()
        .resource::<Runtime>()
        .session
        .as_ref()
        .expect("session")
        .is_finished()
    {
        assert!(app.world().resource::<Battle>().error.is_none());
    }
}

fn ai_action(app: &mut App) {
    focus(app, true);
    let before = revision(app);
    for _ in 0..16 {
        app.update();
        if revision(app) > before {
            break;
        }
    }
    assert_eq!(revision(app), before + 1, "exactly one AI action");
    focus(app, false);
    app.update();
}

#[test]
fn play_initializes_real_mulligan_and_saves_without_frame_write_spam() {
    let (mut app, store) = empty_application();
    assert_unique_screen(&mut app, Screen::Title);
    activate(&mut app, Control::Play);
    assert_unique_screen(&mut app, Screen::Mulligan);
    let battle = app.world().resource::<Battle>();
    let view = battle.view.as_ref().expect("human projection");
    assert_eq!(view.player, PlayerId::Zero);
    assert_eq!(view.hand.len(), 6);
    assert_eq!(view.phase, Phase::Mulligan);
    assert_eq!(view.players[1].hand_size, 6);
    assert!(battle.legal.contains(&Command::KeepHand));
    assert_eq!(store.0.lock().expect("slot").writes, 1);
    let before = session_save(&app);
    for _ in 0..12 {
        app.update();
    }
    assert_eq!(session_save(&app), before);
    assert_eq!(store.0.lock().expect("slot").writes, 1);
    human_command(&mut app, &Command::MulliganHand);
    assert_eq!(revision(&app), 1);
    assert!(
        app.world()
            .resource::<Battle>()
            .view
            .as_ref()
            .expect("view")
            .decision_submitted
    );
    assert_eq!(
        app.world().resource::<Battle>().notification,
        "Your decision is saved; waiting for opponent."
    );
    assert_eq!(store.0.lock().expect("slot").writes, 2);
    ai_action(&mut app);
    assert_eq!(
        app.world()
            .resource::<Battle>()
            .view
            .as_ref()
            .expect("view")
            .phase,
        Phase::InitialResourcing
    );
    assert_unique_screen(&mut app, Screen::Battle);
    assert_eq!(store.0.lock().expect("slot").writes, 3);
}

#[test]
fn invalid_selection_and_unavailable_controls_do_not_change_authoritative_state() {
    let (mut app, store) = resume_fixture(42, None, Speed::Normal);
    let before = session_save(&app);
    let writes = store.0.lock().expect("slot").writes;
    activate(&mut app, Control::Pass);
    activate(&mut app, Control::ClaimInitiative);
    activate(&mut app, Control::Card(CardId(999)));
    activate(&mut app, Control::PlayUnit);
    assert_eq!(session_save(&app), before);
    assert_eq!(revision(&app), 0);
    assert_eq!(store.0.lock().expect("slot").writes, writes);
    activate(&mut app, Control::Close);
    assert_eq!(app.world().resource::<Battle>().selected, None);
    assert_eq!(app.world().resource::<Battle>().attacker, None);
}

#[test]
fn normal_fast_and_instant_ai_timing_respects_focus_and_never_catches_up() {
    for (speed, frames) in [(Speed::Normal, 8), (Speed::Fast, 2), (Speed::Instant, 1)] {
        let (mut app, _) = resume_fixture(42, Some(Command::KeepHand), speed);
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(revision(&app), 1, "inactive AI is paused");
        focus(&mut app, true);
        app.update();
        assert_eq!(revision(&app), 1, "first active frame resets scheduling");
        for _ in 0..frames - 1 {
            app.update();
            assert_eq!(revision(&app), 1, "delay not elapsed yet");
        }
        app.update();
        assert_eq!(revision(&app), 2);
        focus(&mut app, false);
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(30)));
        for _ in 0..4 {
            app.update();
        }
        assert_eq!(revision(&app), 2);
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )));
        focus(&mut app, true);
        app.update();
        assert_eq!(revision(&app), 2, "background time is never caught up");
        app.update();
        assert!(
            revision(&app) <= 3,
            "at most one opponent decision per frame"
        );
    }
}

#[test]
fn failed_save_pauses_commands_and_retry_repeats_only_the_write() {
    let (mut app, store) = resume_fixture(42, None, Speed::Instant);
    store.0.lock().expect("slot").write_failure =
        Some(StorageError::Write("test write failure".to_owned()));
    activate(&mut app, Control::KeepHand);
    assert_eq!(revision(&app), 1);
    assert!(app.world().resource::<Battle>().error.is_some());
    assert!(app.world().resource::<Runtime>().dirty);
    let before = session_save(&app);
    activate(&mut app, Control::Mulligan);
    focus(&mut app, true);
    for _ in 0..16 {
        app.update();
    }
    assert_eq!(session_save(&app), before);
    focus(&mut app, false);
    let attempts = store.0.lock().expect("slot").writes;
    store.0.lock().expect("slot").write_failure = None;
    activate(&mut app, Control::Retry);
    assert!(app.world().resource::<Battle>().error.is_none());
    assert!(!app.world().resource::<Runtime>().dirty);
    assert_eq!(
        session_save(&app),
        before,
        "retry must not execute the accepted keep twice"
    );
    assert_eq!(store.0.lock().expect("slot").writes, attempts + 1);
    ai_action(&mut app);
    assert_eq!(revision(&app), 2);
}

#[test]
fn refresh_and_resume_preserve_pending_ai_and_do_not_duplicate_accepted_decisions() {
    let (mut first, store) = resume_fixture(42, Some(Command::KeepHand), Speed::Instant);
    let before = session_save(&first);
    let mut expected =
        Session::restore(&Content::load().expect("content"), &before).expect("restore oracle");
    expected
        .step_ai(&Content::load().expect("content"))
        .expect("one AI decision");
    let mut refreshed = application(store);
    assert_unique_screen(&mut refreshed, Screen::Title);
    assert!(refreshed.world().resource::<Battle>().resume_available);
    activate(&mut refreshed, Control::Resume);
    assert_eq!(session_save(&refreshed), before);
    ai_action(&mut refreshed);
    assert_eq!(
        session_save(&refreshed),
        expected.save().expect("oracle save")
    );
    assert_eq!(revision(&refreshed), 2);
    ai_action(&mut first);
    assert_eq!(session_save(&first), session_save(&refreshed));
    assert_eq!(
        first.world().resource::<Battle>().history,
        refreshed.world().resource::<Battle>().history
    );
    activate(&mut refreshed, Control::MainMenu);
    assert_unique_screen(&mut refreshed, Screen::Title);
    assert!(refreshed.world().resource::<Battle>().resume_available);
    activate(&mut refreshed, Control::Resume);
    assert_unique_screen(&mut refreshed, Screen::Battle);
    assert_eq!(revision(&refreshed), 2);
}

#[test]
fn quota_error_preserves_old_slot_and_retry_saves_the_accepted_command_once() {
    let (mut app, store) = resume_fixture(42, None, Speed::Instant);
    let old_slot = store.0.lock().expect("slot").data.clone();
    store.0.lock().expect("slot").write_failure = Some(StorageError::Quota);
    activate(&mut app, Control::KeepHand);
    assert_eq!(revision(&app), 1);
    assert!(
        app.world()
            .resource::<Battle>()
            .error
            .as_ref()
            .expect("quota diagnostic")
            .contains("Local storage is full")
    );
    assert_eq!(store.0.lock().expect("slot").data, old_slot);
    assert!(app.world().resource::<Runtime>().dirty);
    let accepted = session_save(&app);
    focus(&mut app, true);
    for _ in 0..16 {
        app.update();
    }
    assert_eq!(
        session_save(&app),
        accepted,
        "quota failures pause opponent execution"
    );
    focus(&mut app, false);
    store.0.lock().expect("slot").write_failure = None;
    activate(&mut app, Control::Retry);
    assert_eq!(revision(&app), 1);
    assert_eq!(session_save(&app), accepted);
    assert!(app.world().resource::<Battle>().error.is_none());
    let encoded = store
        .0
        .lock()
        .expect("slot")
        .data
        .clone()
        .expect("retried save");
    let saved: LocalSave = serde_json::from_str(&encoded).expect("local envelope");
    assert_eq!(saved.session, accepted);
    ai_action(&mut app);
    assert_eq!(revision(&app), 2);
}

fn result_fixture(health: [u32; 2], cost: u32, power: u32) -> (App, MemoryStore) {
    let cards = format!("[(id: (1), name: \"Unit\", cost: {cost}, power: {power}, health: 1)]");
    let ids = ["(1)"; 20].join(",");
    let decks = format!(
        "[(cards: [{ids}], stronghold_health: {}),(cards: [{ids}], stronghold_health: {})]",
        health[0], health[1]
    );
    let content = Content::parse(&cards, &decks).expect("validated result fixture");
    let session = Session::new(&content, 2).expect("real rules match");
    let catalog = content.cards().to_vec();
    let (mut app, store) = empty_application();
    {
        let mut runtime = app.world_mut().resource_mut::<Runtime>();
        runtime.content = Some(content);
        runtime.session = Some(session);
        runtime.last_seed = Some(2);
    }
    {
        let mut battle = app.world_mut().resource_mut::<Battle>();
        battle.catalog = catalog;
        battle.speed = Speed::Instant;
        battle.resume_available = true;
    }
    activate(&mut app, Control::Resume);
    (app, store)
}

fn result_choice(battle: &Battle, win: bool) -> Command {
    match battle.view.as_ref().expect("view").phase {
        Phase::Mulligan => Command::KeepHand,
        Phase::InitialResourcing => Command::ChooseInitialResources { cards: vec![] },
        Phase::Regroup => Command::ChooseRegroupResource { card: None },
        Phase::Action if win => battle
            .legal
            .iter()
            .find(|command| {
                matches!(
                    command,
                    Command::Attack {
                        target: AttackTarget::Stronghold(_),
                        ..
                    }
                )
            })
            .or_else(|| {
                battle
                    .legal
                    .iter()
                    .find(|command| matches!(command, Command::PlayUnit { .. }))
            })
            .or_else(|| {
                battle
                    .legal
                    .iter()
                    .find(|command| **command == Command::ClaimInitiative)
            })
            .cloned()
            .unwrap_or(Command::Pass),
        Phase::Action => Command::Pass,
        Phase::Finished => panic!("finished fixture has no command"),
    }
}

#[test]
fn real_rules_outcomes_render_victory_defeat_and_draw_and_clear_the_active_slot() {
    for (health, cost, power, outcome, label) in [
        ([1000, 1], 0, 1, Outcome::Winner(PlayerId::Zero), "Victory"),
        ([1, 1000], 0, 1, Outcome::Winner(PlayerId::One), "Defeat"),
        ([1, 1], u32::MAX, 0, Outcome::Draw, "Draw"),
    ] {
        let (mut app, store) = result_fixture(health, cost, power);
        for _ in 0..512 {
            let battle = app.world().resource::<Battle>();
            if battle.view.as_ref().expect("view").outcome.is_some() {
                break;
            }
            if app
                .world()
                .resource::<Runtime>()
                .session
                .as_ref()
                .expect("session")
                .needs_ai()
            {
                ai_action(&mut app);
            } else {
                let command = result_choice(battle, outcome == Outcome::Winner(PlayerId::Zero));
                human_command(&mut app, &command);
            }
        }
        assert_eq!(
            app.world()
                .resource::<Battle>()
                .view
                .as_ref()
                .expect("view")
                .outcome,
            Some(outcome)
        );
        assert_unique_screen(&mut app, Screen::Results);
        let round = app
            .world()
            .resource::<Battle>()
            .view
            .as_ref()
            .expect("view")
            .round;
        let texts: Vec<_> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|text| text.0.as_str())
            .collect();
        assert!(
            texts.contains(&label),
            "the actual Results UI must identify {label}"
        );
        assert!(texts.contains(&format!("Battle ended in round {round}").as_str()));
        assert!(!app.world().resource::<Battle>().resume_available);
        assert!(store.0.lock().expect("slot").data.is_none());
        assert!(store.0.lock().expect("slot").clears > 0);
        assert!(app.world().resource::<Battle>().error.is_none());
    }
}

#[test]
fn existing_save_requires_confirmation_and_corrupt_slot_remains_replaceable() {
    let (mut app, store) = resume_fixture(42, None, Speed::Normal);
    activate(&mut app, Control::MainMenu);
    let original = store.0.lock().expect("slot").data.clone();
    activate(&mut app, Control::Play);
    assert_eq!(app.world().resource::<Battle>().panel, Panel::Replace);
    assert_eq!(store.0.lock().expect("slot").data, original);
    activate(&mut app, Control::Cancel);
    assert_eq!(store.0.lock().expect("slot").data, original);
    activate(&mut app, Control::Play);
    activate(&mut app, Control::ConfirmReplace);
    assert_unique_screen(&mut app, Screen::Mulligan);
    assert_ne!(
        app.world()
            .resource::<Runtime>()
            .session
            .as_ref()
            .expect("session")
            .seed(),
        42
    );
    assert_ne!(store.0.lock().expect("slot").data, original);

    let corrupt = MemoryStore::default();
    corrupt.0.lock().expect("slot").data = Some("damaged save".to_owned());
    let mut app = application(corrupt.clone());
    assert!(!app.world().resource::<Battle>().resume_available);
    activate(&mut app, Control::Play);
    assert_eq!(app.world().resource::<Battle>().panel, Panel::Replace);
    activate(&mut app, Control::ConfirmReplace);
    assert_unique_screen(&mut app, Screen::Mulligan);
    let encoded = corrupt
        .0
        .lock()
        .expect("slot")
        .data
        .clone()
        .expect("replacement save");
    let saved: LocalSave = serde_json::from_str(&encoded).expect("valid replacement envelope");
    assert!(Session::restore(&Content::load().expect("content"), &saved.session).is_ok());
}

#[test]
fn unavailable_read_can_retry_without_discarding_or_starting_a_match() {
    let store = MemoryStore::default();
    store.0.lock().expect("slot").fail_read = true;
    let mut app = application(store.clone());
    assert!(app.world().resource::<Battle>().error.is_some());
    activate(&mut app, Control::Play);
    assert!(app.world().resource::<Runtime>().session.is_none());
    assert_eq!(store.0.lock().expect("slot").writes, 0);
    store.0.lock().expect("slot").fail_read = false;
    activate(&mut app, Control::Retry);
    assert!(app.world().resource::<Battle>().error.is_none());
    activate(&mut app, Control::Play);
    assert_unique_screen(&mut app, Screen::Mulligan);
    assert_eq!(store.0.lock().expect("slot").writes, 1);
}

#[test]
fn buffered_ai_resources_preserve_draft_and_reverse_selection_commits_legally() {
    let (mut app, _) = resume_fixture(42, Some(Command::KeepHand), Speed::Instant);
    ai_action(&mut app);
    let mut selected: Vec<_> = app
        .world()
        .resource::<Battle>()
        .view
        .as_ref()
        .expect("view")
        .hand
        .iter()
        .take(2)
        .map(|card| card.id)
        .collect();
    selected.sort_unstable_by(|a, b| b.cmp(a));
    for &card in &selected {
        activate(&mut app, Control::Card(card));
    }
    activate(&mut app, Control::Hand);
    let before = session_save(&app);
    assert_eq!(
        app.world().resource::<Battle>().resource_selection,
        selected
    );
    assert_eq!(app.world().resource::<Battle>().panel, Panel::Hand);
    ai_action(&mut app);
    assert_eq!(
        app.world()
            .resource::<Battle>()
            .view
            .as_ref()
            .expect("view")
            .phase,
        Phase::InitialResourcing
    );
    assert_eq!(
        app.world().resource::<Battle>().resource_selection,
        selected
    );
    assert_eq!(app.world().resource::<Battle>().panel, Panel::Hand);
    assert_ne!(
        session_save(&app),
        before,
        "AI decision is authoritative even when buffered"
    );
    assert_eq!(
        app.world().resource::<Battle>().notification,
        "Choose up to two cards to resource."
    );
    let before_revision = revision(&app);
    activate(&mut app, Control::ConfirmResources);
    assert_eq!(revision(&app), before_revision + 1);
    let battle = app.world().resource::<Battle>();
    let view = battle.view.as_ref().expect("view");
    assert_eq!(view.phase, Phase::Action);
    assert_eq!(view.resources.len(), 2);
    assert!(selected.iter().all(|id| {
        view.resources
            .iter()
            .any(|resource| resource.card.id == *id)
    }));
    assert!(battle.resource_selection.is_empty());
}

#[test]
fn completed_clear_failure_keeps_finished_fallback_and_retry_only_cleans_storage() {
    let (mut app, store) = resume_fixture(42, None, Speed::Instant);
    store.0.lock().expect("slot").fail_clear = true;
    for _ in 0..512 {
        let battle = app.world().resource::<Battle>();
        let view = battle.view.as_ref().expect("view");
        if view.outcome.is_some() {
            break;
        }
        if app
            .world()
            .resource::<Runtime>()
            .session
            .as_ref()
            .expect("session")
            .needs_ai()
        {
            ai_action(&mut app);
        } else {
            let command = ai::choose(view, &battle.legal, &battle.catalog).expect("human command");
            human_command(&mut app, &command);
        }
    }
    assert_unique_screen(&mut app, Screen::Results);
    assert!(app.world().resource::<Battle>().error.is_some());
    assert!(!app.world().resource::<Battle>().resume_available);
    let before_revision = revision(&app);
    let terminal = session_save(&app);
    let stored = store
        .0
        .lock()
        .expect("slot")
        .data
        .clone()
        .expect("finished fallback");
    let saved: LocalSave = serde_json::from_str(&stored).expect("local envelope");
    assert!(
        Session::restore(&Content::load().expect("content"), &saved.session)
            .expect("finished snapshot")
            .is_finished()
    );
    store.0.lock().expect("slot").fail_clear = false;
    activate(&mut app, Control::Retry);
    assert_eq!(revision(&app), before_revision);
    assert_eq!(session_save(&app), terminal);
    assert!(store.0.lock().expect("slot").data.is_none());
    assert!(app.world().resource::<Battle>().error.is_none());
    assert!(!app.world().resource::<Battle>().resume_available);
    assert_unique_screen(&mut app, Screen::Results);
}

fn first_deployment(app: &mut App) -> CardId {
    for _ in 0..16 {
        let battle = app.world().resource::<Battle>();
        let view = battle.view.as_ref().expect("view");
        if view.phase == Phase::Action
            && !app
                .world()
                .resource::<Runtime>()
                .session
                .as_ref()
                .expect("session")
                .needs_ai()
        {
            break;
        }
        if app
            .world()
            .resource::<Runtime>()
            .session
            .as_ref()
            .expect("session")
            .needs_ai()
        {
            ai_action(app);
        } else {
            let command = ai::choose(view, &battle.legal, &battle.catalog).expect("setup decision");
            human_command(app, &command);
        }
    }
    let deployment = app
        .world()
        .resource::<Battle>()
        .legal
        .iter()
        .find_map(|command| {
            if let Command::PlayUnit { card } = command {
                Some(*card)
            } else {
                None
            }
        })
        .expect("affordable unit");
    human_command(app, &Command::PlayUnit { card: deployment });
    deployment
}

#[test]
fn opponent_action_preserves_visible_inspection_and_scroll_and_reconciles_unit_text() {
    let (mut app, _) = resume_fixture(42, None, Speed::Instant);
    let deployment = first_deployment(&mut app);
    // Record battlefield/sidebar panes before opening a preview. The dismissible preview may
    // rebuild its own content when a revision changes, while battlefield scroll must persist.
    let scroll: Vec<_> = app
        .world_mut()
        .query_filtered::<(Entity, &mut ScrollPosition), With<super::presentation::ScrollPane>>()
        .iter_mut(app.world_mut())
        .map(|(entity, mut scroll)| {
            scroll.x = 12.0;
            scroll.y = 34.0;
            (entity, scroll.0)
        })
        .collect();
    assert!(!scroll.is_empty());
    activate(&mut app, Control::Card(deployment));
    assert_eq!(app.world().resource::<Battle>().panel, Panel::Inspect);
    ai_action(&mut app);
    let battle = app.world().resource::<Battle>();
    assert_eq!(battle.selected, Some(deployment));
    assert_eq!(battle.panel, Panel::Inspect);
    for (entity, expected) in scroll {
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(entity)
                .expect("preserved pane")
                .0,
            expected
        );
    }
    let unit = battle.view.as_ref().expect("view").players[0]
        .units
        .iter()
        .find(|unit| unit.card.id == deployment)
        .expect("visible deployed unit")
        .clone();
    assert!(!unit.ready, "the authoritative engine exhausts deployments");
    let faces: Vec<_> = app
        .world_mut()
        .query_filtered::<(Entity, &Control), With<super::presentation::CardFace>>()
        .iter(app.world())
        .filter_map(|(entity, control)| {
            if *control == Control::Card(deployment) {
                Some(entity)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        faces.len(),
        1,
        "reconciliation does not duplicate card faces"
    );
    let text = app
        .world()
        .get::<Children>(faces[0])
        .expect("card text child")
        .iter()
        .find_map(|child| app.world().get::<Text>(child))
        .expect("unit text");
    assert!(text.0.contains("Exhausted"));
    assert!(text.0.contains(&format!(
        "{} / {}",
        unit.power,
        u64::from(unit.health).saturating_sub(unit.damage)
    )));
    assert!(text.0.contains(&format!("Damage: {}", unit.damage)));
}

fn full_match_choice(battle: &Battle, unit_done: bool, stronghold_done: bool) -> Command {
    let explicit_attack = battle.legal.iter().find(|command| match command {
        Command::Attack {
            target: AttackTarget::Unit(_),
            ..
        } => !unit_done,
        Command::Attack {
            target: AttackTarget::Stronghold(_),
            ..
        } => !stronghold_done,
        _ => false,
    });
    explicit_attack.cloned().unwrap_or_else(|| {
        ai::choose(
            battle.view.as_ref().expect("human view"),
            &battle.legal,
            &battle.catalog,
        )
        .expect("human heuristic fixture")
    })
}

#[test]
fn production_control_flow_completes_matches_and_restarts_without_entity_accumulation() {
    let (mut app, store) = resume_fixture(42, None, Speed::Instant);
    let mut deployed = false;
    let mut attacked_unit = false;
    let mut attacked_stronghold = false;
    let mut passed = false;
    let mut claimed = false;
    let mut rounds = false;
    let mut last_seed = 42;
    for _ in 0..3 {
        for _ in 0..512 {
            let battle = app.world().resource::<Battle>();
            let view = battle.view.as_ref().expect("human view");
            if view.outcome.is_some() {
                break;
            }
            rounds |= view.round > 1;
            if app
                .world()
                .resource::<Runtime>()
                .session
                .as_ref()
                .expect("session")
                .needs_ai()
            {
                ai_action(&mut app);
            } else {
                let command = full_match_choice(battle, attacked_unit, attacked_stronghold);
                deployed |= matches!(command, Command::PlayUnit { .. });
                attacked_unit |= matches!(
                    command,
                    Command::Attack {
                        target: AttackTarget::Unit(_),
                        ..
                    }
                );
                attacked_stronghold |= matches!(
                    command,
                    Command::Attack {
                        target: AttackTarget::Stronghold(_),
                        ..
                    }
                );
                passed |= matches!(command, Command::Pass);
                claimed |= matches!(command, Command::ClaimInitiative);
                let before = revision(&app);
                human_command(&mut app, &command);
                assert_eq!(revision(&app), before + 1);
            }
            let battle = app.world().resource::<Battle>();
            assert_eq!(battle.view.as_ref().expect("view").player, PlayerId::Zero);
            assert_eq!(
                battle.revision,
                app.world()
                    .resource::<Runtime>()
                    .session
                    .as_ref()
                    .expect("session")
                    .revision()
            );
            let current = state(&app);
            assert_unique_screen(&mut app, current);
        }
        assert!(
            app.world()
                .resource::<Runtime>()
                .session
                .as_ref()
                .expect("session")
                .is_finished()
        );
        assert_unique_screen(&mut app, Screen::Results);
        assert!(!app.world().resource::<Battle>().resume_available);
        assert!(store.0.lock().expect("slot").data.is_none());
        let old_nodes: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<Node>>()
            .iter(app.world())
            .collect();
        activate(&mut app, Control::PlayAgain);
        assert_unique_screen(&mut app, Screen::Mulligan);
        let seed = app
            .world()
            .resource::<Runtime>()
            .session
            .as_ref()
            .expect("new session")
            .seed();
        assert_ne!(seed, last_seed);
        last_seed = seed;
        assert_eq!(revision(&app), 0);
        assert!(
            old_nodes
                .iter()
                .all(|entity| app.world().get_entity(*entity).is_err())
        );
    }
    assert!(deployed && attacked_unit && attacked_stronghold && rounds);
    assert!(passed && claimed);
    activate(&mut app, Control::MainMenu);
    assert_unique_screen(&mut app, Screen::Title);
}
