use crate::content::{SECTORS, TAGLINE};
use crate::ecs::{GameMode, TemplateWorld};
use nightshade::prelude::*;

pub fn build(game_world: &mut TemplateWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    let root = UiTreeBuilder::new(world).finish();

    let border = vec4(0.35, 0.9, 1.0, 0.6);
    let panel_bg = vec4(0.02, 0.07, 0.13, 0.62);
    let cyan = vec4(0.62, 0.95, 1.0, 1.0);
    let dim = vec4(0.45, 0.8, 1.0, 0.85);

    let gameplay_panel = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(
                Ab(vec2(28.0, 24.0)),
                Ab(vec2(310.0, 196.0)),
                Anchor::TopLeft,
            )
            .with_rect(4.0, 1.5, border)
            .color_raw::<UiBase>(panel_bg)
            .flow(FlowDirection::Vertical, 14.0, 8.0)
            .entity()
    };
    let sector = text_line(world, gameplay_panel, "SECTOR I", 19.0, cyan, 26.0);
    let shields = text_line(world, gameplay_panel, "SHIELDS  ####", 16.0, dim, 22.0);
    let score = text_line(world, gameplay_panel, "SCORE  0", 16.0, dim, 22.0);
    let thrust = text_line(world, gameplay_panel, "THRUST  [    ]", 14.0, dim, 20.0);
    let progress = text_line(
        world,
        gameplay_panel,
        "APPROACH  [            ]",
        14.0,
        cyan,
        20.0,
    );
    let boss = text_line(
        world,
        gameplay_panel,
        "",
        14.0,
        vec4(1.0, 0.5, 0.45, 1.0),
        20.0,
    );

    let overlay_panel = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(Rl(vec2(50.0, 50.0)), Ab(vec2(760.0, 380.0)), Anchor::Center)
            .with_rect(6.0, 2.0, border)
            .color_raw::<UiBase>(vec4(0.02, 0.06, 0.12, 0.78))
            .flow(FlowDirection::Vertical, 30.0, 18.0)
            .entity()
    };
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

    let damage_flash = {
        let mut tree = UiTreeBuilder::from_parent(world, root);
        tree.add_node()
            .window(Ab(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)), Anchor::TopLeft)
            .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
            .color_raw::<UiBase>(vec4(0.9, 0.1, 0.12, 0.28))
            .with_visible(false)
            .entity()
    };

    ui_mark_render_dirty(world);

    let hud = &mut game_world.resources.game.hud;
    hud.gameplay_panel = Some(gameplay_panel);
    hud.sector = Some(sector);
    hud.shields = Some(shields);
    hud.score = Some(score);
    hud.thrust = Some(thrust);
    hud.progress = Some(progress);
    hud.boss = Some(boss);
    hud.overlay_panel = Some(overlay_panel);
    hud.overlay_heading = Some(overlay_heading);
    hud.overlay_body = Some(overlay_body);
    hud.overlay_prompt = Some(overlay_prompt);
    hud.damage_flash = Some(damage_flash);
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

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let game = &game_world.resources.game;
    let mode = game.mode;
    let sector_index = game.sector;
    let score = game.score;
    let shields = game.shields.max(0) as usize;
    let max_shields = game.max_shields as usize;
    let speed_scale = game.speed_scale;
    let distance = game.distance;
    let goal = game.sector_goal.max(1.0);
    let mode_timer = game.mode_timer;
    let damage_flash = game.damage_flash;
    let boss_ratio = game
        .boss
        .as_ref()
        .map(|boss| (boss.health.max(0) as f32) / (boss.max_health.max(1) as f32));
    let hud = game.hud;

    let playing = mode == GameMode::Playing;

    set_visible(world, hud.gameplay_panel, playing);
    set_visible(world, hud.overlay_panel, !playing);
    set_visible(world, hud.damage_flash, playing && damage_flash > 0.0);

    if playing {
        let sector = &SECTORS[sector_index];
        set_text(
            world,
            hud.sector,
            &format!("{}  {}", sector.name, sector.subtitle),
        );
        set_text(
            world,
            hud.shields,
            &format!("SHIELDS  {}", pips(shields, max_shields)),
        );
        set_text(world, hud.score, &format!("SCORE  {score}"));
        let thrust_filled = (((speed_scale - 0.5) * 8.0).round() as i32).clamp(0, 10) as usize;
        set_text(
            world,
            hud.thrust,
            &format!("THRUST  [{}]", meter(thrust_filled, 10)),
        );
        let approach = ((distance / goal).clamp(0.0, 1.0) * 12.0) as usize;
        set_text(
            world,
            hud.progress,
            &format!("APPROACH  [{}]", meter(approach, 12)),
        );
        if let Some(ratio) = boss_ratio {
            let filled = (ratio.clamp(0.0, 1.0) * 14.0).ceil() as usize;
            set_text(
                world,
                hud.boss,
                &format!("MONARCH  [{}]", meter(filled, 14)),
            );
            set_visible(world, hud.boss, true);
        } else {
            set_visible(world, hud.boss, false);
        }
    } else {
        let blink = (mode_timer * 1.6).fract() < 0.62;
        let (heading, body, prompt) = overlay_text(mode, sector_index, score);
        set_text(world, hud.overlay_heading, &heading);
        set_text(world, hud.overlay_body, &body);
        set_text(world, hud.overlay_prompt, if blink { &prompt } else { " " });
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
        GameMode::Playing => (String::new(), String::new(), String::new()),
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

fn pips(filled: usize, total: usize) -> String {
    (0..total)
        .map(|index| if index < filled { '#' } else { '-' })
        .collect()
}

fn meter(filled: usize, total: usize) -> String {
    (0..total)
        .map(|index| if index < filled { '#' } else { ' ' })
        .collect()
}
