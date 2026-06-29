use crate::content::{SECTORS, TAGLINE};
use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn build(game_world: &mut TemplateWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    world.resources.retained_ui.gamepad_nav.enabled = false;
    let root = UiTreeBuilder::new(world).finish();

    let border = vec4(0.35, 0.9, 1.0, 0.6);
    let panel_bg = vec4(0.02, 0.07, 0.13, 0.62);
    let cyan = vec4(0.62, 0.95, 1.0, 1.0);
    let dim = vec4(0.45, 0.8, 1.0, 0.85);
    let red = vec4(1.0, 0.4, 0.35, 1.0);

    let gameplay_panel = window_panel(
        world,
        root,
        Ab(vec2(28.0, 24.0)),
        Ab(vec2(330.0, 316.0)),
        Anchor::TopLeft,
        border,
        panel_bg,
    );
    let sector = text_line(world, gameplay_panel, "SECTOR I", 19.0, cyan, 24.0);
    let score = text_line(world, gameplay_panel, "SCORE  0", 16.0, dim, 22.0);
    let combo = text_line(
        world,
        gameplay_panel,
        "",
        15.0,
        vec4(1.0, 0.85, 0.35, 1.0),
        18.0,
    );
    label_line(world, gameplay_panel, "SHIELDS", dim);
    let shields_bar = bar(world, gameplay_panel, vec4(0.4, 0.95, 1.0, 1.0));
    label_line(world, gameplay_panel, "THRUST", dim);
    let thrust_bar = bar(world, gameplay_panel, vec4(0.3, 0.7, 1.0, 1.0));
    label_line(world, gameplay_panel, "APPROACH", dim);
    let approach_bar = bar(world, gameplay_panel, vec4(0.7, 0.95, 0.6, 1.0));

    let boss_panel = window_panel(
        world,
        root,
        Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, 18.0)),
        Ab(vec2(540.0, 66.0)),
        Anchor::TopCenter,
        red,
        vec4(0.12, 0.02, 0.04, 0.66),
    );
    let boss_label = centered_line(world, boss_panel, "MONARCH", 17.0, red, 22.0);
    let boss_bar = bar(world, boss_panel, vec4(1.0, 0.3, 0.28, 1.0));

    let pickup_panel = window_panel(
        world,
        root,
        Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -28.0)),
        Ab(vec2(580.0, 120.0)),
        Anchor::BottomCenter,
        vec4(1.0, 0.8, 0.3, 0.7),
        vec4(0.1, 0.08, 0.02, 0.66),
    );
    let pickup_label = centered_line(
        world,
        pickup_panel,
        "",
        24.0,
        vec4(1.0, 0.85, 0.4, 1.0),
        30.0,
    );
    let pickup_time = centered_line(
        world,
        pickup_panel,
        "",
        30.0,
        vec4(1.0, 1.0, 1.0, 1.0),
        36.0,
    );
    let pickup_bar = bar(world, pickup_panel, vec4(1.0, 0.78, 0.2, 1.0));

    let overlay_panel = window_panel(
        world,
        root,
        Rl(vec2(50.0, 50.0)),
        Ab(vec2(760.0, 380.0)),
        Anchor::Center,
        border,
        vec4(0.02, 0.06, 0.12, 0.82),
    );
    let overlay_heading = centered_line(world, overlay_panel, "SPACECRAFT", 52.0, cyan, 66.0);
    let overlay_body = centered_line(world, overlay_panel, TAGLINE, 20.0, dim, 150.0);
    let overlay_prompt = centered_line(
        world,
        overlay_panel,
        "PRESS SPACE TO LAUNCH",
        22.0,
        vec4(1.0, 0.95, 0.6, 1.0),
        40.0,
    );

    let ability_panel = window_panel(
        world,
        root,
        Rl(vec2(0.0, 100.0)) + Ab(vec2(28.0, -24.0)),
        Ab(vec2(252.0, 152.0)),
        Anchor::BottomLeft,
        border,
        panel_bg,
    );
    centered_line(world, ability_panel, "ABILITIES", 12.0, dim, 16.0);
    let lance_label = text_line(world, ability_panel, "[F] LANCE", 15.0, cyan, 18.0);
    let lance_bar = bar(world, ability_panel, vec4(0.5, 0.95, 1.0, 1.0));
    let nova_label = text_line(
        world,
        ability_panel,
        "[C] NOVA",
        15.0,
        vec4(1.0, 0.85, 0.4, 1.0),
        20.0,
    );
    let aegis_label = text_line(
        world,
        ability_panel,
        "[V] AEGIS",
        15.0,
        vec4(0.6, 0.9, 1.0, 1.0),
        18.0,
    );
    let aegis_bar = bar(world, ability_panel, vec4(0.4, 0.8, 1.0, 1.0));

    let damage_flash = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(vec4(1.0, 0.12, 0.12, 0.52))
            .with_visible(false)
            .entity()
    };
    let nova_flash = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(vec4(1.0, 1.0, 0.92, 0.55))
            .with_visible(false)
            .entity()
    };

    let comms_panel = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(
                Rl(vec2(0.0, 100.0)) + Ab(vec2(28.0, -188.0)),
                Ab(vec2(420.0, 92.0)),
                Anchor::BottomLeft,
            )
            .with_rect(3.0, 1.5, vec4(0.4, 0.9, 1.0, 0.85))
            .color_raw::<UiBase>(vec4(0.03, 0.06, 0.13, 0.9))
            .with_visible(false)
            .entity()
    };
    let comms_avatar = {
        let mut tree = UiTreeBuilder::from_parent(world, comms_panel);
        tree.add_node()
            .window(Ab(vec2(12.0, 12.0)), Ab(vec2(68.0, 68.0)), Anchor::TopLeft)
            .with_rect(2.0, 1.0, vec4(0.5, 0.95, 1.0, 1.0))
            .color_raw::<UiBase>(vec4(0.07, 0.16, 0.27, 1.0))
            .entity()
    };
    let comms_initial = {
        let mut tree = UiTreeBuilder::from_parent(world, comms_panel);
        tree.add_node()
            .window(Ab(vec2(12.0, 18.0)), Ab(vec2(68.0, 56.0)), Anchor::TopLeft)
            .with_text("W", 34.0)
            .text_center()
            .color_raw::<UiBase>(vec4(0.7, 0.95, 1.0, 1.0))
            .entity()
    };
    let comms_name = {
        let mut tree = UiTreeBuilder::from_parent(world, comms_panel);
        tree.add_node()
            .window(Ab(vec2(92.0, 14.0)), Ab(vec2(312.0, 22.0)), Anchor::TopLeft)
            .with_text("", 15.0)
            .text_left()
            .color_raw::<UiBase>(vec4(0.5, 0.95, 1.0, 1.0))
            .entity()
    };
    let comms_text = {
        let mut tree = UiTreeBuilder::from_parent(world, comms_panel);
        tree.add_node()
            .window(Ab(vec2(92.0, 38.0)), Ab(vec2(316.0, 46.0)), Anchor::TopLeft)
            .with_text("", 17.0)
            .text_left()
            .color_raw::<UiBase>(vec4(0.92, 0.96, 1.0, 1.0))
            .entity()
    };

    ui_mark_render_dirty(world);

    let hud = &mut game_world.resources.game.hud;
    hud.gameplay_panel = Some(gameplay_panel);
    hud.sector = Some(sector);
    hud.score = Some(score);
    hud.combo = Some(combo);
    hud.shields_bar = Some(shields_bar);
    hud.thrust_bar = Some(thrust_bar);
    hud.approach_bar = Some(approach_bar);
    hud.boss_panel = Some(boss_panel);
    hud.boss_label = Some(boss_label);
    hud.boss_bar = Some(boss_bar);
    hud.pickup_panel = Some(pickup_panel);
    hud.pickup_label = Some(pickup_label);
    hud.pickup_time = Some(pickup_time);
    hud.pickup_bar = Some(pickup_bar);
    hud.overlay_panel = Some(overlay_panel);
    hud.overlay_heading = Some(overlay_heading);
    hud.overlay_body = Some(overlay_body);
    hud.overlay_prompt = Some(overlay_prompt);
    hud.damage_flash = Some(damage_flash);
    hud.ability_panel = Some(ability_panel);
    hud.lance_label = Some(lance_label);
    hud.lance_bar = Some(lance_bar);
    hud.nova_label = Some(nova_label);
    hud.aegis_label = Some(aegis_label);
    hud.aegis_bar = Some(aegis_bar);
    hud.nova_flash = Some(nova_flash);
    hud.comms_panel = Some(comms_panel);
    hud.comms_avatar = Some(comms_avatar);
    hud.comms_initial = Some(comms_initial);
    hud.comms_name = Some(comms_name);
    hud.comms_text = Some(comms_text);
}

fn window_panel(
    world: &mut World,
    root: Entity,
    position: impl Into<UiValue<Vec2>>,
    size: impl Into<UiValue<Vec2>>,
    anchor: Anchor,
    border: Vec4,
    fill: Vec4,
) -> Entity {
    let mut tree = UiTreeBuilder::from_parent(world, root);
    tree.add_node()
        .window(position, size, anchor)
        .with_rect(4.0, 1.5, border)
        .color_raw::<UiBase>(fill)
        .flow(FlowDirection::Vertical, 12.0, 6.0)
        .entity()
}

fn text_line(
    world: &mut World,
    parent: Entity,
    text: &str,
    size: f32,
    color: Vec4,
    height: f32,
) -> Entity {
    let mut tree = UiTreeBuilder::from_parent(world, parent);
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .with_text(text, size)
        .text_left()
        .color_raw::<UiBase>(color)
        .entity()
}

fn label_line(world: &mut World, parent: Entity, text: &str, color: Vec4) {
    let mut tree = UiTreeBuilder::from_parent(world, parent);
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 15.0)))
        .with_text(text, 12.0)
        .text_left()
        .color_raw::<UiBase>(color);
}

fn centered_line(
    world: &mut World,
    parent: Entity,
    text: &str,
    size: f32,
    color: Vec4,
    height: f32,
) -> Entity {
    let mut tree = UiTreeBuilder::from_parent(world, parent);
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .with_text(text, size)
        .text_center()
        .color_raw::<UiBase>(color)
        .entity()
}

fn bar(world: &mut World, parent: Entity, color: Vec4) -> Entity {
    let bar_entity = {
        let mut tree = UiTreeBuilder::from_parent(world, parent);
        tree.add_progress_bar(0.0)
    };
    if let Some(fill) = world
        .ui
        .get_ui_progress_bar(bar_entity)
        .map(|data| data.fill_entity)
        && let Some(node_color) = world.ui.get_ui_node_color_mut(fill)
    {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
    bar_entity
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let game = &game_world.resources.game;
    let mode = game.mode;
    let sector_index = game.sector;
    let score = game.score;
    let shields = game.shields.max(0) as f32;
    let max_shields = game.max_shields.max(1) as f32;
    let speed_scale = game.speed_scale;
    let current_node = game.current_node;
    let node_total = game.level.nodes.len().max(1) as f32;
    let mode_timer = game.mode_timer;
    let damage_flash = game.damage_flash;
    let boss = game.boss.as_ref().map(|boss| {
        (
            boss.kind.stats().name,
            boss.health.max(0),
            boss.max_health.max(1),
        )
    });
    let effect = game.effect.map(|kind| {
        (
            kind.tag(),
            kind.label(),
            kind.color(),
            game.effect_timer.max(0.0),
            game.effect_duration.max(0.01),
        )
    });
    let mods = game.mods;
    let laser_cooldown = game.laser_cooldown;
    let nova_charges = game.nova_charges;
    let aegis_timer = game.aegis_timer;
    let aegis_cooldown = game.aegis_cooldown;
    let nova_flash = game.nova_flash;
    let combo = game.combo;
    let score_flash = game.score_flash;
    let best_combo = game.best_combo;
    let best_score = game.best_score;
    let comms_line = game.comms_line.clone();
    let comms_timer = game.comms_timer;
    let loop_count = game.loop_count;
    let menu_cursor = game.menu_cursor;
    let settings_cursor = game.settings_cursor;
    let shake_on = game.shake_enabled;
    let flash_on = game.flash_enabled;
    let starfield_on = game.starfield_enabled;
    let hard_on = game.hard_mode;
    let crt_on = game.crt_enabled;
    let audio_on = game.audio_enabled;
    let hud = game.hud;

    let playing = mode == GameMode::Playing;
    let has_ability = mods.lance > 0 || mods.nova_max > 0 || mods.aegis > 0;

    set_visible(world, hud.gameplay_panel, playing);
    set_visible(
        world,
        hud.overlay_panel,
        !playing && mode != GameMode::Cinematic,
    );
    let low_shield_warn = shields <= 1.0 && (mode_timer * 3.0).sin() > 0.0;
    set_visible(
        world,
        hud.damage_flash,
        playing && flash_on && (damage_flash > 0.0 || low_shield_warn),
    );
    set_visible(
        world,
        hud.nova_flash,
        playing && nova_flash > 0.0 && flash_on,
    );
    let comms_on = playing && comms_timer > 0.0 && !comms_line.is_empty();
    set_visible(world, hud.comms_panel, comms_on);
    if comms_on {
        let (speaker, text) = comms_line
            .split_once(": ")
            .unwrap_or(("", comms_line.as_str()));
        let accent = match speaker {
            "WREN" => vec4(0.5, 0.95, 1.0, 1.0),
            "TALON" => vec4(1.0, 0.78, 0.4, 1.0),
            _ => vec4(0.62, 0.92, 1.0, 1.0),
        };
        let initial = speaker.chars().next().map(String::from).unwrap_or_default();
        set_text(world, hud.comms_name, speaker);
        set_text(world, hud.comms_text, text);
        set_text(world, hud.comms_initial, &initial);
        tint_node(world, hud.comms_name, accent);
        tint_node(world, hud.comms_initial, accent);
    }
    set_visible(world, hud.boss_panel, playing && boss.is_some());
    set_visible(world, hud.pickup_panel, playing && effect.is_some());
    set_visible(world, hud.ability_panel, playing && has_ability);
    set_visible(world, hud.lance_label, playing && mods.lance > 0);
    set_visible(world, hud.lance_bar, playing && mods.lance > 0);
    set_visible(world, hud.nova_label, playing && mods.nova_max > 0);
    set_visible(world, hud.aegis_label, playing && mods.aegis > 0);
    set_visible(world, hud.aegis_bar, playing && mods.aegis > 0);

    if playing && has_ability {
        if mods.lance > 0 {
            let span = LASER_DURATION + LASER_COOLDOWN;
            let ready = (1.0 - laser_cooldown / span).clamp(0.0, 1.0);
            set_bar(world, hud.lance_bar, ready);
            tint_node(world, hud.lance_label, ready_color(laser_cooldown <= 0.0));
        }
        if mods.nova_max > 0 {
            set_text(
                world,
                hud.nova_label,
                &format!("[C] NOVA   x{nova_charges}"),
            );
            tint_node(world, hud.nova_label, ready_color(nova_charges > 0));
        }
        if mods.aegis > 0 {
            let ready = if aegis_timer > 0.0 {
                1.0
            } else {
                (1.0 - aegis_cooldown / AEGIS_COOLDOWN).clamp(0.0, 1.0)
            };
            set_bar(world, hud.aegis_bar, ready);
            let label = if aegis_timer > 0.0 {
                "[V] AEGIS  ON"
            } else {
                "[V] AEGIS"
            };
            set_text(world, hud.aegis_label, label);
            tint_node(
                world,
                hud.aegis_label,
                ready_color(aegis_timer > 0.0 || aegis_cooldown <= 0.0),
            );
        }
    }

    if playing {
        let sector = &SECTORS[sector_index];
        let sector_text = if loop_count > 0 {
            format!("{}  ·  LOOP {}", sector.name, loop_count + 1)
        } else {
            format!("{}  {}", sector.name, sector.subtitle)
        };
        set_text(world, hud.sector, &sector_text);
        set_text(world, hud.score, &format!("SCORE  {score}"));
        tint_node(
            world,
            hud.score,
            if score_flash > 0.0 {
                vec4(1.0, 0.92, 0.4, 1.0)
            } else {
                vec4(0.45, 0.8, 1.0, 0.85)
            },
        );
        if combo > 1 {
            set_text(
                world,
                hud.combo,
                &format!("COMBO  x{}   ({combo})", combo_multiplier(combo)),
            );
        } else {
            set_text(world, hud.combo, " ");
        }
        set_bar(world, hud.shields_bar, shields / max_shields);
        set_bar(
            world,
            hud.thrust_bar,
            ((speed_scale - 0.5) / 1.4).clamp(0.0, 1.0),
        );
        set_bar(
            world,
            hud.approach_bar,
            (current_node as f32 / node_total).clamp(0.0, 1.0),
        );

        if let Some((name, health, max_health)) = boss {
            set_text(world, hud.boss_label, &format!("BOSS  ·  {name}"));
            set_bar(world, hud.boss_bar, health as f32 / max_health as f32);
        }
        if let Some((tag, label, color, timer, duration)) = effect {
            set_text(world, hud.pickup_label, &format!("{tag}   {label}"));
            set_text(world, hud.pickup_time, &format!("{timer:.2}s"));
            set_bar(world, hud.pickup_bar, (timer / duration).clamp(0.0, 1.0));
            tint_bar(world, hud.pickup_bar, color);
        }
    } else {
        let blink = (mode_timer * 1.6).fract() < 0.62;
        let (heading, body, prompt) = match mode {
            GameMode::Title => title_overlay(menu_cursor, best_score),
            GameMode::Settings => settings_overlay(
                settings_cursor,
                shake_on,
                flash_on,
                starfield_on,
                hard_on,
                crt_on,
                audio_on,
            ),
            GameMode::Paused => pause_overlay(menu_cursor),
            GameMode::LevelSelect => level_select_overlay(menu_cursor),
            _ => overlay_text(mode, sector_index, score, best_combo),
        };
        let blinkable = matches!(
            mode,
            GameMode::Title | GameMode::Settings | GameMode::Paused | GameMode::LevelSelect
        );
        set_text(world, hud.overlay_heading, &heading);
        set_text(world, hud.overlay_body, &body);
        set_text(
            world,
            hud.overlay_prompt,
            if blink || blinkable { &prompt } else { " " },
        );
    }

    ui_mark_render_dirty(world);
}

fn overlay_text(
    mode: GameMode,
    sector_index: usize,
    score: u32,
    best_combo: u32,
) -> (String, String, String) {
    match mode {
        GameMode::Title => (
            "SPACECRAFT".to_string(),
            TAGLINE.to_string(),
            "PRESS SPACE TO LAUNCH".to_string(),
        ),
        GameMode::Briefing => {
            let sector = &SECTORS[sector_index];
            (
                format!("{}  —  {}", sector.name, sector.subtitle),
                sector.briefing.to_string(),
                "SPACE  —  ENGAGE".to_string(),
            )
        }
        GameMode::SectorClear => (
            "SECTOR CLEAR".to_string(),
            format!("{}\n\nSCORE  {score}", SECTORS[sector_index].debrief),
            "SPACE  —  PRESS ON".to_string(),
        ),
        GameMode::GameOver => (
            "SHIELDS DOWN".to_string(),
            format!(
                "The corridor closes behind you, and the swarm never slows.\nBack there, the Lantern Fleet's air runs thin.\nSCORE  {score}      BEST COMBO  x{best_combo}"
            ),
            "SPACE  —  TRY AGAIN".to_string(),
        ),
        GameMode::Victory => (
            "THE LOOP IS BROKEN".to_string(),
            format!(
                "The Monarch goes dark and the swarm forgets how to move.\nThe Lantern Fleet runs the gap — every last lantern home.\nTALON: Tesse can rest now, Ranger.\nSCORE  {score}      BEST COMBO  x{best_combo}"
            ),
            "SPACE  —  PRESS ON".to_string(),
        ),
        GameMode::Playing
        | GameMode::Settings
        | GameMode::Paused
        | GameMode::LevelSelect
        | GameMode::Cinematic => (String::new(), String::new(), String::new()),
    }
}

fn level_select_overlay(cursor: usize) -> (String, String, String) {
    let items: Vec<String> = SECTORS
        .iter()
        .map(|sector| format!("{}  —  {}", sector.name, sector.subtitle))
        .collect();
    (
        "SELECT SECTOR".to_string(),
        menu_body(&items, cursor),
        "UP / DOWN  SELECT       SPACE  LAUNCH       ESC  BACK".to_string(),
    )
}

fn pause_overlay(cursor: usize) -> (String, String, String) {
    let items = ["RESUME".to_string(), "QUIT TO TITLE".to_string()];
    (
        "PAUSED".to_string(),
        menu_body(&items, cursor),
        "UP / DOWN  SELECT       SPACE  CONFIRM       ESC  RESUME".to_string(),
    )
}

fn menu_body(items: &[String], cursor: usize) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == cursor {
                format!(">  {item}")
            } else {
                format!("    {item}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn title_overlay(cursor: usize, best: u32) -> (String, String, String) {
    let items = [
        "STORY".to_string(),
        "ARCADE".to_string(),
        "ENDLESS".to_string(),
        "SETTINGS".to_string(),
    ];
    let body = if best > 0 {
        format!("BEST  {best}\n\n{}", menu_body(&items, cursor))
    } else {
        menu_body(&items, cursor)
    };
    (
        "SPACECRAFT".to_string(),
        body,
        "UP / DOWN  SELECT       SPACE  CONFIRM".to_string(),
    )
}

fn settings_overlay(
    cursor: usize,
    shake: bool,
    flash: bool,
    starfield: bool,
    hard: bool,
    crt: bool,
    audio: bool,
) -> (String, String, String) {
    let on = |value: bool| if value { "ON" } else { "OFF" };
    let items = [
        format!("SCREEN SHAKE     {}", on(shake)),
        format!("DAMAGE FLASH     {}", on(flash)),
        format!("STARFIELD        {}", on(starfield)),
        format!("DIFFICULTY       {}", if hard { "HARD" } else { "NORMAL" }),
        format!("CRT FILTER       {}", on(crt)),
        format!("AUDIO            {}", on(audio)),
        "BACK".to_string(),
    ];
    (
        "SETTINGS".to_string(),
        menu_body(&items, cursor),
        "UP / DOWN  SELECT       SPACE  TOGGLE".to_string(),
    )
}

fn ready_color(ready: bool) -> Vec4 {
    if ready {
        vec4(0.65, 1.0, 0.7, 1.0)
    } else {
        vec4(0.5, 0.6, 0.7, 0.7)
    }
}

fn tint_node(world: &mut World, entity: Option<Entity>, color: Vec4) {
    if let Some(entity) = entity
        && let Some(node_color) = world.ui.get_ui_node_color_mut(entity)
    {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
}

fn set_visible(world: &mut World, entity: Option<Entity>, visible: bool) {
    if let Some(entity) = entity {
        ui_set_visible(world, entity, visible);
    }
}

fn set_text(world: &mut World, entity: Option<Entity>, text: &str) {
    if let Some(entity) = entity {
        ui_set_text(world, entity, text);
    }
}

fn set_bar(world: &mut World, entity: Option<Entity>, value: f32) {
    if let Some(entity) = entity {
        ui_progress_bar_set_value(world, entity, value.clamp(0.0, 1.0));
    }
}

fn tint_bar(world: &mut World, entity: Option<Entity>, color: Vec4) {
    if let Some(entity) = entity
        && let Some(fill) = world
            .ui
            .get_ui_progress_bar(entity)
            .map(|data| data.fill_entity)
        && let Some(node_color) = world.ui.get_ui_node_color_mut(fill)
    {
        node_color.colors[UiBase::INDEX] = Some(color);
    }
}
