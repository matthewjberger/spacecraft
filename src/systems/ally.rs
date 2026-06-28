use crate::ecs::{GameMode, ModeKind, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

const PHASE_INACTIVE: u8 = 0;
const PHASE_ESCORT: u8 = 1;
const PHASE_LEAVING: u8 = 2;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.mode == GameMode::Paused {
        return;
    }
    let ship = game.ship_position;
    let elapsed = game.elapsed;
    let active = game.run_mode == ModeKind::Story
        && matches!(game.mode, GameMode::Playing | GameMode::Cinematic);
    let force_leave = game.force_ally_leave;

    let base = nalgebra_glm::quat_angle_axis(SHIP_BASE_YAW, &Vec3::new(0.0, 1.0, 0.0));
    let corner_offsets = [
        Vec3::new(-1.9, 0.4, -1.25),
        Vec3::new(-1.9, -0.18, -1.25),
        Vec3::new(1.9, 0.4, -1.25),
        Vec3::new(1.9, -0.18, -1.25),
    ];

    for ally in game.allies.iter_mut() {
        let slot = ally.slot;
        match ally.phase {
            PHASE_ESCORT if !active || force_leave => {
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

        let (render_pos, bank, visible) = match ally.phase {
            PHASE_ESCORT => {
                let target = ship + Vec3::new(slot * 6.6, 0.7, 2.8);
                ally.position = approach_vec3(ally.position, target, 2.4 * delta);
                let bob = (elapsed * 2.6 + slot).sin() * 0.18;
                let bank = -slot * 0.14 + (ally.position.x - target.x) * 0.05;
                (ally.position + Vec3::new(0.0, bob, 0.0), bank, true)
            }
            PHASE_LEAVING => {
                ally.position += ally.velocity * delta;
                ally.timer -= delta;
                let bank = -slot.signum() * 0.9;
                let visible = ally.timer > 0.0;
                if !visible {
                    ally.phase = PHASE_INACTIVE;
                }
                (ally.position, bank, visible)
            }
            _ => (ally.position, 0.0, false),
        };

        let roll = nalgebra_glm::quat_angle_axis(bank, &Vec3::new(0.0, 0.0, 1.0));
        let rotation = roll * base;
        if let Some(transform) = world.core.get_local_transform_mut(ally.entity) {
            transform.translation = render_pos;
            transform.rotation = rotation;
            transform.scale = if visible {
                Vec3::new(SHIP_SCALE, SHIP_SCALE, SHIP_SCALE)
            } else {
                Vec3::zeros()
            };
        }
        mark_local_transform_dirty(world, ally.entity);

        let exhaust_dir = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
        if let Some(thruster) = ally.thruster {
            let tail =
                render_pos + nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, -0.1, -1.4));
            if let Some(emitter) = world.core.get_particle_emitter_mut(thruster) {
                emitter.position = tail;
                emitter.direction = exhaust_dir;
                emitter.spawn_rate = if visible { 360.0 } else { 0.0 };
            }
        }
        for (corner_index, offset) in corner_offsets.iter().enumerate() {
            if let Some(&corner) = ally.corner_thrusters.get(corner_index) {
                let port = render_pos + nalgebra_glm::quat_rotate_vec3(&rotation, offset);
                if let Some(emitter) = world.core.get_particle_emitter_mut(corner) {
                    emitter.position = port
                        + nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, -0.4, 0.0));
                    emitter.direction = exhaust_dir;
                    emitter.spawn_rate = if visible { 240.0 } else { 0.0 };
                }
            }
        }
    }
}
