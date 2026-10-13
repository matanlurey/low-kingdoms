use bevy::{
    ecs::system::RunSystemOnce,
    prelude::*,
    text::{ComputedTextBlock, FontCx},
    ui::{LayoutContext, ui_surface::UiSurface},
};
use low_kingdoms_battle::content::{CardDefinition, Content};
use low_kingdoms_rules::engine::Game;

use super::{Battle, CardFace, Presentation, Screen, Zone, fixture, project};

fn mulligan(window: Vec2) -> App {
    let (mut app, _) = fixture();
    let content = Content::load().expect("embedded content");
    let definitions = content
        .cards()
        .iter()
        .map(CardDefinition::unit_definition)
        .collect();
    let (game, _) = Game::new(content.config(42), definitions).expect("opening hand");
    project(&mut app, &game);
    app.world_mut().resource_mut::<Battle>().catalog = content.cards().to_vec();
    app.world_mut()
        .query::<&mut Window>()
        .single_mut(app.world_mut())
        .expect("window")
        .resolution
        .set(window.x, window.y);
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Mulligan);
    app.update();
    app.update();
    app
}

/// Computes the renderer's actual pane/card styles in the available hand slot.
/// Card faces have explicit dimensions, so their text does not need measurement.
fn card_bounds(app: &mut App, slot: UVec2, dpr: u32) -> Vec<Rect> {
    let world = app.world_mut();
    let pane = world.resource::<Presentation>().zones[&Zone::Hand];
    let cards = world
        .get::<Children>(pane)
        .expect("hand children")
        .iter()
        .filter(|entity| world.get::<CardFace>(*entity).is_some())
        .collect::<Vec<_>>();
    assert_eq!(cards.len(), 6);
    let root = world.spawn_empty().id();
    let physical = slot * UVec2::splat(dpr);
    let scale_factor = if dpr == 2 { 2.0 } else { 1.0 };
    let context = LayoutContext {
        scale_factor,
        physical_size: physical.as_vec2(),
    };
    let mut surface = UiSurface::default();
    surface.upsert_node(
        &context,
        root,
        &Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    for entity in std::iter::once(pane).chain(cards.iter().copied()) {
        surface.upsert_node(
            &context,
            entity,
            world.get::<Node>(entity).expect("node"),
            None,
        );
    }
    surface.update_children(root, std::iter::once(pane));
    surface.update_children(pane, cards.iter().copied());
    world.insert_resource(surface);
    world.init_resource::<FontCx>();
    world
        .run_system_once(
            move |mut surface: ResMut<UiSurface>,
                  mut fonts: ResMut<FontCx>,
                  mut buffers: Query<&mut ComputedTextBlock>| {
                surface.compute_layout(root, physical, &mut buffers, &mut fonts);
            },
        )
        .expect("layout system");
    let mut surface = world.resource_mut::<UiSurface>();
    cards
        .into_iter()
        .map(|entity| {
            let (layout, _) = surface.get_layout(entity, true).expect("card layout");
            let min = Vec2::new(layout.location.x, layout.location.y) / scale_factor;
            let size = Vec2::new(layout.size.width, layout.size.height) / scale_factor;
            Rect::from_corners(min, min + size)
        })
        .collect()
}

#[test]
fn overflowing_mulligan_cards_start_inside_both_scroll_axes_at_each_pixel_ratio() {
    for dpr in [1, 2] {
        for (window, slot) in [
            (Vec2::new(667.0, 375.0), UVec2::new(651, 170)),
            (Vec2::new(667.0, 375.0), UVec2::new(651, 60)),
            (Vec2::new(375.0, 667.0), UVec2::new(359, 170)),
        ] {
            let mut app = mulligan(window);
            let bounds = card_bounds(&mut app, slot, dpr);
            for card in &bounds {
                assert!(card.min.x >= 0.0 && card.min.y >= 0.0, "{card:?}");
            }
            let first = bounds.first().expect("first card");
            assert!(first.max.x <= slot.as_vec2().x);
            assert!(first.min.y <= f32::EPSILON);
            if slot.y >= 110 {
                let last = bounds.last().expect("last card");
                let maximum_scroll = (last.max - slot.as_vec2()).max(Vec2::ZERO);
                let final_card =
                    Rect::from_corners(last.min - maximum_scroll, last.max - maximum_scroll);
                assert!(final_card.min.x >= 0.0 && final_card.min.y >= 0.0);
                assert!(final_card.max.cmple(slot.as_vec2()).all());
            }
        }
    }
}

#[test]
fn fitting_landscape_mulligan_hand_keeps_equal_outer_spacing_at_each_pixel_ratio() {
    for dpr in [1, 2] {
        let mut app = mulligan(Vec2::new(844.0, 390.0));
        let bounds = card_bounds(&mut app, UVec2::new(828, 170), dpr);
        let leading = bounds.first().expect("first card").min.x;
        let trailing = 828.0 - bounds.last().expect("last card").max.x;
        assert!(leading > 0.0);
        assert!((leading - trailing).abs() <= f32::EPSILON);
    }
}
