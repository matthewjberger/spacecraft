use crate::content::BossKind;
use crate::ecs::{Boss, GameState, PickupKind, Sound, TemplateWorld};
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

    if stats.beam {
        run_boss_beam(world, game, delta);
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
        if let Some(beam_entity) = game.boss_beam
            && let Some(beam) = world.core.get_beam_mut(beam_entity)
        {
            beam.alpha = 0.0;
            beam.width = 0.0;
        }
        award(game, stats.score);
        game.hitstop = HITSTOP_BIG * 2.0;
        game.cam_kick += NOVA_KICK;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_LASER);
        game.sounds.push(Sound::BigExplode);
    }
}

fn run_boss_beam(world: &mut World, game: &mut GameState, delta: f32) {
    ensure_boss_beam(world, game);
    let ship = game.ship_position;
    let protected = game.effect == Some(PickupKind::Barrier) || game.aegis_timer > 0.0;

    let mut origin = Vec3::zeros();
    let mut aim = (0.0, 0.0);
    let mut firing = 0.0;
    let mut arrived = false;
    if let Some(boss) = game.boss.as_mut() {
        arrived = boss.arrived;
        if boss.arrived {
            boss.beam_timer -= delta;
            if boss.beam_timer <= 0.0 {
                boss.beam_timer = BOSS_BEAM_INTERVAL;
                boss.firing = BOSS_BEAM_DURATION + BOSS_BEAM_CHARGE;
                boss.aim_x = ship.x;
                boss.aim_y = ship.y;
            }
            if boss.firing > 0.0 {
                boss.firing -= delta;
            }
        }
        origin = boss.position;
        aim = (boss.aim_x, boss.aim_y);
        firing = boss.firing;
    }

    let charging = firing > BOSS_BEAM_DURATION;
    let hot = firing > 0.0 && !charging;

    if let Some(beam_entity) = game.boss_beam
        && let Some(beam) = world.core.get_beam_mut(beam_entity)
    {
        if firing > 0.0 && arrived {
            beam.start = origin;
            beam.end = Vec3::new(aim.0, aim.1, ship.z + 4.0);
            if charging {
                beam.width = 0.16;
                beam.alpha = 0.55;
                beam.intensity = 2.2;
                beam.color = Vec3::new(2.6, 0.5, 0.4);
            } else {
                beam.width = 1.3;
                beam.alpha = 1.0;
                beam.intensity = 6.5;
                beam.color = Vec3::new(3.8, 0.4, 0.3);
            }
            beam.strands = 10;
            beam.flicker = 0.2;
            beam.flicker_speed = 55.0;
        } else {
            beam.alpha = 0.0;
            beam.width = 0.0;
        }
    }

    if hot
        && !protected
        && game.invuln <= 0.0
        && (ship.x - aim.0).abs() < BOSS_BEAM_RADIUS
        && (ship.y - aim.1).abs() < BOSS_BEAM_RADIUS
    {
        game.shields -= 1;
        game.invuln = DAMAGE_INVULN;
        game.damage_flash = DAMAGE_FLASH_TIME;
        game.shake = DAMAGE_SHAKE;
        game.cam_kick += DAMAGE_KICK;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_DAMAGE);
    }
}

fn ensure_boss_beam(world: &mut World, game: &mut GameState) {
    if game.boss_beam.is_some() {
        return;
    }
    let handle = spawn_vfx(world, VfxPreset::Laser, Vec3::new(0.0, BASE_HEIGHT, -50.0));
    let mut beam_entity = None;
    for entity in handle.entities {
        if beam_entity.is_none() && world.core.get_beam_mut(entity).is_some() {
            beam_entity = Some(entity);
        } else {
            despawn_recursive_immediate(world, entity);
        }
    }
    if let Some(entity) = beam_entity
        && let Some(beam) = world.core.get_beam_mut(entity)
    {
        beam.alpha = 0.0;
        beam.width = 0.0;
    }
    game.boss_beam = beam_entity;
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
    let scaled_health =
        ((stats.health as f32) * (1.0 + difficulty(game) as f32 * 0.4)).round() as i32;
    game.boss = Some(Boss {
        entity,
        kind,
        position,
        health: scaled_health,
        max_health: scaled_health,
        fire_timer: 2.0,
        phase: 0.0,
        spin: 0.0,
        arrived: false,
        beam_timer: BOSS_BEAM_INTERVAL,
        firing: 0.0,
        aim_x: 0.0,
        aim_y: 0.0,
    });
    game.escort_timer = stats.escort_interval.max(2.0);
}
