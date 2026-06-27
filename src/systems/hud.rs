use crate::ecs::TemplateWorld;
use nightshade::prelude::*;

pub fn build(game_world: &mut TemplateWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    let root = UiTreeBuilder::new(world).finish();

    let panel = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(
                Ab(vec2(30.0, 26.0)),
                Ab(vec2(280.0, 104.0)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 1.5, vec4(0.35, 0.9, 1.0, 0.55))
            .color_raw::<UiBase>(vec4(0.02, 0.07, 0.13, 0.55))
            .flow(FlowDirection::Vertical, 14.0, 10.0)
            .entity()
    };

    let score = {
        let mut tree = UiTreeBuilder::from_parent(world, panel);
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 34.0)))
            .with_text("SCORE  0", 24.0)
            .color_raw::<UiBase>(vec4(0.6, 0.95, 1.0, 1.0))
            .entity()
    };

    let speed = {
        let mut tree = UiTreeBuilder::from_parent(world, panel);
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 22.0)))
            .with_text("THRUST  [::::      ]", 15.0)
            .color_raw::<UiBase>(vec4(0.45, 0.82, 1.0, 0.9))
            .entity()
    };

    ui_mark_render_dirty(world);

    let game = &mut game_world.resources.game;
    game.hud_score = Some(score);
    game.hud_speed = Some(speed);
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let score = game_world.resources.game.score;
    let speed_scale = game_world.resources.game.speed_scale;

    if let Some(entity) = game_world.resources.game.hud_score {
        ui_set_text(world, entity, &format!("SCORE  {score}"));
    }
    if let Some(entity) = game_world.resources.game.hud_speed {
        let filled = (((speed_scale - 0.5) * 8.0).round() as i32).clamp(0, 10) as usize;
        let meter: String = (0..10)
            .map(|index| if index < filled { ':' } else { ' ' })
            .collect();
        ui_set_text(world, entity, &format!("THRUST  [{meter}]"));
    }
    ui_mark_render_dirty(world);
}
