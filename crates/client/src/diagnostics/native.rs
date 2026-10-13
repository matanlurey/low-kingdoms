use std::{
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};

use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use low_kingdoms_rules::{
    command::{AttackTarget, Command},
    model::{CardId, Phase},
};

use crate::{
    gameplay::{Activate, Battle, CapturePauseAI, Panel},
    platform::PresentationReady,
    screens::Screen,
    ui::Control,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CaptureScene {
    #[default]
    Title,
    Mulligan,
    Battle,
    Results,
}

impl CaptureScene {
    const fn name(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Mulligan => "mulligan",
            Self::Battle => "battle",
            Self::Results => "results",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CaptureArgument {
    path: PathBuf,
    scene: CaptureScene,
}

#[derive(Resource)]
struct NativeCapture {
    path: PathBuf,
    scene: CaptureScene,
    started: Instant,
    ready_frames: u8,
    requested: bool,
    fixture_started: bool,
    fixture_ready: bool,
    fixture_steps: u16,
}

pub(super) fn configure(app: &mut App) {
    match screenshot_argument(std::env::args_os().skip(1)) {
        Ok(Some(argument)) => {
            app.insert_resource(NativeCapture {
                path: argument.path,
                scene: argument.scene,
                started: Instant::now(),
                ready_frames: 0,
                requested: false,
                fixture_started: false,
                fixture_ready: false,
                fixture_steps: 0,
            })
            .insert_resource(CapturePauseAI::default())
            .add_systems(Update, (advance_fixture, request_capture).chain());
        }
        Ok(None) => {}
        Err(message) => {
            error!("{message}");
            app.add_systems(Startup, |mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::error());
            });
        }
    }
}

fn screenshot_argument(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<CaptureArgument>, String> {
    let mut arguments = arguments.into_iter();
    let Some(argument) = arguments.next() else {
        return Ok(None);
    };
    if argument != "--screenshot" {
        return Err(
            "Test-support launch accepts --screenshot <output.png> [--scene title|mulligan|battle|results]"
                .into(),
        );
    }
    let Some(path) = arguments.next().filter(|path| !path.is_empty()) else {
        return Err("--screenshot requires a PNG output path".into());
    };
    let path = PathBuf::from(path);
    if path.extension().is_none_or(|extension| extension != "png") {
        return Err("--screenshot output must have a .png extension".into());
    }
    let scene = match arguments.next() {
        None => CaptureScene::Title,
        Some(argument) if argument == "--scene" => {
            let Some(scene) = arguments.next() else {
                return Err("--scene requires title, mulligan, battle, or results".into());
            };
            match scene.to_str() {
                Some("title") => CaptureScene::Title,
                Some("mulligan") => CaptureScene::Mulligan,
                Some("battle") => CaptureScene::Battle,
                Some("results") => CaptureScene::Results,
                _ => return Err("--scene requires title, mulligan, battle, or results".into()),
            }
        }
        Some(_) => return Err("Unexpected extra screenshot arguments".into()),
    };
    if arguments.next().is_some() {
        return Err("Unexpected extra screenshot arguments".into());
    }
    Ok(Some(CaptureArgument { path, scene }))
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy owns the injected read-only state and battle resources."
)]
fn advance_fixture(
    mut capture: ResMut<NativeCapture>,
    mut pause_ai: ResMut<CapturePauseAI>,
    battle: Res<Battle>,
    state: Res<State<Screen>>,
    mut actions: MessageWriter<Activate>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.requested || capture.fixture_ready {
        return;
    }
    if battle.error.is_some() {
        error!(
            "Native gameplay capture encountered an application error; no screenshot was accepted"
        );
        exit.write(AppExit::error());
        return;
    }
    if scene_ready(capture.scene, *state.get(), &battle) {
        capture.fixture_ready = true;
        // Freeze only scheduling after the real session reaches the requested scene.
        // Authoritative match state, focus events, and the input path remain unchanged.
        pause_ai.0 = true;
        return;
    }
    if !capture.fixture_started {
        actions.write(Activate(Control::Play));
        actions.write(Activate(Control::SpeedInstant));
        capture.fixture_started = true;
        return;
    }
    if capture.fixture_steps >= 1024 {
        error!("Native gameplay fixture exceeded its bounded command budget");
        exit.write(AppExit::error());
        return;
    }
    if battle.panel == Panel::Replace {
        actions.write(Activate(Control::ConfirmReplace));
        capture.fixture_steps += 1;
        return;
    }
    let Some(view) = battle.view.as_ref() else {
        return;
    };
    let expected_screen = match view.phase {
        Phase::Mulligan => Screen::Mulligan,
        Phase::Finished => Screen::Results,
        Phase::InitialResourcing | Phase::Regroup | Phase::Action => Screen::Battle,
    };
    if *state.get() != expected_screen {
        // Let the real screen transition render its controls before activating them.
        return;
    }
    let Some(command) = fixture_command(&battle) else {
        // No human command means the production AI or a screen transition is pending.
        return;
    };
    for control in command_controls(&command) {
        actions.write(Activate(control));
    }
    capture.fixture_steps += 1;
}

fn scene_ready(scene: CaptureScene, screen: Screen, battle: &Battle) -> bool {
    match scene {
        CaptureScene::Title => screen == Screen::Title,
        CaptureScene::Mulligan => {
            screen == Screen::Mulligan
                && battle
                    .view
                    .as_ref()
                    .is_some_and(|view| view.phase == Phase::Mulligan && !view.decision_submitted)
        }
        CaptureScene::Battle => {
            screen == Screen::Battle
                && battle.view.as_ref().is_some_and(|view| {
                    view.phase == Phase::Action
                        && view.round >= 2
                        && view.players.iter().all(|player| !player.units.is_empty())
                })
        }
        CaptureScene::Results => {
            screen == Screen::Results
                && battle
                    .view
                    .as_ref()
                    .is_some_and(|view| view.phase == Phase::Finished && view.outcome.is_some())
        }
    }
}

fn fixture_command(battle: &Battle) -> Option<Command> {
    // Fixture choices come solely from the same legal list consumed by human controls.
    // Preserve inexpensive units during resourcing so the capture exercises deployment.
    if let Some(command) = battle
        .legal
        .iter()
        .find(|command| **command == Command::KeepHand)
    {
        return Some(command.clone());
    }
    if let Some(command) = battle
        .legal
        .iter()
        .filter(|command| {
            matches!(
                command,
                Command::ChooseInitialResources { .. } | Command::ChooseRegroupResource { .. }
            )
        })
        .max_by_key(|command| resource_score(command, battle))
    {
        return Some(command.clone());
    }
    for select in [
        is_stronghold_attack,
        is_unit_deployment,
        is_unit_attack,
        is_pass,
    ] {
        if let Some(command) = battle.legal.iter().find(|command| select(command)) {
            return Some(command.clone());
        }
    }
    None
}

fn resource_score(command: &Command, battle: &Battle) -> (usize, u64) {
    let cost = |id: CardId| {
        battle
            .view
            .as_ref()
            .and_then(|view| view.hand.iter().find(|card| card.id == id))
            .and_then(|card| {
                battle
                    .catalog
                    .iter()
                    .find(|definition| definition.id == card.definition)
            })
            .map_or(0, |definition| u64::from(definition.cost))
    };
    match command {
        Command::ChooseInitialResources { cards } => (
            cards.len(),
            cards
                .iter()
                .fold(0_u64, |sum, id| sum.saturating_add(cost(*id))),
        ),
        Command::ChooseRegroupResource { card } => {
            (usize::from(card.is_some()), card.map_or(0, cost))
        }
        _ => (0, 0),
    }
}

const fn is_stronghold_attack(command: &Command) -> bool {
    matches!(
        command,
        Command::Attack {
            target: AttackTarget::Stronghold(_),
            ..
        }
    )
}

const fn is_unit_deployment(command: &Command) -> bool {
    matches!(command, Command::PlayUnit { .. })
}

const fn is_unit_attack(command: &Command) -> bool {
    matches!(
        command,
        Command::Attack {
            target: AttackTarget::Unit(_),
            ..
        }
    )
}

const fn is_pass(command: &Command) -> bool {
    matches!(command, Command::Pass)
}

fn command_controls(command: &Command) -> Vec<Control> {
    match command {
        Command::KeepHand => vec![Control::KeepHand],
        Command::ChooseInitialResources { cards } => cards
            .iter()
            .copied()
            .map(Control::Card)
            .chain([Control::ConfirmResources])
            .collect(),
        Command::ChooseRegroupResource { card } => card
            .iter()
            .copied()
            .map(Control::Card)
            .chain([Control::ConfirmResources])
            .collect(),
        Command::PlayUnit { card } => vec![Control::Card(*card), Control::PlayUnit],
        Command::Attack { attacker, target } => vec![
            Control::Card(*attacker),
            Control::AttackUnit,
            match target {
                AttackTarget::Unit(card) => Control::Card(*card),
                AttackTarget::Stronghold(_) => Control::EnemyStronghold,
            },
        ],
        Command::Pass => vec![Control::Pass],
        Command::ClaimInitiative => vec![Control::More, Control::ClaimInitiative],
        Command::MulliganHand => vec![Control::Mulligan],
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects the owned Res system parameter."
)]
fn request_capture(
    mut commands: Commands,
    mut capture: ResMut<NativeCapture>,
    readiness: Res<PresentationReady>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.started.elapsed() > Duration::from_secs(60) {
        error!("Native capture timed out before screenshot completion");
        exit.write(AppExit::error());
        return;
    }
    if capture.requested {
        return;
    }
    capture.ready_frames = if capture.fixture_ready && readiness.ready {
        capture.ready_frames.saturating_add(1)
    } else {
        0
    };
    if capture.ready_frames < 12 {
        return;
    }
    if let Some(parent) = capture.path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        error!("Cannot create native capture directory: {error}");
        exit.write(AppExit::error());
        return;
    }
    capture.requested = true;
    let path = capture.path.clone();
    let scene = capture.scene.name();
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            match save_capture(&captured, &path) {
                Ok(()) => {
                    info!("Native {scene} screenshot saved to {}", path.display());
                    exit.write(AppExit::Success);
                }
                Err(message) => {
                    error!("Native screenshot failed: {message}");
                    exit.write(AppExit::error());
                }
            }
        },
    );
}

fn save_capture(captured: &ScreenshotCaptured, path: &std::path::Path) -> Result<(), String> {
    let image = captured
        .image
        .clone()
        .try_into_dynamic()
        .map_err(|error| format!("Unsupported captured image: {error}"))?
        .to_rgb8();
    let mut dark = 0_u64;
    let mut bright = 0_u64;
    for pixel in image.pixels() {
        if pixel.0.iter().all(|channel| *channel < 70) {
            dark += 1;
        }
        if pixel.0.iter().all(|channel| *channel > 150) {
            bright += 1;
        }
    }
    if dark < 1000 || bright < 100 {
        return Err(format!(
            "Capture lacks readable UI contrast ({dark} background / {bright} bright pixels)"
        ));
    }
    image
        .save(path)
        .map_err(|error| format!("Cannot write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_arguments_validate_real_output_contract() {
        assert_eq!(screenshot_argument(Vec::<OsString>::new()).unwrap(), None);
        assert_eq!(
            screenshot_argument(["--screenshot".into(), "out/title.png".into()]).unwrap(),
            Some(CaptureArgument {
                path: PathBuf::from("out/title.png"),
                scene: CaptureScene::Title,
            })
        );
        for scene in [
            CaptureScene::Title,
            CaptureScene::Mulligan,
            CaptureScene::Battle,
            CaptureScene::Results,
        ] {
            assert_eq!(
                screenshot_argument([
                    "--screenshot".into(),
                    "out/scene.png".into(),
                    "--scene".into(),
                    scene.name().into(),
                ])
                .unwrap(),
                Some(CaptureArgument {
                    path: PathBuf::from("out/scene.png"),
                    scene
                })
            );
        }
        for arguments in [
            vec!["--screenshot"],
            vec!["--screenshot", ""],
            vec!["--screenshot", "bad.jpg"],
            vec!["--screenshot", "title.png", "extra"],
            vec!["--screenshot", "title.png", "--scene"],
            vec!["--screenshot", "title.png", "--scene", "unknown"],
            vec!["--screenshot", "title.png", "--scene", "battle", "extra"],
            vec!["--unknown"],
        ] {
            assert!(screenshot_argument(arguments.into_iter().map(OsString::from)).is_err());
        }
    }

    #[test]
    fn fixture_attacks_and_deployments_follow_contextual_control_sequences() {
        let card = CardId(1);
        assert_eq!(
            command_controls(&Command::PlayUnit { card }),
            [Control::Card(card), Control::PlayUnit]
        );
        assert_eq!(
            command_controls(&Command::Attack {
                attacker: card,
                target: AttackTarget::Unit(CardId(2))
            }),
            [
                Control::Card(card),
                Control::AttackUnit,
                Control::Card(CardId(2))
            ]
        );
        assert_eq!(
            command_controls(&Command::Attack {
                attacker: card,
                target: AttackTarget::Stronghold(low_kingdoms_rules::model::PlayerId::One)
            }),
            [
                Control::Card(card),
                Control::AttackUnit,
                Control::EnemyStronghold
            ]
        );
    }

    #[test]
    fn native_scene_driver_reaches_real_sessions_then_pauses_scheduling() {
        use bevy::{
            input::InputPlugin,
            input_focus::{InputDispatchPlugin, InputFocusPlugin},
            state::app::StatesPlugin,
            time::TimeUpdateStrategy,
            window::PrimaryWindow,
        };

        for scene in [
            CaptureScene::Mulligan,
            CaptureScene::Battle,
            CaptureScene::Results,
        ] {
            let mut app = App::new();
            crate::gameplay::tests::install_empty_store(&mut app);
            app.add_plugins((
                MinimalPlugins,
                StatesPlugin,
                InputPlugin,
                InputFocusPlugin,
                InputDispatchPlugin,
            ))
            .init_resource::<Assets<Font>>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .insert_resource(crate::platform::PresentationPlatform {
                supports_quit: true,
            })
            .insert_resource(CapturePauseAI::default())
            .insert_resource(NativeCapture {
                path: PathBuf::from("unused-headless.png"),
                scene,
                started: Instant::now(),
                ready_frames: 0,
                requested: false,
                fixture_started: false,
                fixture_ready: false,
                fixture_steps: 0,
            })
            .add_plugins(crate::app::ClientPlugin)
            .add_systems(Update, advance_fixture);
            app.world_mut().spawn((Window::default(), PrimaryWindow));
            for _ in 0..512 {
                app.update();
                assert!(app.world().resource::<Battle>().error.is_none());
                if app.world().resource::<NativeCapture>().fixture_ready {
                    break;
                }
            }
            assert!(
                app.world().resource::<NativeCapture>().fixture_ready,
                "{scene:?}"
            );
            assert!(scene_ready(
                scene,
                *app.world().resource::<State<Screen>>().get(),
                app.world().resource::<Battle>(),
            ));
            let revision = app.world().resource::<Battle>().revision;
            for _ in 0..3 {
                app.update();
            }
            assert_eq!(app.world().resource::<Battle>().revision, revision);
        }
    }
}
