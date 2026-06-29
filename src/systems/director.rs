use crate::ecs::{GameState, SceneryKind, TemplateWorld};
use crate::level::{Segment, select_next};
use crate::systems::common::*;
use crate::systems::{boss, comms, enemies, pickups, scenery, structures};
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    update_course_curve(game, delta);
    if game.level_done || game.current_node >= game.level.nodes.len() {
        return;
    }
    let node = game.current_node;

    if !game.beat_started {
        game.beat_started = true;
        game.beat_distance = 0.0;
        let segment = game.level.nodes[node].segment.clone();
        enter_segment(world, game, &segment);
        return;
    }

    let advanced = RAIL_SPEED * game.speed_scale * delta;
    game.beat_distance += advanced;

    if let Segment::Belt { density, .. } = &game.level.nodes[node].segment {
        let density = *density;
        game.belt_accumulator += advanced * (density as f32) / 100.0;
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

    let segment = game.level.nodes[node].segment.clone();
    if segment_complete(game, &segment) {
        if matches!(segment, Segment::Wave { .. }) {
            comms::wave_clear(game);
        }
        advance_node(game);
    }
}

fn advance_node(game: &mut GameState) {
    let edges = game.level.nodes[game.current_node].edges.clone();
    match select_next(&edges, game.combo, game.shields, &mut game.random_state) {
        Some(target) => {
            game.current_node = target;
            game.beat_started = false;
        }
        None => game.level_done = true,
    }
}

fn update_course_curve(game: &mut GameState, delta: f32) {
    game.curve_timer -= delta;
    if game.curve_timer <= 0.0 {
        game.curve_timer = random_range(
            &mut game.random_state,
            COURSE_CURVE_MIN_HOLD,
            COURSE_CURVE_MAX_HOLD,
        );
        let roll = next_random(&mut game.random_state);
        if roll < 0.2 {
            game.curve_target_x = 0.0;
        } else if roll < 0.58 {
            game.curve_target_x = random_range(
                &mut game.random_state,
                -COURSE_CURVE_MAX_X * 0.45,
                COURSE_CURVE_MAX_X * 0.45,
            );
        } else {
            let sign_x = if next_random(&mut game.random_state) < 0.5 {
                -1.0
            } else {
                1.0
            };
            game.curve_target_x = sign_x
                * random_range(
                    &mut game.random_state,
                    COURSE_CURVE_MAX_X * 0.72,
                    COURSE_CURVE_MAX_X,
                );
        }
    }
    game.curve_x = approach(
        game.curve_x,
        game.curve_target_x,
        COURSE_CURVE_RESPONSE * delta,
    );
}

fn enter_segment(world: &mut World, game: &mut GameState, segment: &Segment) {
    match segment {
        Segment::Field { length, count } => {
            scenery::spawn_field(world, game, *length, *count);
        }
        Segment::Belt { .. } => {
            game.belt_accumulator = 0.0;
            comms::belt(game);
        }
        Segment::Derelicts { length, count } => {
            let span = length / *count as f32;
            for index in 0..*count {
                let side = if index % 2 == 0 { -1.0 } else { 1.0 };
                let lateral = random_range(&mut game.random_state, 14.0, 26.0);
                let x = side * lateral;
                let z = -COURSE_AHEAD
                    - index as f32 * span
                    - random_range(&mut game.random_state, 0.0, span * 0.6);
                if next_random(&mut game.random_state) < 0.72 {
                    structures::spawn_building(
                        world,
                        game,
                        Vec3::new(x, structures::BUILDING_BASE, z),
                    );
                } else {
                    let y = BASE_HEIGHT + random_range(&mut game.random_state, -3.0, 13.0);
                    structures::spawn_derelict(world, game, Vec3::new(x, y, z));
                }
            }
        }
        Segment::Rings { count } => scenery::spawn_rings(world, game, *count),
        Segment::Wave { groups } => {
            comms::wave(game);
            let mut kinds: Vec<crate::content::EnemyKind> = Vec::new();
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
        Segment::MiniBoss(kind) => {
            clear_rings(world, game);
            comms::mini_boss(game);
            boss::spawn(world, game, *kind);
        }
        Segment::Boss(kind) => {
            clear_rings(world, game);
            comms::boss(game);
            boss::spawn(world, game, *kind);
        }
        Segment::Breather { length } => spawn_pickup(world, game, *length),
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

fn segment_complete(game: &GameState, segment: &Segment) -> bool {
    match segment {
        Segment::Field { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Segment::Belt { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Segment::Derelicts { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Segment::Rings { count } => {
            game.beat_distance >= *count as f32 * RING_SPACING + PATTERN_GAP
        }
        Segment::Breather { length } => game.beat_distance >= *length,
        Segment::Wave { .. } => game.enemies.is_empty(),
        Segment::MiniBoss(_) | Segment::Boss(_) => game.boss.is_none(),
    }
}
