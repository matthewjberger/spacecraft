use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &game_world.resources.game;
    let Some(camera) = game.camera else {
        return;
    };
    let mode = game.mode;
    let ship = game.ship_position;
    let roll = game.roll;
    let speed_scale = game.speed_scale;
    let shake = if game.shake_enabled { game.shake } else { 0.0 };
    let elapsed = game.elapsed;
    let cam_kick = game.cam_kick;
    let fov_pop = game.cam_fov_pop;
    let steer_lead = -game.roll / MAX_BANK;

    if matches!(
        mode,
        GameMode::Title | GameMode::Settings | GameMode::LevelSelect
    ) {
        let orbit = game.menu_orbit;
        let focus = Vec3::new(0.0, BASE_HEIGHT + 0.4, ship.z - 5.5);
        let (radius, height) = if mode == GameMode::Settings {
            (6.6, 1.5)
        } else {
            (8.6, 2.7)
        };
        let target = Vec3::new(
            focus.x + radius * orbit.cos(),
            focus.y + height,
            focus.z + radius * orbit.sin(),
        );
        if let Some(transform) = world.core.get_local_transform_mut(camera) {
            transform.translation = approach_vec3(
                transform.translation,
                target,
                CAMERA_RESPONSE * delta * 0.55,
            );
            let forward = (focus - transform.translation).normalize();
            let view = nalgebra_glm::quat_look_at_rh(&forward, &Vec3::new(0.0, 1.0, 0.0));
            transform.rotation = nalgebra_glm::quat_inverse(&view);
        }
        mark_local_transform_dirty(world, camera);
        if let Some(component) = world.core.get_camera_mut(camera)
            && let Projection::Perspective(ref mut perspective) = component.projection
        {
            perspective.y_fov_rad = BASE_FOV_DEGREES.to_radians();
        }
        return;
    }

    let shake_offset = Vec3::new(
        (elapsed * 92.0).sin() * shake * 0.16,
        (elapsed * 71.0).cos() * shake * 0.16,
        0.0,
    );
    let target = Vec3::new(
        ship.x * CAMERA_FOLLOW_X + steer_lead * CAMERA_LEAD,
        BASE_HEIGHT + CAMERA_HEIGHT + ship.y * CAMERA_FOLLOW_Y,
        ship.z + CAMERA_DISTANCE + (speed_scale - 1.0) * CAMERA_BOOST_DOLLY + cam_kick,
    ) + shake_offset;

    if let Some(transform) = world.core.get_local_transform_mut(camera) {
        transform.translation =
            approach_vec3(transform.translation, target, CAMERA_RESPONSE * delta);
        let pitch = nalgebra_glm::quat_angle_axis(CAMERA_PITCH, &Vec3::new(1.0, 0.0, 0.0));
        let roll_quat =
            nalgebra_glm::quat_angle_axis(roll * CAMERA_ROLL, &Vec3::new(0.0, 0.0, 1.0));
        transform.rotation = roll_quat * pitch;
    }
    mark_local_transform_dirty(world, camera);

    if let Some(component) = world.core.get_camera_mut(camera)
        && let Projection::Perspective(ref mut perspective) = component.projection
    {
        let target_fov =
            (BASE_FOV_DEGREES + (speed_scale - 1.0) * BOOST_FOV_DEGREES + fov_pop).to_radians();
        perspective.y_fov_rad = approach(perspective.y_fov_rad, target_fov, FOV_RESPONSE * delta);
    }
}
