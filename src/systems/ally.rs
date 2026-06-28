use crate::ecs::{GameMode, ModeKind, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

const PHASE_INACTIVE: u8 = 0;
const PHASE_ESCORT: u8 = 1;
const PHASE_LEAVING: u8 = 2;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let ship = game.ship_position;
    let elapsed = game.elapsed;
    let active = game.run_mode == ModeKind::Story
        && matches!(game.mode, GameMode::Playing | GameMode::Cinematic);

    let base = nalgebra_glm::quat_angle_axis(SHIP_BASE_YAW, &Vec3::new(0.0, 1.0, 0.0));
    let mut updates: Vec<(Entity, Vec3, f32, bool)> = Vec::new();

    for ally in game.allies.iter_mut() {
        let slot = ally.slot;
        match ally.phase {
            PHASE_ESCORT if !active => {
                ally.phase = PHASE_LEAVING;
                ally.velocity = Vec3::new(slot.signum() * 20.0, 7.0, -24.0);
                ally.timer = 3.0;
            }
            PHASE_INACTIVE if active => {
                ally.phase = PHASE_ESCORT;
                ally.position = ship + Vec3::new(slot * 4.0, 0.6, 30.0);
            }
            _ => {}
        }

        match ally.phase {
            PHASE_ESCORT => {
                let target = ship + Vec3::new(slot * 6.6, 0.7, 2.8);
                ally.position = approach_vec3(ally.position, target, 2.4 * delta);
                let bob = (elapsed * 2.6 + slot).sin() * 0.18;
                let bank = -slot * 0.14 + (ally.position.x - target.x) * 0.05;
                updates.push((
                    ally.entity,
                    ally.position + Vec3::new(0.0, bob, 0.0),
                    bank,
                    true,
                ));
            }
            PHASE_LEAVING => {
                ally.position += ally.velocity * delta;
                ally.timer -= delta;
                let bank = -slot.signum() * 0.9;
                let visible = ally.timer > 0.0;
                if !visible {
                    ally.phase = PHASE_INACTIVE;
                }
                updates.push((ally.entity, ally.position, bank, visible));
            }
            _ => {
                updates.push((ally.entity, ally.position, 0.0, false));
            }
        }
    }

    for (entity, position, bank, visible) in updates {
        let roll = nalgebra_glm::quat_angle_axis(bank, &Vec3::new(0.0, 0.0, 1.0));
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = roll * base;
            transform.scale = if visible {
                Vec3::new(SHIP_SCALE, SHIP_SCALE, SHIP_SCALE)
            } else {
                Vec3::zeros()
            };
        }
        mark_local_transform_dirty(world, entity);
    }
}
