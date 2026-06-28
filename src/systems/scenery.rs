use crate::ecs::{GameState, Scenery, SceneryKind, Sound, TemplateWorld};
use crate::systems::asteroid_mesh;
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn spawn_field(world: &mut World, game: &mut GameState, length: f32, count: usize) {
    for _ in 0..count {
        let x = random_range(&mut game.random_state, -ASTEROID_FIELD_X, ASTEROID_FIELD_X);
        let y =
            BASE_HEIGHT + random_range(&mut game.random_state, -ASTEROID_FIELD_Y, ASTEROID_FIELD_Y);
        let z = -COURSE_AHEAD - random_range(&mut game.random_state, 0.0, length);
        let scenery = spawn_asteroid(world, game, Vec3::new(x, y, z), 1.0, 3.6);
        game.scenery.push(scenery);
    }
}

pub fn spawn_belt_rock(world: &mut World, game: &mut GameState) {
    let x = random_range(&mut game.random_state, -ASTEROID_FIELD_X, ASTEROID_FIELD_X);
    let y = BASE_HEIGHT + random_range(&mut game.random_state, -ASTEROID_FIELD_Y, ASTEROID_FIELD_Y);
    let z = -COURSE_AHEAD - random_range(&mut game.random_state, 0.0, 60.0);
    let roll = next_random(&mut game.random_state);
    let (size_min, size_max) = if roll < 0.6 {
        (0.4, 1.2)
    } else if roll < 0.92 {
        (1.4, 3.2)
    } else {
        (3.6, 6.0)
    };
    let scenery = spawn_asteroid(world, game, Vec3::new(x, y, z), size_min, size_max);
    game.scenery.push(scenery);
}

pub fn spawn_rings(world: &mut World, game: &mut GameState, count: usize) {
    let phase = next_random(&mut game.random_state) * std::f32::consts::TAU;
    let amplitude_x = 6.0 + next_random(&mut game.random_state) * 3.0;
    let amplitude_y = 3.4 + next_random(&mut game.random_state) * 1.8;
    for step in 0..count {
        let phase_step = step as f32;
        let x = (phase_step * 0.9 + phase).sin() * amplitude_x;
        let y = (phase_step * 0.6 + phase * 1.3).sin() * amplitude_y;
        let z = -COURSE_AHEAD - phase_step * RING_SPACING;
        let scenery = spawn_ring(world, game, Vec3::new(x, BASE_HEIGHT + y, z));
        game.scenery.push(scenery);
    }
}

fn spawn_ring(world: &mut World, game: &mut GameState, position: Vec3) -> Scenery {
    let entity = spawn_entities(world, NAME, 1)[0];
    let phase = next_random(&mut game.random_state) * std::f32::consts::TAU;
    Scenery {
        entity,
        kind: SceneryKind::Ring,
        position,
        spin_axis: Vec3::new(0.0, 0.0, 1.0),
        spin_speed: 0.0,
        angle: 0.0,
        radius: RING_RADIUS,
        resolved: false,
        collected: false,
        collect_timer: 0.0,
        pulse_phase: phase,
        material_name: String::new(),
    }
}

fn spawn_asteroid(
    world: &mut World,
    game: &mut GameState,
    position: Vec3,
    size_min: f32,
    size_max: f32,
) -> Scenery {
    let variant = ((next_random(&mut game.random_state) * asteroid_mesh::ASTEROID_VARIANTS as f32)
        as usize)
        .min(asteroid_mesh::ASTEROID_VARIANTS - 1);
    let base = random_range(&mut game.random_state, size_min, size_max);
    let scale = Vec3::new(
        base * random_range(&mut game.random_state, 0.8, 1.2),
        base * random_range(&mut game.random_state, 0.8, 1.2),
        base * random_range(&mut game.random_state, 0.8, 1.2),
    );
    let entity = spawn_mesh(
        world,
        &asteroid_mesh::asteroid_name(variant),
        position,
        scale,
    );
    apply_material(
        world,
        entity,
        "rock",
        [0.54, 0.52, 0.58, 1.0],
        [0.05, 0.05, 0.08],
        false,
        true,
    );
    let axis = Vec3::new(
        random_range(&mut game.random_state, -1.0, 1.0),
        random_range(&mut game.random_state, -1.0, 1.0),
        random_range(&mut game.random_state, -1.0, 1.0),
    )
    .normalize();
    Scenery {
        entity,
        kind: SceneryKind::Asteroid,
        position,
        spin_axis: axis,
        spin_speed: random_range(&mut game.random_state, 0.3, 1.4),
        angle: 0.0,
        radius: base,
        resolved: false,
        collected: false,
        collect_timer: 0.0,
        pulse_phase: 0.0,
        material_name: String::new(),
    }
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let speed = RAIL_SPEED * game.speed_scale;
    let ship_position = game.ship_position;

    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();

    for index in 0..game.scenery.len() {
        game.scenery[index].position.z += speed * delta;
        game.scenery[index].angle += game.scenery[index].spin_speed * delta;

        let position = game.scenery[index].position;
        let angle = game.scenery[index].angle;
        let axis = game.scenery[index].spin_axis;
        let kind = game.scenery[index].kind;
        let radius = game.scenery[index].radius;
        let entity = game.scenery[index].entity;

        if kind == SceneryKind::Ring {
            if !game.scenery[index].resolved && position.z >= ship_position.z {
                game.scenery[index].resolved = true;
                let planar = ((position.x - ship_position.x).powi(2)
                    + (position.y - ship_position.y).powi(2))
                .sqrt();
                if planar < radius {
                    award(game, 1);
                    game.ring_boost = RING_BOOST_TIME;
                    game.scenery[index].collected = true;
                    game.sounds.push(Sound::Ring);
                    bursts.push((position, Vec3::new(0.4, 0.9, 1.0), 30));
                }
            }
            if game.scenery[index].collected {
                game.scenery[index].collect_timer += delta;
            }
            continue;
        }

        let rotation = nalgebra_glm::quat_angle_axis(angle, &axis);
        let bend = course_bend(game, position);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position + bend;
            transform.rotation = rotation;
        }
        mark_local_transform_dirty(world, entity);
    }

    let mut passed: Vec<Entity> = Vec::new();
    game.scenery.retain(|scenery| {
        let collected_done = scenery.collected && scenery.collect_timer >= RING_COLLECT_TIME;
        if scenery.position.z > SCENERY_DESPAWN_Z || collected_done {
            passed.push(scenery.entity);
            false
        } else {
            true
        }
    });
    for entity in passed {
        despawn_recursive_immediate(world, entity);
    }

    for (position, color, count) in bursts {
        let entity = spawn_burst(world, position, color, count);
        game.bursts.push((entity, 0.0));
    }

    let mut expired: Vec<usize> = Vec::new();
    for index in 0..game.bursts.len() {
        game.bursts[index].1 += delta;
        if game.bursts[index].1 > BURST_LIFETIME {
            expired.push(index);
        }
    }
    for index in expired.into_iter().rev() {
        let (entity, _) = game.bursts.remove(index);
        despawn_recursive_immediate(world, entity);
    }
}
