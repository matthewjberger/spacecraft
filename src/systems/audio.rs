use crate::ecs::{Sound, TemplateWorld};
use crate::systems::common::next_random;
use nightshade::prelude::*;

const FIRE_CLIPS: &[(&str, &[u8])] = &[
    ("fire_0", include_bytes!("../../assets/audio/fire_0.ogg")),
    ("fire_1", include_bytes!("../../assets/audio/fire_1.ogg")),
    ("fire_2", include_bytes!("../../assets/audio/fire_2.ogg")),
    ("fire_3", include_bytes!("../../assets/audio/fire_3.ogg")),
    ("fire_4", include_bytes!("../../assets/audio/fire_4.ogg")),
    ("fire_5", include_bytes!("../../assets/audio/fire_5.ogg")),
    ("fire_6", include_bytes!("../../assets/audio/fire_6.ogg")),
    ("fire_7", include_bytes!("../../assets/audio/fire_7.ogg")),
    ("fire_8", include_bytes!("../../assets/audio/fire_8.ogg")),
    ("fire_9", include_bytes!("../../assets/audio/fire_9.ogg")),
];

const ALT_CLIPS: &[(&str, &[u8])] = &[
    ("alt_0", include_bytes!("../../assets/audio/alt_0.ogg")),
    ("alt_1", include_bytes!("../../assets/audio/alt_1.ogg")),
    ("alt_2", include_bytes!("../../assets/audio/alt_2.ogg")),
    ("alt_3", include_bytes!("../../assets/audio/alt_3.ogg")),
    ("alt_4", include_bytes!("../../assets/audio/alt_4.ogg")),
];

const CLIPS: &[(&str, &[u8])] = &[
    (
        "enemy_hit",
        include_bytes!("../../assets/audio/enemy_hit.ogg"),
    ),
    (
        "enemy_explode",
        include_bytes!("../../assets/audio/enemy_explode.ogg"),
    ),
    (
        "big_explode",
        include_bytes!("../../assets/audio/big_explode.ogg"),
    ),
    (
        "player_hit",
        include_bytes!("../../assets/audio/player_hit.ogg"),
    ),
    ("shield", include_bytes!("../../assets/audio/shield.ogg")),
    ("nitrous", include_bytes!("../../assets/audio/nitrous.ogg")),
    ("nova", include_bytes!("../../assets/audio/nova.ogg")),
    ("pickup", include_bytes!("../../assets/audio/pickup.ogg")),
    ("ring", include_bytes!("../../assets/audio/ring.ogg")),
    ("ui_move", include_bytes!("../../assets/audio/ui_move.ogg")),
    (
        "ui_confirm",
        include_bytes!("../../assets/audio/ui_confirm.ogg"),
    ),
    ("ui_back", include_bytes!("../../assets/audio/ui_back.ogg")),
    ("victory", include_bytes!("../../assets/audio/victory.ogg")),
];

fn profile(sound: Sound) -> (&'static str, f32, AudioBus, f32, bool) {
    match sound {
        Sound::EnemyHit => ("enemy_hit", 0.5, AudioBus::Sfx, 1.0, true),
        Sound::EnemyExplode => ("enemy_explode", 0.7, AudioBus::Sfx, 2.0, true),
        Sound::BigExplode => ("big_explode", 0.95, AudioBus::Sfx, 2.5, false),
        Sound::PlayerHit => ("player_hit", 0.8, AudioBus::Sfx, 1.5, false),
        Sound::Shield => ("shield", 0.6, AudioBus::Sfx, 1.5, false),
        Sound::Nitrous => ("nitrous", 0.75, AudioBus::Sfx, 2.5, false),
        Sound::Nova => ("nova", 0.9, AudioBus::Sfx, 2.5, false),
        Sound::Pickup => ("pickup", 0.7, AudioBus::Sfx, 1.0, false),
        Sound::Ring => ("ring", 0.55, AudioBus::Sfx, 1.0, true),
        Sound::UiMove => ("ui_move", 0.5, AudioBus::Ui, 0.8, false),
        Sound::UiConfirm => ("ui_confirm", 0.7, AudioBus::Ui, 1.2, false),
        Sound::UiBack => ("ui_back", 0.6, AudioBus::Ui, 1.2, false),
        Sound::Victory => ("victory", 0.9, AudioBus::Music, 6.0, false),
        Sound::Fire => (FIRE_CLIPS[0].0, 0.42, AudioBus::Sfx, 1.0, true),
        Sound::FireAlt => (ALT_CLIPS[0].0, 0.5, AudioBus::Sfx, 1.2, true),
    }
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    if !game_world.resources.game.audio_enabled {
        game_world.resources.game.sounds.clear();
        let voices = std::mem::take(&mut game_world.resources.game.sfx_voices);
        for (entity, _) in voices {
            despawn_recursive_immediate(world, entity);
        }
        return;
    }

    initialize_audio_system(world);
    build_audio_buses_system(world);

    if !game_world.resources.game.audio_loaded {
        for (name, bytes) in CLIPS.iter().chain(FIRE_CLIPS).chain(ALT_CLIPS) {
            if let Ok(data) = load_sound_from_bytes(bytes) {
                audio_engine_load_sound(&mut world.resources.audio, *name, data);
            }
        }
        set_audio_bus_volume(world, AudioBus::Sfx, -2.0, 0.0);
        set_audio_bus_volume(world, AudioBus::Ui, -4.0, 0.0);
        set_audio_bus_volume(world, AudioBus::Music, -3.0, 0.0);
        game_world.resources.game.audio_loaded = true;
    }

    let queued = std::mem::take(&mut game_world.resources.game.sounds);
    let mut spawned: Vec<Sound> = Vec::new();
    for sound in queued {
        if spawned.iter().filter(|entry| **entry == sound).count() >= 3 {
            continue;
        }
        let (name, volume, bus, lifetime, pitched) = match sound {
            Sound::Fire => {
                let game = &mut game_world.resources.game;
                let clip = FIRE_CLIPS[game.fire_sound_index as usize % FIRE_CLIPS.len()].0;
                game.fire_sound_index = game.fire_sound_index.wrapping_add(1);
                (clip, 0.42, AudioBus::Sfx, 1.0, true)
            }
            Sound::FireAlt => {
                let game = &mut game_world.resources.game;
                let clip = ALT_CLIPS[game.alt_sound_index as usize % ALT_CLIPS.len()].0;
                game.alt_sound_index = game.alt_sound_index.wrapping_add(1);
                (clip, 0.5, AudioBus::Sfx, 1.2, true)
            }
            other => profile(other),
        };
        let rate = if pitched {
            0.94 + next_random(&mut game_world.resources.game.random_state) * 0.12
        } else {
            1.0
        };
        let reverb_send = match bus {
            AudioBus::Sfx => Some(-2.5),
            AudioBus::Ui => Some(-14.0),
            _ => None,
        };
        let mut source = AudioSource::new(name)
            .with_volume(volume)
            .with_bus(bus)
            .with_spatial(false)
            .with_playback_rate(rate as f64)
            .playing();
        if let Some(send) = reverb_send {
            source = source.with_reverb_zone("default", send);
        }
        let entity = spawn_entities(world, AUDIO_SOURCE, 1)[0];
        world.core.set_audio_source(entity, source);
        game_world
            .resources
            .game
            .sfx_voices
            .push((entity, lifetime));
        spawned.push(sound);
    }

    update_audio_system(world);

    let delta = world.resources.window.timing.delta_time;
    let voices = std::mem::take(&mut game_world.resources.game.sfx_voices);
    let mut keep = Vec::with_capacity(voices.len());
    for (entity, timer) in voices {
        let remaining = timer - delta;
        if remaining <= 0.0 {
            despawn_recursive_immediate(world, entity);
        } else {
            keep.push((entity, remaining));
        }
    }
    game_world.resources.game.sfx_voices = keep;
}
