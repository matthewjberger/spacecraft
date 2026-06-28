use crate::ecs::{GameState, Missile, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    let ship = game.ship_position;

    if game.mods.seeker > 0 {
        game.missile_timer -= delta;
        if game.missile_timer <= 0.0 && !game.enemies.is_empty() {
            game.missile_timer = SEEKER_INTERVAL / game.mods.seeker as f32;
            let side = if game.missiles.len().is_multiple_of(2) {
                -1.0
            } else {
                1.0
            };
            let origin = Vec3::new(ship.x + side * 1.6, ship.y - 0.1, ship.z - 1.0);
            spawn_missile(world, game, origin);
        }
    }

    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();
    let mut enemy_hits: Vec<usize> = Vec::new();
    let mut remove: Vec<usize> = Vec::new();

    for index in 0..game.missiles.len() {
        let position = game.missiles[index].position;

        let mut nearest: Option<usize> = None;
        let mut nearest_distance = f32::MAX;
        for enemy_index in 0..game.enemies.len() {
            if game.enemies[enemy_index].position.z > position.z + 2.0 {
                continue;
            }
            let distance = (game.enemies[enemy_index].position - position).magnitude();
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = Some(enemy_index);
            }
        }
        if let Some(enemy_index) = nearest {
            let desired =
                (game.enemies[enemy_index].position - position).normalize() * MISSILE_SPEED;
            let steered =
                approach_vec3(game.missiles[index].velocity, desired, MISSILE_TURN * delta);
            let speed = steered.magnitude().max(0.001);
            game.missiles[index].velocity = steered / speed * MISSILE_SPEED;
        }

        let velocity = game.missiles[index].velocity;
        game.missiles[index].position += velocity * delta;
        game.missiles[index].life -= delta;
        let position = game.missiles[index].position;
        let entity = game.missiles[index].entity;
        if let Some(emitter) = world.core.get_particle_emitter_mut(entity) {
            emitter.position = position;
        }

        let mut hit = false;
        for enemy_index in 0..game.enemies.len() {
            let body = game.enemies[enemy_index].radius;
            if (game.enemies[enemy_index].position - position).magnitude()
                < body + MISSILE_HIT_RADIUS
            {
                enemy_hits.push(enemy_index);
                bursts.push((position, Vec3::new(1.0, 0.6, 0.2), 22));
                hit = true;
                break;
            }
        }
        if !hit
            && let Some(boss) = game.boss.as_mut()
            && (boss.position - position).magnitude()
                < boss.kind.stats().radius + MISSILE_HIT_RADIUS
        {
            boss.health -= MISSILE_DAMAGE;
            bursts.push((position, Vec3::new(1.0, 0.5, 0.2), 18));
            hit = true;
        }

        if hit || game.missiles[index].life <= 0.0 || position.z > SCENERY_DESPAWN_Z {
            remove.push(index);
        }
    }

    for enemy_index in &enemy_hits {
        game.enemies[*enemy_index].health -= MISSILE_DAMAGE;
    }
    let mut dead: Vec<usize> = Vec::new();
    for enemy_index in 0..game.enemies.len() {
        if game.enemies[enemy_index].health <= 0 {
            dead.push(enemy_index);
        }
    }
    for enemy_index in dead.into_iter().rev() {
        let enemy = game.enemies.remove(enemy_index);
        bursts.push((enemy.position, Vec3::new(1.0, 0.5, 0.2), 28));
        award(game, ENEMY_SCORE);
        despawn_recursive_immediate(world, enemy.entity);
        if let Some(thruster) = enemy.thruster {
            despawn_recursive_immediate(world, thruster);
        }
    }

    remove.sort_unstable();
    remove.dedup();
    for index in remove.into_iter().rev() {
        let missile = game.missiles.remove(index);
        despawn_recursive_immediate(world, missile.entity);
    }

    for (position, color, count) in bursts {
        let entity = spawn_burst(world, position, color, count);
        game.bursts.push((entity, 0.0));
    }
}

fn spawn_missile(world: &mut World, game: &mut GameState, origin: Vec3) {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Point,
        position: origin,
        direction: Vec3::new(0.0, 0.0, -1.0),
        spawn_rate: 220.0,
        burst_count: 0,
        particle_lifetime_min: 0.16,
        particle_lifetime_max: 0.34,
        initial_velocity_min: 1.0,
        initial_velocity_max: 2.5,
        velocity_spread: 0.2,
        gravity: Vec3::zeros(),
        drag: 0.1,
        size_start: 0.16,
        size_end: 0.02,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, vec4(1.0, 1.0, 0.85, 1.0)),
                (0.4, vec4(1.0, 0.7, 0.25, 1.0)),
                (1.0, vec4(0.8, 0.25, 0.0, 0.0)),
            ],
        },
        emissive_strength: 4.0,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    game.missiles.push(Missile {
        entity,
        position: origin,
        velocity: Vec3::new(0.0, 0.0, -MISSILE_SPEED),
        life: MISSILE_LIFE,
    });
}
