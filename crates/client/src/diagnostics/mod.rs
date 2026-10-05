use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod native;

#[expect(
    clippy::redundant_pub_crate,
    reason = "Only the private platform edge configures test diagnostics."
)]
pub(crate) fn configure(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Last, browser::publish_snapshot);
    #[cfg(not(target_arch = "wasm32"))]
    native::configure(app);
}
