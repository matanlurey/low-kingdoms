use bevy::{prelude::*, window::WindowResolution};

#[cfg(not(target_arch = "wasm32"))]
use bevy::window::WindowResizeConstraints;

#[cfg(target_arch = "wasm32")]
mod browser;

#[derive(Resource)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Platform configuration stays crate-local across sibling modules."
)]
pub(crate) struct PresentationPlatform {
    pub(crate) supports_quit: bool,
}

impl PresentationPlatform {
    pub(crate) const fn current() -> Self {
        Self {
            supports_quit: !cfg!(target_arch = "wasm32"),
        }
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Only the application bootstrap composes this private platform plugin."
)]
pub(crate) fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Low Kingdoms".into(),
            resolution: WindowResolution::new(1280, 720),
            #[cfg(not(target_arch = "wasm32"))]
            resize_constraints: WindowResizeConstraints {
                min_width: 320.0,
                min_height: 360.0,
                ..default()
            },
            canvas: Some("#game-canvas".into()),
            fit_canvas_to_parent: true,
            // The host suppresses only menu keys, preserving browser zoom shortcuts.
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "Runtime composition remains within the client crate."
)]
pub(crate) fn configure_runtime(app: &mut App) {
    app.init_resource::<PresentationReady>().add_systems(
        PostUpdate,
        update_readiness.after(bevy::ui::UiSystems::PostLayout),
    );
    #[cfg(target_arch = "wasm32")]
    app.add_systems(First, browser::sync_canvas_extent)
        .add_systems(PostUpdate, browser::announce_ready.after(update_readiness));
    #[cfg(feature = "test-support")]
    crate::diagnostics::configure(app);
}

/// Live text layout readiness; rendered screenshots remain a separate verification layer.
#[derive(Resource, Default)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "Readiness is shared with private platform diagnostics only."
)]
pub(crate) struct PresentationReady {
    pub(crate) ready: bool,
    settled_frames: u8,
}

fn update_readiness(
    mut readiness: ResMut<PresentationReady>,
    text: Query<(&ComputedNode, &bevy::text::TextLayoutInfo), With<Text>>,
    controls: Query<&ComputedNode, With<Button>>,
) {
    let settled = !text.is_empty()
        && !controls.is_empty()
        && text
            .iter()
            .all(|(node, layout)| !node.is_empty() && !layout.glyphs.is_empty())
        && controls.iter().all(|node| !node.is_empty());
    readiness.settled_frames = if settled {
        readiness.settled_frames.saturating_add(1)
    } else {
        0
    };
    readiness.ready = readiness.settled_frames >= 3;
}
