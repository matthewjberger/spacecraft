use crate::ecs::{Projectile, SceneryKind, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let firing = read_fire_input(world);
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }

    game.fire_cooldown -= delta;
    if firing && game.fire_cooldown <= 0.0 {
        game.fire_cooldown = FIRE_INTERVAL;
        let side = if game.next_turret == 0 { -1.0 } else { 1.0 };
        game.next_turret ^= 1;
        let origin = Vec3::new(
            game.ship_position.x + side * TURRET_OFFSET_X,
            game.ship_position.y + TURRET_OFFSET_Y,
            game.ship_position.z + TURRET_OFFSET_Z,
        );
        let velocity = Vec3::new(0.0, 0.0, -PROJECTILE_SPEED);
        let entity = spawn_tracer(world, origin);
        game.projectiles.push(Projectile {
            entity,
            position: origin,
            velocity,
            age: 0.0,
        });
    }

    let mut remove: Vec<usize> = Vec::new();
    let mut hits: Vec<usize> = Vec::new();
    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();

    for index in 0..game.projectiles.len() {
        let step = game.projectiles[index].velocity * delta;
        game.projectiles[index].position += step;
        game.projectiles[index].age += delta;
        let position = game.projectiles[index].position;

        if let Some(emitter) = world
            .core
            .get_particle_emitter_mut(game.projectiles[index].entity)
        {
            emitter.position = position;
        }

        if position.z < -PROJECTILE_RANGE || game.projectiles[index].age > 5.0 {
            remove.push(index);
            continue;
        }

        let mut struck = None;
        for scenery_index in 0..game.scenery.len() {
            if game.scenery[scenery_index].kind != SceneryKind::Asteroid {
                continue;
            }
            let separation = (game.scenery[scenery_index].position - position).magnitude();
            if separation < game.scenery[scenery_index].radius + PROJECTILE_HIT_RADIUS {
                struck = Some(scenery_index);
                break;
            }
        }
        if let Some(scenery_index) = struck {
            bursts.push((
                game.scenery[scenery_index].position,
                Vec3::new(1.0, 0.5, 0.2),
                28,
            ));
            hits.push(scenery_index);
            remove.push(index);
        }
    }

    hits.sort_unstable();
    hits.dedup();
    for scenery_index in hits.into_iter().rev() {
        let scenery_item = game.scenery.remove(scenery_index);
        despawn_recursive_immediate(world, scenery_item.entity);
        game.score += 1;
    }

    remove.sort_unstable();
    remove.dedup();
    for index in remove.into_iter().rev() {
        let projectile = game.projectiles.remove(index);
        despawn_recursive_immediate(world, projectile.entity);
    }

    for (position, color, count) in bursts {
        let entity = spawn_burst(world, position, color, count);
        game.bursts.push((entity, 0.0));
    }
}

fn read_fire_input(world: &mut World) -> bool {
    let mut firing = world
        .resources
        .input
        .keyboard
        .is_key_pressed(KeyCode::Space);
    if let Some(gamepad) = query_active_gamepad(world)
        && (gamepad.is_pressed(gilrs::Button::South) || gamepad.is_pressed(gilrs::Button::West))
    {
        firing = true;
    }
    firing
}

fn spawn_tracer(world: &mut World, origin: Vec3) -> Entity {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Point,
        position: origin,
        direction: Vec3::new(0.0, 0.0, -1.0),
        spawn_rate: 260.0,
        burst_count: 0,
        particle_lifetime_min: 0.12,
        particle_lifetime_max: 0.28,
        initial_velocity_min: 1.0,
        initial_velocity_max: 3.0,
        velocity_spread: 0.15,
        gravity: Vec3::zeros(),
        drag: 0.1,
        size_start: 0.16,
        size_end: 0.02,
        color_gradient: tracer_gradient(),
        emissive_strength: 9.0,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    entity
}

fn tracer_gradient() -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 1.0, 0.9, 1.0)),
            (0.3, vec4(0.6, 0.95, 1.0, 1.0)),
            (0.7, vec4(0.2, 0.6, 1.0, 0.7)),
            (1.0, vec4(0.0, 0.2, 0.6, 0.0)),
        ],
    }
}
