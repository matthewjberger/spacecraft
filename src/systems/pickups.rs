use crate::ecs::{GameState, Pickup, PickupKind, TemplateWorld};
use crate::systems::common::*;
use crate::systems::enemy_mesh::DRONE_MESH;
use nightshade::prelude::*;

pub fn spawn(world: &mut World, game: &mut GameState, kind: PickupKind, position: Vec3) {
    let (mesh, scale) = if kind == PickupKind::Nitrous {
        ("Cylinder", Vec3::new(0.42, 0.85, 0.42))
    } else {
        (DRONE_MESH, Vec3::new(0.9, 0.9, 0.9))
    };
    let entity = spawn_mesh(world, mesh, position, scale);
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
    let terminal = if kind == PickupKind::Nitrous {
        world
            .core
            .set_particle_emitter(entity, nitrous_sparks(position));
        let term = spawn_mesh(world, "Cylinder", position, Vec3::new(0.16, 0.12, 0.16));
        let term_name = format!("battery_term_{}", term.id);
        apply_material(
            world,
            term,
            &term_name,
            [0.82, 0.85, 0.9, 1.0],
            [0.18, 0.2, 0.22],
            false,
            false,
        );
        Some(term)
    } else {
        None
    };
    game.pickups.push(Pickup {
        entity,
        kind,
        position,
        spin: 0.0,
        resolved: false,
        terminal,
    });
}

fn nitrous_sparks(position: Vec3) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.45 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 55.0,
        burst_count: 0,
        particle_lifetime_min: 0.2,
        particle_lifetime_max: 0.5,
        initial_velocity_min: 0.6,
        initial_velocity_max: 2.4,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::zeros(),
        drag: 0.3,
        size_start: 0.09,
        size_end: 0.01,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, vec4(0.6, 1.0, 0.45, 1.0)),
                (0.5, vec4(0.3, 0.9, 0.3, 0.8)),
                (1.0, vec4(0.1, 0.5, 0.1, 0.0)),
            ],
        },
        emissive_strength: 3.2,
        enabled: true,
        ..Default::default()
    }
}

pub fn random_kind(state: &mut u64) -> PickupKind {
    match (next_random(state) * 4.0) as u32 {
        0 => PickupKind::Overdrive,
        1 => PickupKind::Barrier,
        2 => PickupKind::Spread,
        _ => PickupKind::Nitrous,
    }
}

pub fn maybe_drop(world: &mut World, game: &mut GameState, position: Vec3) {
    if next_random(&mut game.random_state) < ASTEROID_DROP_CHANCE {
        let kind = random_kind(&mut game.random_state);
        spawn(world, game, kind, position);
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
    let magnet_range = if game.mods.magnet > 0 {
        MAGNET_BASE_RANGE + game.mods.magnet as f32 * MAGNET_RANGE_PER
    } else {
        0.0
    };

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
        if magnet_range > 0.0 {
            let delta_x = ship.x - game.pickups[index].position.x;
            let delta_y = ship.y - game.pickups[index].position.y;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
            if distance > 0.01 && distance < magnet_range {
                let step = (MAGNET_PULL_SPEED * delta).min(distance);
                game.pickups[index].position.x += delta_x / distance * step;
                game.pickups[index].position.y += delta_y / distance * step;
            }
        }
        let position = game.pickups[index].position;
        let spin = game.pickups[index].spin;
        let entity = game.pickups[index].entity;
        let kind = game.pickups[index].kind;
        let pulse = 0.9 + 0.22 * (elapsed * 4.0 + index as f32).sin();

        let (base_scale, axis) = if kind == PickupKind::Nitrous {
            (Vec3::new(0.42, 0.85, 0.42), Vec3::new(0.0, 1.0, 0.0))
        } else {
            (
                Vec3::new(0.9, 0.9, 0.9),
                Vec3::new(0.3, 1.0, 0.2).normalize(),
            )
        };
        let rotation = nalgebra_glm::quat_angle_axis(spin, &axis);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = rotation;
            transform.scale = base_scale * pulse;
        }
        mark_local_transform_dirty(world, entity);
        if kind == PickupKind::Nitrous {
            if let Some(emitter) = world.core.get_particle_emitter_mut(entity) {
                emitter.position = position;
            }
            if let Some(terminal) = game.pickups[index].terminal {
                let offset =
                    nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.92 * pulse, 0.0));
                if let Some(transform) = world.core.get_local_transform_mut(terminal) {
                    transform.translation = position + offset;
                    transform.rotation = rotation;
                    transform.scale = Vec3::new(0.16, 0.12, 0.16) * pulse;
                }
                mark_local_transform_dirty(world, terminal);
            }
        }

        if !game.pickups[index].resolved {
            if (position - ship).magnitude() < PICKUP_COLLECT_RADIUS {
                game.pickups[index].resolved = true;
                let kind = game.pickups[index].kind;
                let color = kind.color();
                collected = Some(kind);
                bursts.push((position, Vec3::new(color.x, color.y, color.z), 44));
                remove.push(index);
            } else if position.z > ship.z + 5.0 {
                game.pickups[index].resolved = true;
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
        if let Some(terminal) = pickup.terminal {
            despawn_recursive_immediate(world, terminal);
        }
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
