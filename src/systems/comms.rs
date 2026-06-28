use crate::content::BossKind;
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

fn bark(game: &mut GameState, lines: &[&str]) {
    if game.comms_timer > 0.0 {
        return;
    }
    say(game, lines);
}

pub fn boss_phase(game: &mut GameState, kind: BossKind, phase: u8) {
    if kind == BossKind::Monarch {
        let line = match phase {
            1 => "MONARCH: ...home... you carry our voices... why do you... burn us...",
            _ => "WREN: That's the colony's ghost talking. End the loop, Ranger. Now.",
        };
        game.comms_line = line.to_string();
        game.comms_timer = COMMS_DURATION;
        return;
    }
    let lines: &[&str] = match phase {
        1 => &[
            "TALON: It's changing tactics — stay sharp!",
            "WREN: Armor's splitting. Keep on it.",
        ],
        _ => &[
            "WREN: It's wounded and wild — don't let up!",
            "TALON: Last push, Ranger. Put it down!",
        ],
    };
    say(game, lines);
}

pub fn kill_streak(game: &mut GameState) {
    bark(
        game,
        &[
            "TALON: Beautiful flying, Ranger!",
            "WREN: That's how you clear a lane.",
            "TALON: They can't lay a finger on you!",
        ],
    );
}

pub fn wave_clear(game: &mut GameState) {
    bark(
        game,
        &[
            "WREN: Lane's clear. Keep the corridor moving.",
            "TALON: Scratch that wing. Next stretch ahead.",
        ],
    );
}

pub fn roll(game: &mut GameState) {
    bark(game, &["TALON: Nice roll!", "WREN: Good evasion, Ranger."]);
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
