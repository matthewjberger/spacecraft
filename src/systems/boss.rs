use crate::content::BossKind;
use crate::ecs::{Boss, GameState, PickupKind, Sound, TemplateWorld};
use crate::systems::common::*;
use crate::systems::{comms, enemies};
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
    let mut fire_pattern: Option<(u8, Vec3, f32)> = None;
    let mut phase_changed: Option<u8> = None;
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

        let fraction = boss.health as f32 / boss.max_health.max(1) as f32;
        let next_phase = if fraction > 0.66 {
            0
        } else if fraction > 0.33 {
            1
        } else {
            2
        };
        if next_phase > boss.phase_index {
            boss.phase_index = next_phase;
            boss.firing = 0.0;
            boss.lunge = 0.0;
            boss.pattern_timer = 0.4;
            phase_changed = Some(next_phase);
        }

        if boss.phase_index >= 2 {
            boss.lunge += delta * 1.5;
        }
        let surge = boss.lunge.sin().max(0.0) * 9.0;
        let weave = 6.0 + boss.phase_index as f32 * 1.6;
        boss.position.x = (elapsed * (0.7 + boss.phase_index as f32 * 0.22)).sin() * weave;
        boss.position.y = BASE_HEIGHT + (elapsed * 0.9).cos() * 1.4;
        let render = Vec3::new(boss.position.x, boss.position.y, boss.position.z + surge);
        transform_update = Some((boss.entity, render, boss.spin));
        boss_position = render;

        if boss.arrived {
            boss.pattern_timer -= delta;
            if boss.pattern_timer <= 0.0 {
                let pattern = choose_pattern(boss.phase_index, boss.pattern);
                boss.pattern = boss.pattern.wrapping_add(1);
                boss.pattern_timer = stats.fire_interval * pattern_scale(boss.phase_index);
                boss.spiral_angle += 0.7;
                fire_pattern = Some((pattern, render, boss.spiral_angle));
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

    if let Some((pattern, origin, spiral)) = fire_pattern {
        match pattern {
            0 => aimed_volley(world, game, origin, ship, stats.volley, stats.spread),
            1 => spiral_burst(world, game, origin, ship, stats.volley + 3, spiral),
            _ => radial_ring(world, game, origin, 9 + stats.volley),
        }
    }

    if let Some(phase) = phase_changed {
        comms::boss_phase(game, kind, phase);
        game.cam_kick += FIRE_KICK * 5.0;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_LASER);
        game.shake = game.shake.max(DAMAGE_SHAKE * 0.6);
    }

    let escort_due =
        stats.escort_interval > 0.0 && game.boss.as_ref().is_some_and(|boss| boss.arrived);
    if escort_due {
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

fn choose_pattern(phase: u8, counter: u8) -> u8 {
    match phase {
        0 => 0,
        1 => {
            if counter.is_multiple_of(2) {
                0
            } else {
                1
            }
        }
        _ => match counter % 3 {
            0 => 1,
            1 => 2,
            _ => 0,
        },
    }
}

fn pattern_scale(phase: u8) -> f32 {
    match phase {
        0 => 1.0,
        1 => 0.8,
        _ => 0.62,
    }
}

fn aimed_volley(
    world: &mut World,
    game: &mut GameState,
    origin: Vec3,
    ship: Vec3,
    count: usize,
    spread: f32,
) {
    for shot in 0..count {
        let offset = if count > 1 {
            shot as f32 / (count - 1) as f32 - 0.5
        } else {
            0.0
        };
        let target = ship + Vec3::new(offset * spread * 2.0, ((shot % 2) as f32 - 0.5) * 2.4, 0.0);
        enemies::spawn_enemy_shot(world, game, origin, target);
    }
}

fn spiral_burst(
    world: &mut World,
    game: &mut GameState,
    origin: Vec3,
    ship: Vec3,
    count: usize,
    base: f32,
) {
    let step = std::f32::consts::TAU / count as f32;
    for shot in 0..count {
        let angle = base + shot as f32 * step;
        let target = origin + Vec3::new(angle.cos() * 6.0, angle.sin() * 6.0, ship.z - origin.z);
        enemies::spawn_enemy_shot(world, game, origin, target);
    }
}

fn radial_ring(world: &mut World, game: &mut GameState, origin: Vec3, count: usize) {
    let step = std::f32::consts::TAU / count as f32;
    for shot in 0..count {
        let angle = shot as f32 * step;
        let target = Vec3::new(
            origin.x + angle.cos() * 9.0,
            BASE_HEIGHT + angle.sin() * 9.0,
            0.0,
        );
        enemies::spawn_enemy_shot(world, game, origin, target);
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
                boss.beam_timer = BOSS_BEAM_INTERVAL * (1.0 - boss.phase_index as f32 * 0.24);
                boss.firing = BOSS_BEAM_DURATION + BOSS_BEAM_CHARGE;
                boss.aim_x = ship.x;
                boss.aim_y = ship.y;
            }
            if boss.firing > 0.0 {
                boss.firing -= delta;
                if boss.firing > BOSS_BEAM_DURATION {
                    boss.aim_x = approach(boss.aim_x, ship.x, BOSS_BEAM_TRACK * delta);
                    boss.aim_y = approach(boss.aim_y, ship.y, BOSS_BEAM_TRACK * delta);
                }
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
        ((stats.health as f32) * (1.0 + difficulty(game) as f32 * 0.25)).round() as i32;
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
        phase_index: 0,
        pattern: 0,
        pattern_timer: 2.0,
        spiral_angle: 0.0,
        lunge: 0.0,
    });
    game.escort_timer = stats.escort_interval.max(2.0);
}
