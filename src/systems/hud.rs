use crate::content::{ModKind, SECTORS, SHOP_ITEMS, TAGLINE};
use crate::ecs::{GameMode, GameState, TemplateWorld};
use crate::systems::common::*;
use crate::systems::shop;
use nightshade::prelude::*;

pub fn build(game_world: &mut TemplateWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
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
        Ab(vec2(300.0, 232.0)),
        Anchor::TopLeft,
        border,
        panel_bg,
    );
    let sector = text_line(world, gameplay_panel, "SECTOR I", 19.0, cyan, 24.0);
    let score = text_line(world, gameplay_panel, "SCORE  0", 16.0, dim, 22.0);
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

    let shop_panel = window_panel(
        world,
        root,
        Rl(vec2(50.0, 50.0)),
        Ab(vec2(780.0, 492.0)),
        Anchor::Center,
        vec4(1.0, 0.85, 0.4, 0.7),
        vec4(0.04, 0.05, 0.1, 0.88),
    );
    centered_line(
        world,
        shop_panel,
        "OUTFITTING",
        30.0,
        vec4(1.0, 0.9, 0.5, 1.0),
        40.0,
    );
    let shop_credits = centered_line(
        world,
        shop_panel,
        "CREDITS  0",
        20.0,
        vec4(0.7, 1.0, 0.7, 1.0),
        28.0,
    );
    let mut shop_lines: [Option<Entity>; 8] = [None; 8];
    for slot in shop_lines.iter_mut().take(SHOP_ITEMS.len()) {
        *slot = Some(text_line(world, shop_panel, "", 17.0, dim, 26.0));
    }
    let shop_prompt = centered_line(
        world,
        shop_panel,
        "UP / DOWN  SELECT      SPACE  BUY      ENTER  LAUNCH",
        15.0,
        vec4(1.0, 0.95, 0.6, 1.0),
        30.0,
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
            .color_raw::<UiBase>(vec4(0.9, 0.1, 0.12, 0.28))
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

    ui_mark_render_dirty(world);

    let hud = &mut game_world.resources.game.hud;
    hud.gameplay_panel = Some(gameplay_panel);
    hud.sector = Some(sector);
    hud.score = Some(score);
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
    hud.shop_panel = Some(shop_panel);
    hud.shop_credits = Some(shop_credits);
    hud.shop_lines = shop_lines;
    hud.shop_prompt = Some(shop_prompt);
    hud.ability_panel = Some(ability_panel);
    hud.lance_label = Some(lance_label);
    hud.lance_bar = Some(lance_bar);
    hud.nova_label = Some(nova_label);
    hud.aegis_label = Some(aegis_label);
    hud.aegis_bar = Some(aegis_bar);
    hud.nova_flash = Some(nova_flash);
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
    let beat_index = game.beat_index;
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
    let shop_data = if mode == GameMode::Shop {
        Some((game.credits, shop_lines(game)))
    } else {
        None
    };
    let mods = game.mods;
    let laser_cooldown = game.laser_cooldown;
    let nova_charges = game.nova_charges;
    let aegis_timer = game.aegis_timer;
    let aegis_cooldown = game.aegis_cooldown;
    let nova_flash = game.nova_flash;
    let menu_cursor = game.menu_cursor;
    let settings_cursor = game.settings_cursor;
    let shake_on = game.shake_enabled;
    let flash_on = game.flash_enabled;
    let starfield_on = game.starfield_enabled;
    let hud = game.hud;

    let playing = mode == GameMode::Playing;
    let shopping = mode == GameMode::Shop;
    let has_ability = mods.lance > 0 || mods.nova_max > 0 || mods.aegis > 0;

    set_visible(world, hud.gameplay_panel, playing);
    set_visible(world, hud.overlay_panel, !playing && !shopping);
    set_visible(world, hud.shop_panel, shopping);
    set_visible(
        world,
        hud.damage_flash,
        playing && damage_flash > 0.0 && flash_on,
    );
    set_visible(
        world,
        hud.nova_flash,
        playing && nova_flash > 0.0 && flash_on,
    );
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

    if let Some((credits, lines)) = shop_data {
        set_text(world, hud.shop_credits, &format!("CREDITS  {credits}"));
        for (index, (text, color)) in lines.into_iter().enumerate() {
            set_text(world, hud.shop_lines[index], &text);
            tint_node(world, hud.shop_lines[index], color);
        }
    }

    if playing {
        let sector = &SECTORS[sector_index];
        set_text(
            world,
            hud.sector,
            &format!("{}  {}", sector.name, sector.subtitle),
        );
        set_text(world, hud.score, &format!("SCORE  {score}"));
        set_bar(world, hud.shields_bar, shields / max_shields);
        set_bar(
            world,
            hud.thrust_bar,
            ((speed_scale - 0.5) / 1.4).clamp(0.0, 1.0),
        );
        let beat_total = SECTORS[sector_index].beats.len().max(1) as f32;
        set_bar(
            world,
            hud.approach_bar,
            (beat_index as f32 / beat_total).clamp(0.0, 1.0),
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
    } else if !shopping {
        let blink = (mode_timer * 1.6).fract() < 0.62;
        let (heading, body, prompt) = match mode {
            GameMode::Title => title_overlay(menu_cursor),
            GameMode::Settings => {
                settings_overlay(settings_cursor, shake_on, flash_on, starfield_on)
            }
            _ => overlay_text(mode, sector_index, score),
        };
        let blinkable = matches!(mode, GameMode::Title | GameMode::Settings);
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

fn overlay_text(mode: GameMode, sector_index: usize, score: u32) -> (String, String, String) {
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
            format!("Corridor held.\nSCORE  {score}"),
            "SPACE  —  PRESS ON".to_string(),
        ),
        GameMode::GameOver => (
            "SHIELDS DOWN".to_string(),
            format!("The corridor closes behind you.\nSCORE  {score}"),
            "SPACE  —  TRY AGAIN".to_string(),
        ),
        GameMode::Victory => (
            "THE DRIFT IS BROKEN".to_string(),
            format!("The Monarch is dust. The fleet rolls in.\nFINAL SCORE  {score}"),
            "SPACE  —  FLY AGAIN".to_string(),
        ),
        GameMode::Playing | GameMode::Shop | GameMode::Settings => {
            (String::new(), String::new(), String::new())
        }
    }
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

fn title_overlay(cursor: usize) -> (String, String, String) {
    let items = ["LAUNCH".to_string(), "SETTINGS".to_string()];
    (
        "SPACECRAFT".to_string(),
        menu_body(&items, cursor),
        "UP / DOWN  SELECT       SPACE  CONFIRM".to_string(),
    )
}

fn settings_overlay(
    cursor: usize,
    shake: bool,
    flash: bool,
    starfield: bool,
) -> (String, String, String) {
    let on = |value: bool| if value { "ON" } else { "OFF" };
    let items = [
        format!("SCREEN SHAKE     {}", on(shake)),
        format!("DAMAGE FLASH     {}", on(flash)),
        format!("STARFIELD        {}", on(starfield)),
        "BACK".to_string(),
    ];
    (
        "SETTINGS".to_string(),
        menu_body(&items, cursor),
        "UP / DOWN  SELECT       SPACE  TOGGLE".to_string(),
    )
}

fn shop_lines(game: &GameState) -> Vec<(String, Vec4)> {
    SHOP_ITEMS
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let level = shop::item_level(&game.mods, item.kind);
            let cost = shop::current_cost(game, item);
            let selected = index == game.shop_cursor;
            let marker = if selected { ">" } else { " " };
            let status = if shop::maxed(game, item) {
                "MAX".to_string()
            } else {
                format!("{cost}c")
            };
            let level_text = if item.kind == ModKind::Repair {
                String::new()
            } else {
                format!("Lv {level}/{}", item.max_level)
            };
            let text = format!(
                "{marker} {})  {}   {}   {}   {}",
                index + 1,
                item.name,
                item.desc,
                level_text,
                status
            );
            let color = if selected {
                vec4(1.0, 0.95, 0.6, 1.0)
            } else if shop::can_buy(game, item) {
                vec4(0.65, 1.0, 0.7, 1.0)
            } else if shop::maxed(game, item) {
                vec4(0.5, 0.7, 0.85, 0.7)
            } else {
                vec4(0.9, 0.6, 0.5, 0.75)
            };
            (text, color)
        })
        .collect()
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
