use crate::ecs::{GameState, SceneryKind, Sound, TemplateWorld};
use crate::systems::common::*;
use crate::systems::laser;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }

    if game.aegis_timer > 0.0 {
        game.aegis_timer -= delta;
    }
    if game.aegis_cooldown > 0.0 {
        game.aegis_cooldown -= delta;
    }
    if game.nova_flash > 0.0 {
        game.nova_flash -= delta;
    }

    if nova_pressed(world) && game.mods.nova_max > 0 && game.nova_charges > 0 {
        game.nova_charges -= 1;
        detonate_nova(world, game);
    }

    if aegis_pressed(world) && game.mods.aegis > 0 && game.aegis_cooldown <= 0.0 {
        let level = game.mods.aegis as f32;
        game.aegis_timer = AEGIS_DURATION + level * 0.4;
        game.aegis_cooldown = (AEGIS_COOLDOWN - level * 1.2).max(2.5) + game.aegis_timer;
        let ship = game.ship_position;
        let burst = spawn_burst(world, ship, Vec3::new(0.4, 0.9, 1.0), 36);
        game.bursts.push((burst, 0.0));
        game.sounds.push(Sound::Shield);
    }
}

fn detonate_nova(world: &mut World, game: &mut GameState) {
    let ship = game.ship_position;
    game.sounds.push(Sound::Nova);
    game.nova_flash = NOVA_FLASH_TIME;
    game.shake = DAMAGE_SHAKE;
    game.cam_kick += NOVA_KICK;
    game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_DAMAGE);
    game.hitstop = game.hitstop.max(HITSTOP_BIG);

    for ring in 0..10 {
        let angle = ring as f32 * 0.63;
        let offset = Vec3::new(angle.cos() * 4.0, angle.sin() * 4.0, -2.0);
        let burst = spawn_burst(world, ship + offset, Vec3::new(1.0, 1.0, 0.9), 40);
        game.bursts.push((burst, 0.0));
    }

    let mut killed_enemies: Vec<usize> = Vec::new();
    for index in 0..game.enemies.len() {
        if game.enemies[index].position.z > ship.z - NOVA_RANGE_Z {
            killed_enemies.push(index);
        }
    }
    for index in killed_enemies.into_iter().rev() {
        let enemy = game.enemies.remove(index);
        let burst = spawn_burst(world, enemy.position, Vec3::new(1.0, 0.7, 0.3), 26);
        game.bursts.push((burst, 0.0));
        award(game, ENEMY_SCORE);
        despawn_recursive_immediate(world, enemy.entity);
    }

    let mut sliced: Vec<usize> = Vec::new();
    for index in 0..game.scenery.len() {
        if game.scenery[index].kind != SceneryKind::Asteroid {
            continue;
        }
        let position = game.scenery[index].position;
        let planar = ((position.x - ship.x).powi(2) + (position.y - ship.y).powi(2)).sqrt();
        if planar < NOVA_RADIUS && position.z > ship.z - NOVA_RANGE_Z && position.z < ship.z + 8.0 {
            sliced.push(index);
        }
    }
    for index in sliced.into_iter().rev() {
        let item = game.scenery.remove(index);
        laser::spawn_fragments(world, game, item.position, item.radius);
        award(game, 1);
        despawn_recursive_immediate(world, item.entity);
    }

    if let Some(boss) = game.boss.as_mut() {
        boss.health -= NOVA_BOSS_DAMAGE;
    }

    for shot in game.enemy_shots.drain(..) {
        despawn_recursive_immediate(world, shot.entity);
    }
}

fn nova_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::KeyC)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::East)
}

fn aegis_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::KeyV)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::West)
}
