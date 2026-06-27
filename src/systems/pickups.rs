use crate::ecs::{GameState, Pickup, PickupKind, TemplateWorld};
use crate::systems::common::*;
use crate::systems::enemy_mesh::DRONE_MESH;
use nightshade::prelude::*;

pub fn spawn(world: &mut World, game: &mut GameState, kind: PickupKind, position: Vec3) {
    let entity = spawn_mesh(world, DRONE_MESH, position, Vec3::new(0.9, 0.9, 0.9));
    let color = kind.color();
    apply_material(
        world,
        entity,
        "pickup",
        [color.x, color.y, color.z, 1.0],
        kind.emissive(),
        true,
        false,
    );
    game.pickups.push(Pickup {
        entity,
        kind,
        position,
        spin: 0.0,
        resolved: false,
    });
}

pub fn random_kind(state: &mut u64) -> PickupKind {
    match (next_random(state) * 3.0) as u32 {
        0 => PickupKind::Overdrive,
        1 => PickupKind::Barrier,
        _ => PickupKind::Spread,
    }
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    let speed = RAIL_SPEED * game.speed_scale;
    let ship = game.ship_position;
    let elapsed = game.elapsed;

    if game.effect.is_some() {
        game.effect_timer -= delta;
        if game.effect_timer <= 0.0 {
            game.effect = None;
            game.effect_timer = 0.0;
        }
    }

    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();
    let mut collected: Option<PickupKind> = None;
    let mut remove: Vec<usize> = Vec::new();

    for index in 0..game.pickups.len() {
        game.pickups[index].position.z += speed * delta;
        game.pickups[index].spin += delta * 2.2;
        let position = game.pickups[index].position;
        let spin = game.pickups[index].spin;
        let entity = game.pickups[index].entity;
        let pulse = 0.9 + 0.22 * (elapsed * 4.0 + index as f32).sin();

        let axis = Vec3::new(0.3, 1.0, 0.2).normalize();
        let rotation = nalgebra_glm::quat_angle_axis(spin, &axis);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = rotation;
            transform.scale = Vec3::new(0.9 * pulse, 0.9 * pulse, 0.9 * pulse);
        }
        mark_local_transform_dirty(world, entity);

        if !game.pickups[index].resolved && position.z >= ship.z {
            game.pickups[index].resolved = true;
            let planar = ((position.x - ship.x).powi(2) + (position.y - ship.y).powi(2)).sqrt();
            if planar < PICKUP_COLLECT_RADIUS {
                let kind = game.pickups[index].kind;
                let color = kind.color();
                collected = Some(kind);
                bursts.push((position, Vec3::new(color.x, color.y, color.z), 44));
                remove.push(index);
            }
        }
        if position.z > SCENERY_DESPAWN_Z {
            remove.push(index);
        }
    }

    remove.sort_unstable();
    remove.dedup();
    for index in remove.into_iter().rev() {
        let pickup = game.pickups.remove(index);
        despawn_recursive_immediate(world, pickup.entity);
    }

    if let Some(kind) = collected {
        game.effect = Some(kind);
        game.effect_duration = kind.duration();
        game.effect_timer = kind.duration();
    }

    for (position, color, count) in bursts {
        let entity = spawn_burst(world, position, color, count);
        game.bursts.push((entity, 0.0));
    }
}
