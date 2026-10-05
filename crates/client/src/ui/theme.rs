#![expect(
    clippy::redundant_pub_crate,
    reason = "This constants-only theme is shared by private sibling presentation modules; its values are intentionally crate-local."
)]

use bevy::prelude::*;

pub(crate) const BACKGROUND: Color = Color::srgb(0.055, 0.065, 0.085);
pub(crate) const FOREGROUND: Color = Color::srgb(0.96, 0.94, 0.89);
pub(crate) const SECONDARY: Color = Color::srgb(0.67, 0.70, 0.75);
pub(crate) const ACCENT: Color = Color::srgb(0.87, 0.73, 0.47);
pub(crate) const BUTTON: Color = Color::srgb(0.12, 0.14, 0.18);
pub(crate) const HOVER: Color = Color::srgb(0.19, 0.22, 0.27);
pub(crate) const PRESSED: Color = Color::srgb(0.28, 0.25, 0.19);
pub(crate) const BORDER: Color = Color::srgb(0.34, 0.37, 0.42);
pub(crate) const CONTROL_HEIGHT: f32 = 52.0;
pub(crate) const SCREEN_PADDING: f32 = 24.0;
pub(crate) const GAP: f32 = 24.0;
pub(crate) const COMPACT_GAP: f32 = 14.0;
pub(crate) const PANEL_MAX_WIDTH: f32 = 640.0;
pub(crate) const CONTROL_WIDTH_PERCENT: f32 = 76.0;
pub(crate) const CONTROL_MAX_WIDTH: f32 = 280.0;
pub(crate) const BORDER_WIDTH: f32 = 2.0;
pub(crate) const CORNER_RADIUS: f32 = 4.0;
pub(crate) const DIVIDER_WIDTH: f32 = 48.0;
pub(crate) const DIVIDER_HEIGHT: f32 = 2.0;
pub(crate) const HEADING_WIDTH_RATIO: f32 = 0.1;
pub(crate) const HEADING_MIN: f32 = 32.0;
pub(crate) const HEADING_MAX: f32 = 64.0;
pub(crate) const HEADING_COMPACT: f32 = 40.0;
pub(crate) const BODY_SIZE: f32 = 18.0;
pub(crate) const BODY_COMPACT_SIZE: f32 = 16.0;
pub(crate) const CAPTION_SIZE: f32 = 13.0;
pub(crate) const CONTROL_TEXT_SIZE: f32 = 18.0;
pub(crate) const COMPACT_HEIGHT: f32 = 440.0;
pub(crate) const COMPACT_WIDTH: f32 = 600.0;
