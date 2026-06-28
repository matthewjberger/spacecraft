use crate::content::{Behavior, EnemyKind};
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
    let mut shots: Vec<(Vec3, Vec3)> = Vec::new();

    for index in 0..game.enemies.len() {
        let behavior = game.enemies[index].behavior;
        let closing = game.enemies[index].closing_speed;
        let hold_z = game.enemies[index].hold_z;
        let phase = game.enemies[index].sway_phase;
        let amount = game.enemies[index].sway_amount;
        let lane_x = game.enemies[index].lane_x;
        let lane_y = game.enemies[index].lane_y;
        let previous = game.enemies[index].position;
        let mut position = previous;

        match behavior {
            Behavior::Rusher => {
                position.z += closing * delta;
                let target_x =
                    lane_x + (ship.x - lane_x) * 0.32 + (elapsed * 3.2 + phase).sin() * 0.9;
                let target_y = lane_y + (ship.y - lane_y) * 0.24;
                position.x = approach(position.x, target_x, 3.4 * delta);
                position.y = approach(position.y, target_y, 3.0 * delta);
            }
            Behavior::Strafer => {
                if position.z < hold_z {
                    position.z += closing * delta;
                }
                let target_x = (elapsed * 1.5 + phase).sin() * amount * 3.4 + ship.x * 0.3;
                let target_y = lane_y + (elapsed * 1.1 + phase).cos() * amount * 1.1;
                position.x = approach(position.x, target_x, 2.6 * delta);
                position.y = approach(position.y, target_y, 2.2 * delta);
            }
            Behavior::Turret => {
                if position.z < hold_z {
                    position.z += closing * delta;
                }
                let target_x = lane_x + (elapsed * 0.6 + phase).sin() * 1.4;
                let target_y = lane_y + (elapsed * 0.5 + phase).cos() * 0.8;
                position.x = approach(position.x, target_x, 1.6 * delta);
                position.y = approach(position.y, target_y, 1.4 * delta);
            }
            Behavior::Weaver => {
                position.z += closing * delta;
                let target_x = lane_x
                    + (elapsed * 2.4 + phase).sin() * amount * 1.6
                    + (ship.x - lane_x) * 0.12;
                let target_y = lane_y + (elapsed * 1.8 + phase).cos() * amount * 0.7;
                position.x = approach(position.x, target_x, 3.4 * delta);
                position.y = approach(position.y, target_y, 3.0 * delta);
            }
            Behavior::Diver => {
                if !game.enemies[index].committed {
                    position.z += closing * delta;
                    position.x = approach(position.x, ship.x, 1.3 * delta);
                    position.y = approach(position.y, ship.y, 1.1 * delta);
                    if position.z >= hold_z {
                        game.enemies[index].committed = true;
                        game.enemies[index].lock = ship;
                    }
                } else {
                    position.z += closing * 2.1 * delta;
                    let lock = game.enemies[index].lock;
                    position.x = approach(position.x, lock.x, 5.2 * delta);
                    position.y = approach(position.y, lock.y, 4.6 * delta);
                }
            }
        }

        game.enemies[index].position = position;
        let entity = game.enemies[index].entity;
        let bank = (((position.x - previous.x) / delta.max(0.0001)) * 0.05).clamp(-0.7, 0.7);

        let face = nalgebra_glm::quat_angle_axis(std::f32::consts::PI, &Vec3::new(0.0, 1.0, 0.0));
        let roll = nalgebra_glm::quat_angle_axis(bank, &Vec3::new(0.0, 0.0, 1.0));
        let bend = course_bend(game, position);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position + bend;
            transform.rotation = roll * face;
        }
        mark_local_transform_dirty(world, entity);

        if game.enemies[index].fires {
            game.enemies[index].fire_timer -= delta;
            if position.z > -72.0 && position.z < -5.0 && game.enemies[index].fire_timer <= 0.0 {
                game.enemies[index].fire_timer = game.enemies[index].fire_interval;
                match behavior {
                    Behavior::Turret => {
                        for offset in [-2.4_f32, 0.0, 2.4] {
                            shots.push((position, ship + Vec3::new(offset, 0.0, 0.0)));
                        }
                    }
                    Behavior::Weaver => {
                        let lead = (elapsed * 2.0 + phase).sin() * 3.2;
                        shots.push((position, ship + Vec3::new(lead, 0.0, 0.0)));
                    }
                    _ => shots.push((position, ship)),
                }
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

    for (origin, target) in shots {
        spawn_enemy_shot(world, game, origin, target);
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
    let diff = difficulty(game);
    game.enemies.push(Enemy {
        entity,
        position,
        health: stats.health + diff as i32,
        radius: stats.radius,
        closing_speed: stats.closing_speed + diff as f32 * 1.5,
        fires: stats.fires,
        fire_interval,
        lane_x: position.x,
        lane_y: position.y,
        sway_phase: random_range(&mut game.random_state, 0.0, std::f32::consts::TAU),
        sway_amount: random_range(&mut game.random_state, 1.0, 2.6) * stats.sway,
        fire_timer: random_range(&mut game.random_state, 0.5, fire_interval),
        behavior: stats.behavior,
        hold_z: stats.hold_z,
        lock: Vec3::zeros(),
        committed: false,
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
        emissive_strength: 4.0,
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
