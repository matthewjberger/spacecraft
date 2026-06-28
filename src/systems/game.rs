use crate::content::{SECTORS, SHOP_ITEMS, STARTING_CREDITS};
use crate::ecs::{GameMode, GameState, ModeKind, ShipMods, Sound, TemplateWorld};
use crate::systems::common::*;
use crate::systems::cutscene;
use crate::systems::shop;
use nightshade::prelude::*;

const DIGIT_KEYS: [KeyCode; 8] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
];

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let advance = read_advance(world);
    let pause = pause_pressed(world);
    let game = &mut game_world.resources.game;
    game.mode_timer += delta;
    if matches!(
        game.mode,
        GameMode::Title | GameMode::Settings | GameMode::LevelSelect
    ) {
        game.menu_orbit += delta * 0.15;
    }
    if game.damage_flash > 0.0 {
        game.damage_flash -= delta;
    }
    if game.shake > 0.0 {
        game.shake = (game.shake - delta * 1.4).max(0.0);
    }
    if game.score_flash > 0.0 {
        game.score_flash -= delta;
    }
    if game.comms_timer > 0.0 {
        game.comms_timer -= delta;
    }
    if game.cam_kick > 0.0 {
        game.cam_kick = approach(game.cam_kick, 0.0, CAMERA_KICK_DECAY * delta);
    }
    if game.cam_fov_pop != 0.0 {
        game.cam_fov_pop = approach(game.cam_fov_pop, 0.0, FOV_POP_DECAY * delta);
    }

    let pre_mode = game.mode;
    let pre_cursor = (game.menu_cursor, game.shop_cursor, game.settings_cursor);
    let confirm_sound = match game.mode {
        GameMode::Shop => buy_pressed(world) || launch_pressed(world),
        GameMode::Title
        | GameMode::Settings
        | GameMode::LevelSelect
        | GameMode::Briefing
        | GameMode::Paused
        | GameMode::SectorClear
        | GameMode::GameOver
        | GameMode::Victory => advance,
        _ => false,
    };
    let back_sound = matches!(game.mode, GameMode::LevelSelect) && pause;

    match game.mode {
        GameMode::Title => {
            if pause {
                world.resources.window.should_exit = true;
            }
            if nav_up(world) {
                game.menu_cursor = game.menu_cursor.saturating_sub(1);
            }
            if nav_down(world) {
                game.menu_cursor = (game.menu_cursor + 1).min(3);
            }
            if advance {
                match game.menu_cursor {
                    0 => start_game(world, game, ModeKind::Story, 0),
                    1 => {
                        game.menu_cursor = 0;
                        enter_mode(game, GameMode::LevelSelect);
                    }
                    2 => start_game(world, game, ModeKind::Endless, 0),
                    _ => {
                        game.settings_cursor = 0;
                        enter_mode(game, GameMode::Settings);
                    }
                }
            }
        }
        GameMode::Settings => {
            if nav_up(world) {
                game.settings_cursor = game.settings_cursor.saturating_sub(1);
            }
            if nav_down(world) {
                game.settings_cursor = (game.settings_cursor + 1).min(5);
            }
            if advance {
                match game.settings_cursor {
                    0 => game.shake_enabled = !game.shake_enabled,
                    1 => game.flash_enabled = !game.flash_enabled,
                    2 => {
                        game.starfield_enabled = !game.starfield_enabled;
                        apply_starfield(world, game);
                    }
                    3 => game.hard_mode = !game.hard_mode,
                    4 => game.crt_enabled = !game.crt_enabled,
                    _ => {
                        game.menu_cursor = 0;
                        enter_mode(game, GameMode::Title);
                    }
                }
            }
        }
        GameMode::LevelSelect => {
            if nav_up(world) {
                game.menu_cursor = game.menu_cursor.saturating_sub(1);
            }
            if nav_down(world) {
                game.menu_cursor = (game.menu_cursor + 1).min(SECTORS.len() - 1);
            }
            if advance {
                start_game(world, game, ModeKind::Arcade, game.menu_cursor);
            }
            if pause {
                game.menu_cursor = 1;
                enter_mode(game, GameMode::Title);
            }
        }
        GameMode::Shop => {
            if nav_up(world) {
                game.shop_cursor = game.shop_cursor.saturating_sub(1);
            }
            if nav_down(world) {
                game.shop_cursor = (game.shop_cursor + 1).min(SHOP_ITEMS.len() - 1);
            }
            if buy_pressed(world) {
                shop::buy(game, game.shop_cursor);
            }
            for (index, key) in DIGIT_KEYS.iter().enumerate().take(SHOP_ITEMS.len()) {
                if world.resources.input.keyboard.just_pressed(*key) {
                    shop::buy(game, index);
                }
            }
            if launch_pressed(world) {
                enter_briefing(world, game, game.sector);
            }
        }
        GameMode::Briefing => {
            if advance {
                enter_mode(game, GameMode::Playing);
            }
        }
        GameMode::Cinematic => {}
        GameMode::Playing => {
            if pause {
                game.menu_cursor = 0;
                enter_mode(game, GameMode::Paused);
            } else if game.shields <= 0 {
                game.best_score = game.best_score.max(game.score);
                enter_mode(game, GameMode::GameOver);
            } else if game.beat_index >= SECTORS[game.sector].beats.len() {
                if game.sector + 1 >= SECTORS.len() {
                    enter_victory(world, game);
                } else {
                    enter_mode(game, GameMode::SectorClear);
                }
            }
        }
        GameMode::Paused => {
            if pause {
                enter_mode(game, GameMode::Playing);
            } else {
                if nav_up(world) {
                    game.menu_cursor = game.menu_cursor.saturating_sub(1);
                }
                if nav_down(world) {
                    game.menu_cursor = (game.menu_cursor + 1).min(1);
                }
                if advance {
                    if game.menu_cursor == 0 {
                        enter_mode(game, GameMode::Playing);
                    } else {
                        to_title(world, game);
                    }
                }
            }
        }
        GameMode::SectorClear => {
            if advance {
                let next = game.sector + 1;
                if next < SECTORS.len() {
                    enter_shop(world, game, next);
                } else {
                    enter_mode(game, GameMode::Victory);
                }
            }
        }
        GameMode::GameOver => {
            if advance {
                to_title(world, game);
            }
        }
        GameMode::Victory => {
            if advance {
                if game.run_mode == ModeKind::Endless {
                    game.loop_count += 1;
                    enter_shop(world, game, 0);
                } else {
                    to_title(world, game);
                }
            }
        }
    }

    if back_sound {
        game.sounds.push(Sound::UiBack);
    } else if confirm_sound {
        game.sounds.push(Sound::UiConfirm);
    }
    if game.mode == pre_mode
        && (game.menu_cursor, game.shop_cursor, game.settings_cursor) != pre_cursor
    {
        game.sounds.push(Sound::UiMove);
    }
    if game.mode == GameMode::Victory && pre_mode != GameMode::Victory {
        game.sounds.push(Sound::Victory);
    }
}

fn enter_mode(game: &mut GameState, mode: GameMode) {
    game.mode = mode;
    game.mode_timer = 0.0;
}

fn apply_starfield(world: &mut World, game: &GameState) {
    if let Some(entity) = game.starfield
        && let Some(emitter) = world.core.get_particle_emitter_mut(entity)
    {
        emitter.enabled = game.starfield_enabled;
        emitter.spawn_rate = if game.starfield_enabled {
            STARFIELD_RATE
        } else {
            0.0
        };
    }
}

fn start_game(world: &mut World, game: &mut GameState, mode: ModeKind, start_sector: usize) {
    game.run_mode = mode;
    game.score = 0;
    game.best_combo = 0;
    game.credits = STARTING_CREDITS + if mode == ModeKind::Arcade { 40 } else { 0 };
    game.mods = ShipMods::default();
    game.loop_count = 0;
    game.max_shields = 4;
    game.shields = 4;
    enter_shop(world, game, start_sector);
}

fn enter_shop(world: &mut World, game: &mut GameState, sector: usize) {
    clear_world(world, game);
    game.sector = sector;
    game.shop_cursor = 0;
    game.ship_position = Vec3::new(0.0, BASE_HEIGHT, 0.0);
    game.speed_scale = 1.0;
    enter_mode(game, GameMode::Shop);
}

fn enter_briefing(world: &mut World, game: &mut GameState, sector: usize) {
    game.sector = sector;
    begin_sector(world, game);
    if game.run_mode == ModeKind::Story {
        game.cinematic_return = GameMode::Playing;
        start_cinematic(
            world,
            game,
            cutscene::sector_cutscene(&SECTORS[sector], game.ship_position),
        );
    } else {
        enter_mode(game, GameMode::Playing);
    }
}

fn enter_victory(world: &mut World, game: &mut GameState) {
    game.best_score = game.best_score.max(game.score);
    if game.run_mode == ModeKind::Story {
        game.cinematic_return = GameMode::Victory;
        start_cinematic(world, game, cutscene::finale_cutscene(game.ship_position));
    } else {
        enter_mode(game, GameMode::Victory);
    }
}

fn start_cinematic(world: &mut World, game: &mut GameState, scene: nightshade::prelude::Cutscene) {
    if let Some(camera) = game.cutscene_camera {
        set_cutscene_camera(world, camera);
        world.resources.active_camera = Some(camera);
    }
    play_cutscene(world, scene);
    enter_mode(game, GameMode::Cinematic);
}

fn begin_sector(world: &mut World, game: &mut GameState) {
    clear_world(world, game);
    game.beat_index = 0;
    game.beat_started = false;
    game.beat_distance = 0.0;
    game.ship_position = Vec3::new(0.0, BASE_HEIGHT, 0.0);
    game.speed_scale = 1.0;
    game.nova_charges = game.mods.nova_max;
    game.aegis_cooldown = 0.0;
    game.aegis_timer = 0.0;
    game.combo = 0;
    game.combo_timer = 0.0;
}

fn to_title(world: &mut World, game: &mut GameState) {
    clear_world(world, game);
    enter_mode(game, GameMode::Title);
    game.sector = 0;
    game.score = 0;
    game.credits = 0;
    game.mods = ShipMods::default();
    game.loop_count = 0;
    game.max_shields = 4;
    game.shields = 4;
    game.beat_index = 0;
    game.beat_started = false;
    game.ship_position = Vec3::new(0.0, BASE_HEIGHT, 0.0);
    game.speed_scale = 1.0;
}

fn nav_up(world: &World) -> bool {
    let keyboard = &world.resources.input.keyboard;
    keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::DPadUp)
}

fn nav_down(world: &World) -> bool {
    let keyboard = &world.resources.input.keyboard;
    keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::DPadDown)
}

fn buy_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::Enter)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::South)
}

fn launch_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::Space)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::Start)
}

fn pause_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::Escape)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::Select)
}

fn clear_world(world: &mut World, game: &mut GameState) {
    for item in game.scenery.drain(..) {
        despawn_recursive_immediate(world, item.entity);
    }
    for enemy in game.enemies.drain(..) {
        despawn_recursive_immediate(world, enemy.entity);
    }
    for shot in game.enemy_shots.drain(..) {
        despawn_recursive_immediate(world, shot.entity);
    }
    for missile in game.missiles.drain(..) {
        despawn_recursive_immediate(world, missile.entity);
    }
    game.missile_timer = 0.0;
    for pickup in game.pickups.drain(..) {
        despawn_recursive_immediate(world, pickup.entity);
        if let Some(terminal) = pickup.terminal {
            despawn_recursive_immediate(world, terminal);
        }
    }
    for projectile in game.projectiles.drain(..) {
        despawn_recursive_immediate(world, projectile.entity);
    }
    for (entity, _) in game.bursts.drain(..) {
        despawn_recursive_immediate(world, entity);
    }
    if let Some(boss) = game.boss.take() {
        despawn_recursive_immediate(world, boss.entity);
    }
    if let Some(shield) = game.shield.take() {
        despawn_recursive_immediate(world, shield);
    }
    for fragment in game.fragments.drain(..) {
        despawn_recursive_immediate(world, fragment.entity);
    }
    game.laser_timer = 0.0;
    game.laser_cooldown = 0.0;
    if let Some(beam) = game.beam
        && let Some(beam_component) = world.core.get_beam_mut(beam)
    {
        beam_component.alpha = 0.0;
        beam_component.width = 0.0;
    }
    if let Some(beam) = game.boss_beam
        && let Some(beam_component) = world.core.get_beam_mut(beam)
    {
        beam_component.alpha = 0.0;
        beam_component.width = 0.0;
    }
    game.effect = None;
    game.effect_timer = 0.0;
    game.nova_charges = 0;
    game.nova_flash = 0.0;
    game.aegis_timer = 0.0;
    game.aegis_cooldown = 0.0;
    game.barrel = Default::default();
    game.invuln = 0.0;
    game.damage_flash = 0.0;
    game.shake = 0.0;
    game.comms_timer = 0.0;
    game.comms_line.clear();
    game.comms_low_warned = false;
    for ally in &mut game.allies {
        ally.phase = 0;
    }
    crate::systems::structures::clear(world, game);
}

fn read_advance(world: &mut World) -> bool {
    let keyboard = &world.resources.input.keyboard;
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        return true;
    }
    let gamepad = &world.resources.input.gamepad;
    gamepad.just_pressed(gilrs::Button::South) || gamepad.just_pressed(gilrs::Button::Start)
}
