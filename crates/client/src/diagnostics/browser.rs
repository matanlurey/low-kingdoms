use bevy::{
    ecs::system::SystemParam,
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

type ControlNodes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Control,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        Option<&'static CalculatedClip>,
        Option<&'static crate::gameplay::presentation::DisabledControl>,
    ),
>;

#[derive(SystemParam)]
pub(super) struct SnapshotData<'w, 's> {
    state: Res<'w, State<Screen>>,
    readiness: Res<'w, PresentationReady>,
    focus: Res<'w, InputFocus>,
    visible: Res<'w, InputFocusVisible>,
    window: Single<'w, 's, &'static Window>,
    battle: Res<'w, crate::gameplay::Battle>,
    parents: Query<'w, 's, &'static ChildOf>,
    overlays: Query<
        'w,
        's,
        (
            Entity,
            &'static crate::gameplay::presentation::OverlayRoot,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    controls: ControlNodes<'w, 's>,
    texts: Query<
        'w,
        's,
        (
            &'static Text,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    panes: Query<
        'w,
        's,
        (
            Entity,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static ScrollPosition,
            &'static crate::gameplay::presentation::ScrollPane,
        ),
    >,
}

pub(super) fn publish_snapshot(data: SnapshotData) {
    let SnapshotData {
        state,
        readiness,
        focus,
        visible,
        window,
        battle,
        controls,
        parents,
        overlays,
        texts,
        panes,
    } = data;
    let mut focused_control = None;
    let control_json: Vec<_> = controls.iter().map(|(entity, control, node, transform, clip, disabled)| {
        let focused = focus.get() == Some(entity);
        if focused { focused_control = Some(control.id()); }
        let (position, size) = logical_rect(node, transform);
        let rectangle = Rect::from_corners(position, position + size);
        let clipped = clip.map_or(rectangle, |clip| rectangle.intersect(Rect::from_corners(clip.clip.min * node.inverse_scale_factor(), clip.clip.max * node.inverse_scale_factor())));
        let clipped = active_rectangle(entity, &parents, &overlays, clipped);
        serde_json::json!({ "id": control.id(), "label": control.label(), "x":position.x,"y":position.y,"width":size.x,"height":size.y,"focused":focused,"enabled":disabled.is_none(),"visible":clipped.width()>0.0 && clipped.height()>0.0,"visibleRect":{"x":clipped.min.x,"y":clipped.min.y,"width":clipped.width().max(0.0),"height":clipped.height().max(0.0)} })
    }).collect();
    let text_json: Vec<_> = texts.iter().map(|(text,node,transform)| {
        let (position,size)=logical_rect(node,transform);
        serde_json::json!({"text":text.0,"x":position.x,"y":position.y,"width":size.x,"height":size.y})
    }).collect();
    let scroll_json: Vec<_> = panes.iter().map(|(entity,node,transform,scroll,pane)| {
        let (position,size)=logical_rect(node,transform);
        let rectangle = active_rectangle(entity, &parents, &overlays, Rect::from_corners(position, position + size));
        let visible = rectangle.width() > 0.0 && rectangle.height() > 0.0;
        serde_json::json!({"x":position.x,"y":position.y,"width":size.x,"height":size.y,"offsetX":scroll.x,"offsetY":scroll.y,"horizontal":pane.horizontal,"visible":visible})
    }).collect();
    let screen = match state.get() {
        Screen::Title => "title",
        Screen::About => "about",
        Screen::Mulligan => "mulligan",
        Screen::Battle => "battle",
        Screen::Results => "results",
    };
    let payload = serde_json::json!({"ready":readiness.ready,"screen":screen,"focusedControl":focused_control,"focusVisible":visible.0,"viewport":{"width":window.width(),"height":window.height(),"scaleFactor":window.scale_factor()},"controls":control_json,"texts":text_json,"battle":{"view":battle.view,"legal":battle.legal,"catalog":battle.catalog,"history":battle.history,"selected":battle.selected,"attacker":battle.attacker,"resources":battle.resource_selection,"panel":format!("{:?}",battle.panel),"speed":format!("{:?}",battle.speed),"revision":battle.revision,"resumeAvailable":battle.resume_available,"error":battle.error,"scrollRegions":scroll_json}});
    publish_json(&payload.to_string());
}

fn logical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> (Vec2, Vec2) {
    let size = node.size() * node.inverse_scale_factor();
    let center = transform.translation * node.inverse_scale_factor();
    (center - size * 0.5, size)
}

fn active_rectangle(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    overlays: &Query<(
        Entity,
        &crate::gameplay::presentation::OverlayRoot,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut rectangle: Rect,
) -> Rect {
    loop {
        if overlays.get(entity).is_ok() {
            return rectangle;
        }
        let Ok(parent) = parents.get(entity) else {
            break;
        };
        entity = parent.parent();
    }
    for (_, overlay, node, transform) in overlays {
        if overlay.modal {
            return Rect::from_corners(rectangle.min, rectangle.min);
        }
        let (position, size) = logical_rect(node, transform);
        let blocked = Rect::from_corners(position, position + size);
        let intersection = rectangle.intersect(blocked);
        if intersection.width() > 0.0 && intersection.height() > 0.0 {
            let above = Rect::from_corners(
                rectangle.min,
                Vec2::new(rectangle.max.x, intersection.min.y),
            );
            let below = Rect::from_corners(
                Vec2::new(rectangle.min.x, intersection.max.y),
                rectangle.max,
            );
            rectangle = if above.height() > below.height() {
                above
            } else {
                below
            };
        }
    }
    rectangle
}
