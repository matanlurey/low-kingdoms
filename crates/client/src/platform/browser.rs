use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowEvent, WindowResized, WindowScaleFactorChanged},
};
use wasm_bindgen::prelude::wasm_bindgen;

use super::PresentationReady;

#[wasm_bindgen(
    inline_js = "export function announceReady() { window.dispatchEvent(new Event('low-kingdoms-ready')); } export function canvasDimensions() { const parent = document.getElementById('game-canvas')?.parentElement; if (!parent) return new Float32Array(); const size = parent.getBoundingClientRect(); return new Float32Array([size.width, size.height, window.devicePixelRatio]); }"
)]
extern "C" {
    #[wasm_bindgen(js_name = announceReady)]
    fn announce_ready_event();

    #[wasm_bindgen(js_name = canvasDimensions)]
    fn canvas_dimensions() -> Vec<f32>;
}

pub(super) fn sync_canvas_extent(
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut resized: MessageWriter<WindowResized>,
    mut rescaled: MessageWriter<WindowScaleFactorChanged>,
    mut window_events: MessageWriter<WindowEvent>,
) {
    let dimensions = canvas_dimensions();
    let [width, height, scale] = dimensions.as_slice() else {
        return;
    };
    if dimensions
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return;
    }
    let Ok((entity, mut window)) = windows.single_mut() else {
        return;
    };
    let mut resolution = window.resolution.clone();
    resolution.set_scale_factor(*scale);
    resolution.set(*width, *height);
    if resolution == window.resolution {
        return;
    }

    // Winit's device-pixel ResizeObserver box can disagree with browser DPR during
    // emulation or scale changes. Stable parent CSS dimensions keep one logical
    // coordinate system for rendering and input without resizing the parent.
    let scale_changed = (window.scale_factor() - scale).abs() > f32::EPSILON;
    window.resolution = resolution;
    let event = WindowResized {
        window: entity,
        width: window.width(),
        height: window.height(),
    };
    resized.write(event.clone());
    window_events.write(event.into());
    if scale_changed {
        let event = WindowScaleFactorChanged {
            window: entity,
            scale_factor: f64::from(*scale),
        };
        rescaled.write(event.clone());
        window_events.write(event.into());
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "Bevy injects the owned Res system parameter."
)]
pub(super) fn announce_ready(readiness: Res<PresentationReady>, mut announced: Local<bool>) {
    if readiness.ready && !*announced {
        announce_ready_event();
        *announced = true;
        info!("Low Kingdoms text and control layout ready");
    }
}
