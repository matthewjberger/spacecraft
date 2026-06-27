use crate::content::{Beat, SECTORS};
use crate::ecs::{GameState, TemplateWorld};
use crate::systems::common::*;
use crate::systems::{boss, enemies, scenery};
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

    game.beat_distance += RAIL_SPEED * game.speed_scale * delta;

    if beat_complete(game, &beats[game.beat_index]) {
        game.beat_index += 1;
        game.beat_started = false;
    }
}

fn enter_beat(world: &mut World, game: &mut GameState, sector_index: usize, beat_index: usize) {
    match &SECTORS[sector_index].beats[beat_index] {
        Beat::Field { length, count } => scenery::spawn_field(world, game, *length, *count),
        Beat::Rings { count } => scenery::spawn_rings(world, game, *count),
        Beat::Wave { groups } => {
            let mut slot = 0usize;
            for (kind, amount) in groups.iter() {
                for _ in 0..*amount {
                    let lane_x = random_range(&mut game.random_state, -5.5, 5.5);
                    let lane_y = BASE_HEIGHT + random_range(&mut game.random_state, -2.6, 2.6);
                    let stagger =
                        slot as f32 * 16.0 + random_range(&mut game.random_state, 0.0, 10.0);
                    let position = Vec3::new(lane_x, lane_y, -ENEMY_SPAWN_AHEAD - stagger);
                    enemies::spawn(world, game, *kind, position);
                    slot += 1;
                }
            }
        }
        Beat::MiniBoss(kind) | Beat::Boss(kind) => boss::spawn(world, game, *kind),
        Beat::Breather { .. } => {}
    }
}

fn beat_complete(game: &GameState, beat: &Beat) -> bool {
    match beat {
        Beat::Field { length, .. } => game.beat_distance >= *length + PATTERN_GAP,
        Beat::Rings { count } => game.beat_distance >= *count as f32 * RING_SPACING + PATTERN_GAP,
        Beat::Breather { length } => game.beat_distance >= *length,
        Beat::Wave { .. } => game.enemies.is_empty(),
        Beat::MiniBoss(_) | Beat::Boss(_) => game.boss.is_none(),
    }
}
