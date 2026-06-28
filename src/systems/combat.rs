use crate::ecs::{PickupKind, SceneryKind, Sound, TemplateWorld};
use crate::systems::common::*;
use crate::systems::comms;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    let ship = game.ship_position;
    if game.invuln > 0.0 {
        game.invuln -= delta;
    }
    if game.combo_timer > 0.0 {
        game.combo_timer -= delta;
        if game.combo_timer <= 0.0 {
            game.combo = 0;
        }
    }

    let barrier = game.effect == Some(PickupKind::Barrier) || game.aegis_timer > 0.0;
    let deflecting = game.barrel.timer > 0.0;
    let mut bursts: Vec<(Vec3, Vec3, u32)> = Vec::new();
    let mut damage = false;

    let mut shot_remove: Vec<usize> = Vec::new();
    for index in 0..game.enemy_shots.len() {
        let velocity = game.enemy_shots[index].velocity;
        game.enemy_shots[index].position += velocity * delta;
        game.enemy_shots[index].age += delta;
        let position = game.enemy_shots[index].position;
        if let Some(emitter) = world
            .core
            .get_particle_emitter_mut(game.enemy_shots[index].entity)
        {
            emitter.position = position;
        }
        if deflecting && (position - ship).magnitude() < PLAYER_HIT_RADIUS + 1.6 {
            bursts.push((position, Vec3::new(0.5, 1.0, 1.0), 20));
            shot_remove.push(index);
        } else if (position - ship).magnitude() < PLAYER_HIT_RADIUS + 0.5 {
            damage = true;
            bursts.push((position, Vec3::new(1.0, 0.4, 0.2), 16));
            shot_remove.push(index);
        } else if position.z > ship.z + 6.0 || game.enemy_shots[index].age > 6.0 {
            shot_remove.push(index);
        }
    }
    for index in shot_remove.into_iter().rev() {
        let shot = game.enemy_shots.remove(index);
        despawn_recursive_immediate(world, shot.entity);
    }

    if game.invuln <= 0.0 {
        let mut struck: Option<usize> = None;
        for index in 0..game.scenery.len() {
            if game.scenery[index].kind != SceneryKind::Asteroid {
                continue;
            }
            let asteroid = game.scenery[index].position;
            let radius = game.scenery[index].radius;
            let planar = ((asteroid.x - ship.x).powi(2) + (asteroid.y - ship.y).powi(2)).sqrt();
            if planar < radius + PLAYER_HIT_RADIUS && (asteroid.z - ship.z).abs() < radius + 1.2 {
                struck = Some(index);
                break;
            }
        }
        if let Some(index) = struck {
            let item = game.scenery.remove(index);
            bursts.push((item.position, Vec3::new(1.0, 0.6, 0.3), 32));
            despawn_recursive_immediate(world, item.entity);
            damage = true;
        }
    }

    if game.invuln <= 0.0 {
        let mut struck: Option<usize> = None;
        for index in 0..game.enemies.len() {
            if (game.enemies[index].position - ship).magnitude()
                < game.enemies[index].radius + PLAYER_HIT_RADIUS
            {
                struck = Some(index);
                break;
            }
        }
        if let Some(index) = struck {
            let enemy = game.enemies.remove(index);
            bursts.push((enemy.position, Vec3::new(1.0, 0.5, 0.2), 32));
            despawn_recursive_immediate(world, enemy.entity);
            if let Some(thruster) = enemy.thruster {
                despawn_recursive_immediate(world, thruster);
            }
            damage = true;
        }
    }

    if game.invuln <= 0.0 {
        for structure in &game.structures {
            if structure.extent.x <= 0.0 {
                continue;
            }
            let center = Vec3::new(
                structure.position.x,
                structure.position.y + structure.center_y,
                structure.position.z,
            );
            if (ship.x - center.x).abs() < structure.extent.x + PLAYER_HIT_RADIUS
                && (ship.y - center.y).abs() < structure.extent.y + PLAYER_HIT_RADIUS
                && (ship.z - center.z).abs() < structure.extent.z + 1.0
            {
                bursts.push((
                    Vec3::new(ship.x, ship.y, ship.z - 1.5),
                    Vec3::new(0.7, 0.85, 1.0),
                    36,
                ));
                damage = true;
                break;
            }
        }
    }

    if damage && game.invuln <= 0.0 && !barrier {
        game.shields -= 1;
        game.invuln = DAMAGE_INVULN;
        game.damage_flash = DAMAGE_FLASH_TIME;
        game.shake = DAMAGE_SHAKE;
        game.cam_kick += DAMAGE_KICK;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_DAMAGE);
        game.hitstop = game.hitstop.max(HITSTOP_BIG);
        game.sounds.push(Sound::PlayerHit);
    } else if damage {
        game.damage_flash = game.damage_flash.max(0.12);
        if barrier {
            game.sounds.push(Sound::Shield);
        }
    }

    for (position, color, count) in bursts {
        let entity = spawn_burst(world, position, color, count);
        game.bursts.push((entity, 0.0));
    }

    if game.shields <= 1 && !game.comms_low_warned {
        comms::low_shields(game);
        game.comms_low_warned = true;
    } else if game.shields > 1 {
        game.comms_low_warned = false;
    }
}
