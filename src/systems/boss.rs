use crate::content::BossKind;
use crate::ecs::{Boss, GameState, TemplateWorld};
use crate::systems::common::*;
use crate::systems::enemies;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let Some(kind) = game.boss.as_ref().map(|boss| boss.kind) else {
        return;
    };
    let stats = kind.stats();
    let ship = game.ship_position;
    let elapsed = game.elapsed;

    let mut transform_update: Option<(Entity, Vec3, f32)> = None;
    let mut volley_origin: Option<Vec3> = None;
    let mut died = false;
    let mut boss_position = Vec3::zeros();

    if let Some(boss) = game.boss.as_mut() {
        boss.phase += delta;
        boss.spin += delta * 0.5;
        if !boss.arrived {
            boss.position.z += stats.approach_speed * delta;
            if boss.position.z >= stats.hold_z {
                boss.position.z = stats.hold_z;
                boss.arrived = true;
            }
        }
        boss.position.x = (elapsed * 0.7).sin() * 6.0;
        boss.position.y = BASE_HEIGHT + (elapsed * 0.9).cos() * 1.4;
        transform_update = Some((boss.entity, boss.position, boss.spin));
        boss_position = boss.position;
        if boss.arrived {
            boss.fire_timer -= delta;
            if boss.fire_timer <= 0.0 {
                boss.fire_timer = stats.fire_interval;
                volley_origin = Some(boss.position);
            }
        }
        if boss.health <= 0 {
            died = true;
        }
    }

    if let Some((entity, position, spin)) = transform_update {
        let yaw = nalgebra_glm::quat_angle_axis(spin, &Vec3::new(0.0, 1.0, 0.0));
        let tilt = nalgebra_glm::quat_angle_axis(0.28, &Vec3::new(1.0, 0.0, 0.0));
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = yaw * tilt;
            transform.scale = Vec3::new(stats.scale, stats.scale, stats.scale);
        }
        mark_local_transform_dirty(world, entity);
    }

    if let Some(origin) = volley_origin {
        for shot in 0..stats.volley {
            let offset = if stats.volley > 1 {
                shot as f32 / (stats.volley - 1) as f32 - 0.5
            } else {
                0.0
            };
            let target = ship
                + Vec3::new(
                    offset * stats.spread * 2.0,
                    ((shot % 2) as f32 - 0.5) * 2.4,
                    0.0,
                );
            enemies::spawn_enemy_shot(world, game, origin, target);
        }
    }

    if stats.escort_interval > 0.0 && game.boss.as_ref().is_some_and(|boss| boss.arrived) {
        game.escort_timer -= delta;
        if game.escort_timer <= 0.0 {
            game.escort_timer = stats.escort_interval;
            let lane_x = random_range(&mut game.random_state, -5.0, 5.0);
            let lane_y = BASE_HEIGHT + random_range(&mut game.random_state, -2.4, 2.4);
            let position = Vec3::new(lane_x, lane_y, ship.z - ENEMY_SPAWN_AHEAD);
            enemies::spawn(world, game, stats.escort, position);
        }
    }

    if died {
        if let Some(boss) = game.boss.take() {
            for ring in 0..8 {
                let angle = ring as f32 * 1.2;
                let offset = Vec3::new(angle.cos() * 3.2, angle.sin() * 3.2, 0.0);
                let entity =
                    spawn_burst(world, boss_position + offset, Vec3::new(1.0, 0.55, 0.2), 46);
                game.bursts.push((entity, 0.0));
            }
            despawn_recursive_immediate(world, boss.entity);
        }
        game.score += stats.score;
        game.credits += stats.score;
    }
}

pub fn spawn(world: &mut World, game: &mut GameState, kind: BossKind) {
    let stats = kind.stats();
    let position = Vec3::new(0.0, BASE_HEIGHT, BOSS_SPAWN_Z);
    let entity = spawn_mesh(
        world,
        stats.mesh,
        position,
        Vec3::new(stats.scale, stats.scale, stats.scale),
    );
    apply_material(
        world,
        entity,
        "boss",
        stats.base_color,
        stats.emissive,
        false,
        false,
    );
    game.boss = Some(Boss {
        entity,
        kind,
        position,
        health: stats.health,
        max_health: stats.health,
        fire_timer: 2.0,
        phase: 0.0,
        spin: 0.0,
        arrived: false,
    });
    game.escort_timer = stats.escort_interval.max(2.0);
}
