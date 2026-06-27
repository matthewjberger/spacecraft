use crate::content::EnemyKind;
use crate::ecs::{Enemy, GameState, Projectile, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    let ship = game.ship_position;
    let elapsed = game.elapsed;

    let mut remove: Vec<usize> = Vec::new();
    let mut shots: Vec<Vec3> = Vec::new();

    for index in 0..game.enemies.len() {
        game.enemies[index].position.z += game.enemies[index].closing_speed * delta;
        game.enemies[index].spin += delta * 2.6;

        let phase = game.enemies[index].sway_phase;
        let amount = game.enemies[index].sway_amount;
        let lane_x = game.enemies[index].lane_x;
        let lane_y = game.enemies[index].lane_y;
        let target_x = lane_x + (elapsed * 1.7 + phase).sin() * amount + (ship.x - lane_x) * 0.16;
        let target_y =
            lane_y + (elapsed * 1.3 + phase).cos() * amount * 0.5 + (ship.y - lane_y) * 0.12;
        game.enemies[index].position.x =
            approach(game.enemies[index].position.x, target_x, 3.0 * delta);
        game.enemies[index].position.y =
            approach(game.enemies[index].position.y, target_y, 3.0 * delta);

        let position = game.enemies[index].position;
        let spin = game.enemies[index].spin;
        let entity = game.enemies[index].entity;

        let face = nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(0.0, 1.0, 0.0));
        let roll = nalgebra_glm::quat_angle_axis(spin, &Vec3::new(0.0, 0.0, 1.0));
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = roll * face;
        }
        mark_local_transform_dirty(world, entity);

        if game.enemies[index].fires {
            game.enemies[index].fire_timer -= delta;
            if position.z > -60.0 && position.z < -6.0 && game.enemies[index].fire_timer <= 0.0 {
                game.enemies[index].fire_timer = game.enemies[index].fire_interval;
                shots.push(position);
            }
        }

        if position.z > ENEMY_DESPAWN_Z {
            remove.push(index);
        }
    }

    for index in remove.into_iter().rev() {
        let enemy = game.enemies.remove(index);
        despawn_recursive_immediate(world, enemy.entity);
    }

    for origin in shots {
        spawn_enemy_shot(world, game, origin, ship);
    }
}

pub fn spawn(world: &mut World, game: &mut GameState, kind: EnemyKind, position: Vec3) {
    let stats = kind.stats();
    let entity = spawn_mesh(
        world,
        stats.mesh,
        position,
        Vec3::new(stats.scale, stats.scale, stats.scale),
    );
    apply_material(
        world,
        entity,
        "drift",
        stats.base_color,
        stats.emissive,
        false,
        false,
    );
    let fire_interval = if stats.fire_interval > 0.0 {
        stats.fire_interval
    } else {
        1.5
    };
    game.enemies.push(Enemy {
        entity,
        position,
        health: stats.health + game.loop_count as i32,
        radius: stats.radius,
        closing_speed: stats.closing_speed + game.loop_count as f32 * 1.5,
        fires: stats.fires,
        fire_interval,
        lane_x: position.x,
        lane_y: position.y,
        sway_phase: random_range(&mut game.random_state, 0.0, std::f32::consts::TAU),
        sway_amount: random_range(&mut game.random_state, 1.0, 2.6) * stats.sway,
        fire_timer: random_range(&mut game.random_state, 0.5, fire_interval),
        spin: 0.0,
    });
}

pub fn spawn_enemy_shot(world: &mut World, game: &mut GameState, origin: Vec3, target: Vec3) {
    let direction = (target - origin).normalize();
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Point,
        position: origin,
        direction,
        spawn_rate: 240.0,
        burst_count: 0,
        particle_lifetime_min: 0.12,
        particle_lifetime_max: 0.3,
        initial_velocity_min: 1.0,
        initial_velocity_max: 2.5,
        velocity_spread: 0.16,
        gravity: Vec3::zeros(),
        drag: 0.1,
        size_start: 0.18,
        size_end: 0.02,
        color_gradient: enemy_shot_gradient(),
        emissive_strength: 8.0,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    game.enemy_shots.push(Projectile {
        entity,
        position: origin,
        velocity: direction * ENEMY_SHOT_SPEED,
        age: 0.0,
    });
}

fn enemy_shot_gradient() -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 0.95, 0.7, 1.0)),
            (0.3, vec4(1.0, 0.45, 0.2, 1.0)),
            (0.7, vec4(1.0, 0.2, 0.15, 0.7)),
            (1.0, vec4(0.5, 0.0, 0.0, 0.0)),
        ],
    }
}
