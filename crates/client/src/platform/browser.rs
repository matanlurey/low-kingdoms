use bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use super::PresentationReady;

#[wasm_bindgen(
    inline_js = "export function announceReady() { window.dispatchEvent(new Event('low-kingdoms-ready')); }"
)]
extern "C" {
    #[wasm_bindgen(js_name = announceReady)]
    fn announce_ready_event();
}

pub(super) fn announce_ready(readiness: Res<PresentationReady>, mut announced: Local<bool>) {
    if readiness.ready && !*announced {
        announce_ready_event();
        *announced = true;
        info!("Low Kingdoms text and control layout ready");
    }
}
