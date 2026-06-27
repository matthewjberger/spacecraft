use crate::ecs::{GameState, Scenery, SceneryKind, TemplateWorld};
use crate::systems::asteroid_mesh;
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn populate(world: &mut World, game: &mut GameState) {
    game.frontier_z = COURSE_START_Z;
    ensure_course(world, game);
}

fn ensure_course(world: &mut World, game: &mut GameState) {
    while game.frontier_z > -SCENERY_SPAWN_DISTANCE {
        game.frontier_z = spawn_pattern(world, game);
    }
}

fn spawn_pattern(world: &mut World, game: &mut GameState) -> f32 {
    let start_z = game.frontier_z - PATTERN_GAP;
    let choice = next_random(&mut game.random_state);
    if choice < 0.32 {
        ring_slalom(world, game, start_z)
    } else if choice < 0.8 {
        asteroid_field(world, game, start_z)
    } else if choice < 0.93 {
        asteroid_gauntlet(world, game, start_z)
    } else {
        start_z - 30.0
    }
}

fn ring_slalom(world: &mut World, game: &mut GameState, start_z: f32) -> f32 {
    let count = 3 + (next_random(&mut game.random_state) * 3.0) as usize;
    let phase = next_random(&mut game.random_state) * std::f32::consts::TAU;
    let amplitude_x = 3.4 + next_random(&mut game.random_state) * 1.5;
    let amplitude_y = 1.9 + next_random(&mut game.random_state) * 1.1;
    let mut z = start_z;
    for step in 0..count {
        let phase_step = step as f32;
        let x = (phase_step * 0.9 + phase).sin() * amplitude_x;
        let y = (phase_step * 0.6 + phase * 1.3).sin() * amplitude_y;
        let scenery = spawn_ring(world, game, Vec3::new(x, BASE_HEIGHT + y, z));
        game.scenery.push(scenery);
        z -= RING_SPACING;
    }
    z
}

fn asteroid_field(world: &mut World, game: &mut GameState, start_z: f32) -> f32 {
    let length = 90.0 + next_random(&mut game.random_state) * 80.0;
    let count = 16 + (next_random(&mut game.random_state) * 18.0) as usize;
    for _ in 0..count {
        let x = random_range(&mut game.random_state, -ASTEROID_FIELD_X, ASTEROID_FIELD_X);
        let y =
            BASE_HEIGHT + random_range(&mut game.random_state, -ASTEROID_FIELD_Y, ASTEROID_FIELD_Y);
        let z = start_z - random_range(&mut game.random_state, 0.0, length);
        let scenery = spawn_asteroid(world, game, Vec3::new(x, y, z), 1.0, 3.6);
        game.scenery.push(scenery);
    }
    start_z - length
}

fn asteroid_gauntlet(world: &mut World, game: &mut GameState, start_z: f32) -> f32 {
    let gap_x = random_range(&mut game.random_state, -3.0, 3.0);
    let gap_y = random_range(&mut game.random_state, -1.5, 1.5);
    let count = 9;
    for step in 0..count {
        let angle = (step as f32 / count as f32) * std::f32::consts::TAU
            + next_random(&mut game.random_state) * 0.3;
        let radius = 4.5 + next_random(&mut game.random_state) * 2.0;
        let x = gap_x + angle.cos() * radius;
        let y = BASE_HEIGHT + gap_y + angle.sin() * radius * 0.7;
        let z = start_z - random_range(&mut game.random_state, 0.0, 6.0);
        let scenery = spawn_asteroid(world, game, Vec3::new(x, y, z), 1.0, 2.2);
        game.scenery.push(scenery);
    }
    start_z - 26.0
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
    let elapsed = game.elapsed;
    game.frontier_z += speed * delta;

    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();

    let _ = elapsed;

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
                    game.score += 1;
                    game.scenery[index].collected = true;
                    bursts.push((position, Vec3::new(0.4, 0.9, 1.0), 30));
                }
            }
            if game.scenery[index].collected {
                game.scenery[index].collect_timer += delta;
            }
            continue;
        }

        let rotation = nalgebra_glm::quat_angle_axis(angle, &axis);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
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

    ensure_course(world, game);

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
