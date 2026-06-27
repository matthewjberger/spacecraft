use crate::content::SECTORS;
use crate::ecs::{Boss, GameState, TemplateWorld};
use crate::systems::common::*;
use crate::systems::enemies::{spawn_enemy_shot, spawn_fighter};
use crate::systems::enemy_mesh::MONARCH_MESH;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    let sector = &SECTORS[game.sector];
    if !sector.boss {
        return;
    }
    let ship = game.ship_position;
    let elapsed = game.elapsed;

    if game.boss.is_none() && !game.boss_defeated && game.distance >= game.sector_goal {
        spawn_boss(world, game);
    }

    let mut transform_update: Option<(Entity, Vec3, f32)> = None;
    let mut volley_origin: Option<Vec3> = None;
    let mut died = false;
    let mut boss_position = Vec3::zeros();

    if let Some(boss) = game.boss.as_mut() {
        boss.phase += delta;
        boss.spin += delta * 0.5;
        if !boss.arrived {
            boss.position.z += BOSS_APPROACH_SPEED * delta;
            if boss.position.z >= BOSS_HOLD_Z {
                boss.position.z = BOSS_HOLD_Z;
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
                boss.fire_timer = BOSS_FIRE_INTERVAL;
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
            transform.scale = Vec3::new(BOSS_RADIUS, BOSS_RADIUS, BOSS_RADIUS);
        }
        mark_local_transform_dirty(world, entity);
    }

    if let Some(origin) = volley_origin {
        let spread = [
            Vec3::new(-4.5, 0.6, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.5, 0.6, 0.0),
            Vec3::new(-2.2, 2.6, 0.0),
            Vec3::new(2.2, -2.2, 0.0),
        ];
        for offset in spread {
            spawn_enemy_shot(world, game, origin, ship + offset);
        }
    }

    if game.boss.is_some() && game.boss.as_ref().is_some_and(|boss| boss.arrived) {
        game.escort_timer -= delta;
        if game.escort_timer <= 0.0 {
            game.escort_timer = BOSS_ESCORT_INTERVAL;
            spawn_fighter(world, game, sector.enemy_health, sector.enemy_speed);
        }
    }

    if died {
        if let Some(boss) = game.boss.take() {
            for ring in 0..7 {
                let angle = ring as f32 * 1.3;
                let offset = Vec3::new(angle.cos() * 3.0, angle.sin() * 3.0, 0.0);
                let entity =
                    spawn_burst(world, boss_position + offset, Vec3::new(1.0, 0.55, 0.2), 44);
                game.bursts.push((entity, 0.0));
            }
            despawn_recursive_immediate(world, boss.entity);
        }
        game.boss_defeated = true;
        game.score += BOSS_SCORE;
    }
}

fn spawn_boss(world: &mut World, game: &mut GameState) {
    let position = Vec3::new(0.0, BASE_HEIGHT, BOSS_SPAWN_Z);
    let entity = spawn_mesh(
        world,
        MONARCH_MESH,
        position,
        Vec3::new(BOSS_RADIUS, BOSS_RADIUS, BOSS_RADIUS),
    );
    apply_material(
        world,
        entity,
        "monarch",
        [0.1, 0.08, 0.13, 1.0],
        [0.65, 0.05, 0.08],
        false,
        false,
    );
    game.boss = Some(Boss {
        entity,
        core: None,
        position,
        health: BOSS_HEALTH,
        max_health: BOSS_HEALTH,
        fire_timer: 2.0,
        phase: 0.0,
        spin: 0.0,
        arrived: false,
    });
    game.escort_timer = BOSS_ESCORT_INTERVAL;
}
