use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

#[derive(Default)]
struct InputFrame {
    steer_x: f32,
    steer_y: f32,
    boost: f32,
    brake: f32,
    roll_left: bool,
    roll_right: bool,
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let active = game_world.resources.game.mode == GameMode::Playing;
    let frame = if active {
        read_input(world)
    } else {
        InputFrame::default()
    };
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let Some(ship) = game.ship else {
        return;
    };

    game.elapsed += delta;

    if game.barrel.timer <= 0.0 {
        if frame.roll_left {
            game.barrel.timer = BARREL_DURATION;
            game.barrel.direction = 1.0;
        } else if frame.roll_right {
            game.barrel.timer = BARREL_DURATION;
            game.barrel.direction = -1.0;
        }
    }
    let mut barrel_angle = 0.0;
    if game.barrel.timer > 0.0 {
        game.barrel.timer -= delta;
        let progress = (1.0 - game.barrel.timer / BARREL_DURATION).clamp(0.0, 1.0);
        barrel_angle = game.barrel.direction * std::f32::consts::TAU * progress;
    }

    game.ship_position.x =
        (game.ship_position.x + frame.steer_x * SHIP_STEER_SPEED * delta).clamp(-BOUND_X, BOUND_X);
    game.ship_position.y = (game.ship_position.y + frame.steer_y * SHIP_STEER_SPEED * delta)
        .clamp(BASE_HEIGHT - BOUND_Y, BASE_HEIGHT + BOUND_Y);

    game.roll = approach(
        game.roll,
        -frame.steer_x * MAX_BANK,
        ORIENT_RESPONSE * delta,
    );
    game.pitch = approach(
        game.pitch,
        frame.steer_y * MAX_PITCH,
        ORIENT_RESPONSE * delta,
    );

    let target_speed = 1.0 + frame.boost * BOOST_GAIN - frame.brake * BRAKE_GAIN;
    game.speed_scale = approach(game.speed_scale, target_speed, SPEED_RESPONSE * delta);

    let bob = (game.elapsed * 1.7).sin() * IDLE_BOB;
    let lead_yaw = frame.steer_x * MAX_LEAD_YAW;
    let position = Vec3::new(
        game.ship_position.x,
        game.ship_position.y + bob,
        game.ship_position.z,
    );

    let base = nalgebra_glm::quat_angle_axis(SHIP_BASE_YAW, &Vec3::new(0.0, 1.0, 0.0));
    let yaw = nalgebra_glm::quat_angle_axis(lead_yaw, &Vec3::new(0.0, 1.0, 0.0));
    let pitch = nalgebra_glm::quat_angle_axis(game.pitch, &Vec3::new(1.0, 0.0, 0.0));
    let roll = nalgebra_glm::quat_angle_axis(game.roll + barrel_angle, &Vec3::new(0.0, 0.0, 1.0));
    let rotation = yaw * pitch * roll * base;

    if let Some(transform) = world.core.get_local_transform_mut(ship) {
        transform.translation = position;
        transform.rotation = rotation;
        transform.scale = Vec3::new(SHIP_SCALE, SHIP_SCALE, SHIP_SCALE);
    }
    mark_local_transform_dirty(world, ship);

    let speed_scale = game.speed_scale;
    let exhaust_dir = nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, 0.0, -1.0));
    if let Some(exhaust) = game.exhaust {
        let tail =
            position + nalgebra_glm::quat_rotate_vec3(&rotation, &Vec3::new(0.0, -0.1, -1.4));
        if let Some(emitter) = world.core.get_particle_emitter_mut(exhaust) {
            emitter.position = tail;
            emitter.direction = exhaust_dir;
            emitter.spawn_rate = 520.0 + speed_scale * 360.0;
        }
    }

    let thrust = (speed_scale - 1.0).max(0.0);
    let corner_rate = 240.0 + thrust * 420.0;
    let corner_offsets = [
        Vec3::new(-1.9, 0.4, -1.25),
        Vec3::new(-1.9, -0.18, -1.25),
        Vec3::new(1.9, 0.4, -1.25),
        Vec3::new(1.9, -0.18, -1.25),
    ];
    for (slot, offset) in corner_offsets.iter().enumerate() {
        let port = position + nalgebra_glm::quat_rotate_vec3(&rotation, offset);
        game.blaster_ports[slot] = port;
        if let Some(&thruster) = game.corner_thrusters.get(slot)
            && let Some(emitter) = world.core.get_particle_emitter_mut(thruster)
        {
            emitter.position = port;
            emitter.direction = exhaust_dir;
            emitter.spawn_rate = corner_rate;
        }
    }
}

fn read_input(world: &mut World) -> InputFrame {
    let mut frame = InputFrame::default();
    {
        let keyboard = &world.resources.input.keyboard;
        if keyboard.is_key_pressed(KeyCode::KeyA) || keyboard.is_key_pressed(KeyCode::ArrowLeft) {
            frame.steer_x -= 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyD) || keyboard.is_key_pressed(KeyCode::ArrowRight) {
            frame.steer_x += 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyW) || keyboard.is_key_pressed(KeyCode::ArrowUp) {
            frame.steer_y += 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::KeyS) || keyboard.is_key_pressed(KeyCode::ArrowDown) {
            frame.steer_y -= 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::ShiftLeft) {
            frame.boost = 1.0;
        }
        if keyboard.is_key_pressed(KeyCode::ControlLeft) {
            frame.brake = 1.0;
        }
        if keyboard.just_pressed(KeyCode::KeyQ) {
            frame.roll_left = true;
        }
        if keyboard.just_pressed(KeyCode::KeyE) {
            frame.roll_right = true;
        }
    }
    {
        let gamepad = &world.resources.input.gamepad;
        if gamepad.just_pressed(gilrs::Button::LeftTrigger) {
            frame.roll_left = true;
        }
        if gamepad.just_pressed(gilrs::Button::RightTrigger) {
            frame.roll_right = true;
        }
    }
    if let Some(gamepad) = query_active_gamepad(world) {
        let stick_x = gamepad.value(gilrs::Axis::LeftStickX);
        let stick_y = gamepad.value(gilrs::Axis::LeftStickY);
        let magnitude = (stick_x * stick_x + stick_y * stick_y).sqrt();
        if magnitude > STICK_DEADZONE {
            let scaled =
                ((magnitude - STICK_DEADZONE) / (1.0 - STICK_DEADZONE)).clamp(0.0, 1.0) / magnitude;
            frame.steer_x += stick_x * scaled;
            frame.steer_y += stick_y * scaled;
        }
        if gamepad.is_pressed(gilrs::Button::RightTrigger2) {
            frame.boost = 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::LeftTrigger2) {
            frame.brake = 1.0;
        }
    }
    frame.steer_x = frame.steer_x.clamp(-1.0, 1.0);
    frame.steer_y = frame.steer_y.clamp(-1.0, 1.0);
    frame
}
