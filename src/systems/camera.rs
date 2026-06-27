use crate::ecs::TemplateWorld;
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &game_world.resources.game;
    let Some(camera) = game.camera else {
        return;
    };
    let ship = game.ship_position;
    let roll = game.roll;
    let speed_scale = game.speed_scale;
    let shake = game.shake;
    let elapsed = game.elapsed;

    let shake_offset = Vec3::new(
        (elapsed * 92.0).sin() * shake * 0.16,
        (elapsed * 71.0).cos() * shake * 0.16,
        0.0,
    );
    let target = Vec3::new(
        ship.x * CAMERA_FOLLOW_X,
        BASE_HEIGHT + CAMERA_HEIGHT + ship.y * CAMERA_FOLLOW_Y,
        ship.z + CAMERA_DISTANCE,
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
        let target_fov = BASE_FOV_DEGREES + (speed_scale - 1.0) * BOOST_FOV_DEGREES;
        perspective.y_fov_rad = target_fov.to_radians();
    }
}
