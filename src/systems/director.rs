use crate::content::{Beat, EnemyKind, SECTORS};
use crate::ecs::{GameState, SceneryKind, TemplateWorld};
use crate::systems::common::*;
use crate::systems::{boss, comms, enemies, pickups, scenery, structures};
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let sector_index = game.sector;
    let beats = SECTORS[sector_index].beats;
    if game.beat_index >= beats.len() {
        return;
    }

    if !game.beat_started {
        game.beat_started = true;
        game.beat_distance = 0.0;
        enter_beat(world, game, sector_index, game.beat_index);
        return;
    }

    let advanced = RAIL_SPEED * game.speed_scale * delta;
    game.beat_distance += advanced;

    if let Beat::Belt { density, .. } = &beats[game.beat_index] {
        game.belt_accumulator += advanced * (*density as f32) / 100.0;
        let live_rocks = game
            .scenery
            .iter()
            .filter(|scenery| scenery.kind == SceneryKind::Asteroid)
            .count();
        let mut budget = BELT_MAX_ROCKS.saturating_sub(live_rocks);
        while game.belt_accumulator >= 1.0 && budget > 0 {
            scenery::spawn_belt_rock(world, game);
            game.belt_accumulator -= 1.0;
            budget -= 1;
        }
        game.belt_accumulator = game.belt_accumulator.min(2.0);
    }

    if beat_complete(game, &beats[game.beat_index]) {
        game.beat_index += 1;
        game.beat_started = false;
    }
}

fn enter_beat(world: &mut World, game: &mut GameState, sector_index: usize, beat_index: usize) {
    match &SECTORS[sector_index].beats[beat_index] {
        Beat::Field { length, count } => {
            scenery::spawn_field(world, game, *length, *count);
        }
        Beat::Belt { .. } => {
            game.belt_accumulator = 0.0;
            comms::belt(game);
        }
        Beat::Derelicts { length, count } => {
            let span = length / *count as f32;
            for index in 0..*count {
                let side = if index % 2 == 0 { -1.0 } else { 1.0 };
                let lateral = random_range(&mut game.random_state, 17.0, 30.0);
                let x = side * lateral;
                let y = BASE_HEIGHT + random_range(&mut game.random_state, -9.0, 16.0);
                let z = -COURSE_AHEAD
                    - index as f32 * span
                    - random_range(&mut game.random_state, 0.0, span * 0.6);
                structures::spawn_derelict(world, game, Vec3::new(x, y, z));
            }
        }
        Beat::Rings { count } => scenery::spawn_rings(world, game, *count),
        Beat::Wave { groups } => {
            comms::wave(game);
            let mut kinds: Vec<EnemyKind> = Vec::new();
            for (kind, amount) in groups.iter() {
                for _ in 0..*amount {
                    kinds.push(*kind);
                }
            }
            let total = kinds.len();
            let formation = (next_random(&mut game.random_state) * 4.0) as u32;
            for (index, kind) in kinds.into_iter().enumerate() {
                let offset = formation_offset(formation, index, total);
                let position = Vec3::new(
                    offset.x.clamp(-9.5, 9.5),
                    BASE_HEIGHT + offset.y.clamp(-5.0, 5.0),
                    -ENEMY_SPAWN_AHEAD + offset.z,
                );
                enemies::spawn(world, game, kind, position);
            }
        }
        Beat::MiniBoss(kind) => {
            clear_rings(world, game);
            comms::mini_boss(game);
            boss::spawn(world, game, *kind);
        }
        Beat::Boss(kind) => {
            clear_rings(world, game);
            comms::boss(game);
            boss::spawn(world, game, *kind);
        }
        Beat::Breather { length } => spawn_pickup(world, game, *length),
    }
}

fn formation_offset(formation: u32, index: usize, total: usize) -> Vec3 {
    let spread = if total > 1 {
        index as f32 / (total - 1) as f32 - 0.5
    } else {
        0.0
    };
    match formation {
        0 => Vec3::new(spread * 16.0, 0.0, 0.0),
        1 => Vec3::new(spread * 16.0, 0.0, -spread.abs() * 44.0),
        2 => Vec3::new(spread * 14.0, (0.5 - spread.abs()) * 7.5, 0.0),
        _ => Vec3::new(
            (spread * 6.0).sin() * 8.0,
            spread * 6.0,
            index as f32 * -15.0,
        ),
    }
}

fn clear_rings(world: &mut World, game: &mut GameState) {
    let mut removed: Vec<Entity> = Vec::new();
    game.scenery.retain(|scenery| {
        if scenery.kind == SceneryKind::Ring {
            removed.push(scenery.entity);
            false
        } else {
            true
        }
    });
    for entity in removed {
        despawn_recursive_immediate(world, entity);
    }
}

fn spawn_pickup(world: &mut World, game: &mut GameState, length: f32) {
    let kind = pickups::random_kind(&mut game.random_state);
    let x = random_range(&mut game.random_state, -5.0, 5.0);
    let y = BASE_HEIGHT + random_range(&mut game.random_state, -3.0, 3.0);
    let z = -COURSE_AHEAD - random_range(&mut game.random_state, 30.0, length.max(60.0));
    pickups::spawn(world, game, kind, Vec3::new(x, y, z));
}

fn beat_complete(game: &GameState, beat: &Beat) -> bool {
    match beat {
        Beat::Field { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Beat::Belt { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Beat::Derelicts { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Beat::Rings { count } => game.beat_distance >= *count as f32 * RING_SPACING + PATTERN_GAP,
        Beat::Breather { length } => game.beat_distance >= *length,
        Beat::Wave { .. } => game.enemies.is_empty(),
        Beat::MiniBoss(_) | Beat::Boss(_) => game.boss.is_none(),
    }
}
