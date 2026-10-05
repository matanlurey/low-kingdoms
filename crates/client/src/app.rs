//! Application bootstrap. Desktop and browser use the same screen and interaction plugins.

use std::process::{ExitCode, Termination as _};

use bevy::prelude::*;

use crate::{platform, screens::ScreensPlugin, ui::UiInteractionPlugin};

/// Opens the Low Kingdoms application and runs its platform event loop.
///
/// The return value preserves normal Bevy exit requests, including failed native capture checks.
/// The browser event loop is scheduled by Bevy and continues after this function returns.
#[must_use]
pub fn run() -> ExitCode {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(platform::window_plugin()))
        .insert_resource(ClearColor(crate::ui::theme::BACKGROUND))
        .insert_resource(platform::PresentationPlatform::current())
        .add_plugins(ClientPlugin);
    platform::configure_runtime(&mut app);
    app.run().report()
}

pub(crate) struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ScreensPlugin, UiInteractionPlugin))
            .add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Name::new("Application UI camera")));
}
