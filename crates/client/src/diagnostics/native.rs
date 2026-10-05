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

use crate::platform::PresentationReady;

#[derive(Resource)]
struct NativeCapture {
    path: PathBuf,
    started: Instant,
    ready_frames: u8,
    requested: bool,
}

pub(super) fn configure(app: &mut App) {
    match screenshot_argument(std::env::args_os().skip(1)) {
        Ok(Some(path)) => {
            app.insert_resource(NativeCapture {
                path,
                started: Instant::now(),
                ready_frames: 0,
                requested: false,
            })
            .add_systems(Update, request_capture);
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
) -> Result<Option<PathBuf>, String> {
    let mut arguments = arguments.into_iter();
    let Some(argument) = arguments.next() else {
        return Ok(None);
    };
    if argument != "--screenshot" {
        return Err("Test-support launch accepts only --screenshot <output.png>".into());
    }
    let Some(path) = arguments.next().filter(|path| !path.is_empty()) else {
        return Err("--screenshot requires a PNG output path".into());
    };
    if arguments.next().is_some() {
        return Err("Unexpected extra screenshot arguments".into());
    }
    let path = PathBuf::from(path);
    if path.extension().is_none_or(|extension| extension != "png") {
        return Err("--screenshot output must have a .png extension".into());
    }
    Ok(Some(path))
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
    capture.ready_frames = if readiness.ready {
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
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            match save_capture(&captured, &path) {
                Ok(()) => {
                    info!("Native title screenshot saved to {}", path.display());
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
            Some(PathBuf::from("out/title.png"))
        );
        for arguments in [
            vec!["--screenshot"],
            vec!["--screenshot", ""],
            vec!["--screenshot", "bad.jpg"],
            vec!["--screenshot", "title.png", "extra"],
            vec!["--unknown"],
        ] {
            assert!(screenshot_argument(arguments.into_iter().map(OsString::from)).is_err());
        }
    }
}
