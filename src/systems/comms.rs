use crate::ecs::GameState;
use crate::systems::common::*;

pub const COMMS_DURATION: f32 = 3.8;

fn say(game: &mut GameState, lines: &[&str]) {
    if lines.is_empty() {
        return;
    }
    let pick = (next_random(&mut game.random_state) * lines.len() as f32) as usize;
    game.comms_line = lines[pick.min(lines.len() - 1)].to_string();
    game.comms_timer = COMMS_DURATION;
}

pub fn wave(game: &mut GameState) {
    say(
        game,
        &[
            "WREN: Contacts inbound — break and engage.",
            "TALON: Bandits on the lane. I've got your six, Ranger.",
            "WREN: More of them. Keep the corridor moving.",
            "TALON: Eyes up — they're swarming.",
        ],
    );
}

pub fn belt(game: &mut GameState) {
    say(
        game,
        &[
            "TALON: Rock field dead ahead. Thread it clean.",
            "WREN: Debris belt incoming. Mind your hull.",
        ],
    );
}

pub fn mini_boss(game: &mut GameState) {
    say(
        game,
        &[
            "WREN: Heavy contact. Don't let it dig in.",
            "TALON: That one's armored — hit it hard, Ranger.",
        ],
    );
}

pub fn boss(game: &mut GameState) {
    say(
        game,
        &[
            "WREN: That's the anchor. Put it down.",
            "TALON: Big one. Watch its volleys and stay loose.",
        ],
    );
}

pub fn low_shields(game: &mut GameState) {
    say(
        game,
        &[
            "WREN: Your hull's failing — pull back!",
            "TALON: Ranger, you're lit up — get clear!",
        ],
    );
}
