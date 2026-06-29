use crate::ecs::GameState;
use nightshade::prelude::*;

pub use crate::systems::spawn::*;
pub use crate::systems::tuning::*;

pub fn combo_multiplier(combo: u32) -> u32 {
    1 + (combo / 8).min(4)
}

pub fn award(game: &mut GameState, base: u32) {
    let previous_multiplier = combo_multiplier(game.combo);
    game.combo += 1;
    game.best_combo = game.best_combo.max(game.combo);
    game.combo_timer = COMBO_WINDOW;
    let multiplier = combo_multiplier(game.combo);
    game.score += base * multiplier;
    game.score_flash = 0.3;
    if multiplier > previous_multiplier {
        game.score_flash = 0.5;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_COMBO);
    }
}

pub fn difficulty(game: &GameState) -> u32 {
    game.loop_count * 2 + game.sector as u32 + if game.hard_mode { 2 } else { 0 }
}

pub fn aim_lead(game: &GameState) -> (f32, f32) {
    (-game.roll / MAX_BANK, game.pitch / MAX_PITCH)
}

pub fn aim_point(game: &GameState) -> Vec3 {
    let (lead_x, lead_y) = aim_lead(game);
    Vec3::new(
        game.ship_position.x + lead_x * AIM_FAR_LEAD_X,
        game.ship_position.y + lead_y * AIM_FAR_LEAD_Y,
        game.ship_position.z - RETICLE_FAR_Z,
    )
}

pub fn approach(current: f32, target: f32, rate: f32) -> f32 {
    current + (target - current) * rate.clamp(0.0, 1.0)
}

pub fn course_bend(game: &GameState, position: Vec3) -> Vec3 {
    let depth = (game.ship_position.z - position.z).max(0.0);
    let factor = depth * depth;
    Vec3::new(game.curve_x * factor, 0.0, 0.0)
}

pub fn approach_vec3(current: Vec3, target: Vec3, rate: f32) -> Vec3 {
    current + (target - current) * rate.clamp(0.0, 1.0)
}

pub fn next_random(state: &mut u64) -> f32 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    *state = value;
    ((value >> 40) as f32) / ((1u64 << 24) as f32)
}

pub fn random_range(state: &mut u64, low: f32, high: f32) -> f32 {
    low + (high - low) * next_random(state)
}
