use crate::ecs::{GameState, PickupKind, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        if let Some(entity) = game.shield.take() {
            despawn_recursive_immediate(world, entity);
        }
        return;
    }

    ensure_shield(world, game);
    let Some(entity) = game.shield else {
        return;
    };

    let active = game.effect == Some(PickupKind::Barrier) || game.aegis_timer > 0.0;
    let ship = game.ship_position;
    let elapsed = game.elapsed;
    let scale = if active {
        SHIELD_RADIUS * (1.0 + (elapsed * 6.0).sin() * SHIELD_PULSE)
    } else {
        0.0
    };

    let axis = Vec3::new(0.2, 1.0, 0.05).normalize();
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = ship;
        transform.rotation = nalgebra_glm::quat_angle_axis(elapsed * SHIELD_SPIN, &axis);
        transform.scale = Vec3::new(scale, scale, scale);
    }
    mark_local_transform_dirty(world, entity);
}

fn ensure_shield(world: &mut World, game: &mut GameState) {
    if game.shield.is_some() {
        return;
    }
    let entity = spawn_mesh(world, "Sphere", game.ship_position, Vec3::zeros());
    let material = Material {
        base_color: [0.36, 0.72, 1.0, 0.18],
        emissive_factor: [0.28, 0.58, 0.98],
        emissive_strength: 1.7,
        alpha_mode: AlphaMode::Blend,
        blend_opaque_alpha_threshold: 1.0,
        double_sided: true,
        metallic: 0.0,
        roughness: 0.35,
        ..Default::default()
    };
    register_material(world, entity, format!("shield_{}", entity.id), material);
    game.shield = Some(entity);
}
