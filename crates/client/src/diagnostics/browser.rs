use std::fmt::Write as _;

use bevy::{
    input_focus::{InputFocus, InputFocusVisible},
    prelude::*,
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{platform::PresentationReady, screens::Screen, ui::Control};

#[wasm_bindgen(
    inline_js = "let snapshot; function freeze(value) { if (value && typeof value === 'object') { Object.values(value).forEach(freeze); Object.freeze(value); } return value; } export function publishSnapshot(json) { snapshot = freeze(JSON.parse(json)); if (!Object.prototype.hasOwnProperty.call(window, '__LOW_KINGDOMS_TEST__')) { Object.defineProperty(window, '__LOW_KINGDOMS_TEST__', { get: () => snapshot, configurable: false }); } }"
)]
extern "C" {
    #[wasm_bindgen(js_name = publishSnapshot)]
    fn publish_json(json: &str);
}

pub(super) fn publish_snapshot(
    state: Res<State<Screen>>,
    readiness: Res<PresentationReady>,
    focus: Res<InputFocus>,
    visible: Res<InputFocusVisible>,
    window: Single<&Window>,
    controls: Query<(Entity, &Control, &ComputedNode, &UiGlobalTransform)>,
    texts: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
) {
    let mut control_json = String::new();
    let mut focused_control = "null".to_owned();
    for (entity, control, node, transform) in &controls {
        if !control_json.is_empty() {
            control_json.push(',');
        }
        let focused = focus.get() == Some(entity);
        if focused {
            focused_control = json_string(control.id());
        }
        let (position, size) = logical_rect(node, transform);
        let _ = write!(
            control_json,
            "{{\"id\":{},\"label\":{},\"x\":{},\"y\":{},\"width\":{},\"height\":{},\"focused\":{focused}}}",
            json_string(control.id()),
            json_string(control.label()),
            position.x,
            position.y,
            size.x,
            size.y
        );
    }
    let mut text_json = String::new();
    for (text, node, transform) in &texts {
        if !text_json.is_empty() {
            text_json.push(',');
        }
        let (position, size) = logical_rect(node, transform);
        let _ = write!(
            text_json,
            "{{\"text\":{},\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}",
            json_string(&text.0),
            position.x,
            position.y,
            size.x,
            size.y
        );
    }
    let screen = match state.get() {
        Screen::Title => "title",
        Screen::About => "about",
    };
    publish_json(&format!(
        "{{\"ready\":{},\"screen\":\"{screen}\",\"focusedControl\":{focused_control},\"focusVisible\":{},\"viewport\":{{\"width\":{},\"height\":{},\"scaleFactor\":{}}},\"controls\":[{control_json}],\"texts\":[{text_json}]}}",
        readiness.ready,
        visible.0,
        window.width(),
        window.height(),
        window.scale_factor()
    ));
}

fn logical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> (Vec2, Vec2) {
    let size = node.size() * node.inverse_scale_factor();
    let center = transform.translation * node.inverse_scale_factor();
    (center - size * 0.5, size)
}

fn json_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", u32::from(character));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}
