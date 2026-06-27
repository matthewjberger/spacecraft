use crate::ecs::TemplateWorld;
use crate::systems::asteroid_mesh;
use crate::systems::backdrop;
use crate::systems::common::*;
use crate::systems::enemy_mesh;
use nightshade::ecs::light::components::{Light, LightType};
use nightshade::prelude::*;

const SHIP_BYTES: &[u8] = include_bytes!("../../assets/gltf/Spitfire.glb");

pub fn build(game_world: &mut TemplateWorld, world: &mut World) {
    world.resources.window.title = "Spacecraft".to_string();
    world.resources.render_settings.atmosphere = Atmosphere::Space;
    world.resources.render_settings.bloom_enabled = true;
    world.resources.render_settings.bloom_intensity = 0.85;
    world.resources.render_settings.ambient_light = [0.05, 0.07, 0.13, 1.0];
    world.resources.debug_draw.show_grid = false;

    let grading = &mut world.resources.render_settings.color_grading;
    grading.saturation = 1.15;
    grading.contrast = 1.06;

    let sun = spawn_sun(world);
    if let Some(light) = world.core.get_light_mut(sun) {
        light.color = Vec3::new(0.82, 0.88, 1.0);
        light.intensity = 5.5;
    }
    add_accent_light(
        world,
        Vec3::new(-9.0, 6.0, -2.0),
        Vec3::new(0.7, 0.3, 1.0),
        4.5,
        45.0,
    );
    add_accent_light(
        world,
        Vec3::new(9.0, -5.0, -6.0),
        Vec3::new(0.2, 0.7, 1.0),
        4.0,
        45.0,
    );

    asteroid_mesh::register_asteroid_meshes(world);
    enemy_mesh::register_enemy_meshes(world);

    let camera = spawn_camera(
        world,
        Vec3::new(0.0, CAMERA_HEIGHT, CAMERA_DISTANCE),
        "Chase Camera".to_string(),
    );
    if let Some(component) = world.core.get_camera_mut(camera) {
        component.smoothing = None;
        if let Projection::Perspective(ref mut perspective) = component.projection {
            perspective.y_fov_rad = BASE_FOV_DEGREES.to_radians();
            perspective.z_far = Some(2000.0);
        }
    }
    if let Some(transform) = world.core.get_local_transform_mut(camera) {
        transform.rotation = nalgebra_glm::quat_angle_axis(CAMERA_PITCH, &Vec3::new(1.0, 0.0, 0.0));
    }
    mark_local_transform_dirty(world, camera);
    world.resources.active_camera = Some(camera);

    let ship = load_ship(world);
    if let Some(entity) = ship {
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.scale = Vec3::new(SHIP_SCALE, SHIP_SCALE, SHIP_SCALE);
        }
        mark_local_transform_dirty(world, entity);
    }

    let exhaust = spawn_exhaust(world);
    let corner_thrusters = vec![
        spawn_corner_thruster(world),
        spawn_corner_thruster(world),
        spawn_corner_thruster(world),
        spawn_corner_thruster(world),
    ];
    let starfield = spawn_starfield(world);

    let uptime = world.resources.window.timing.uptime_milliseconds;
    let game = &mut game_world.resources.game;
    game.random_state ^= uptime | 1;
    game.camera = Some(camera);
    game.ship = ship;
    game.exhaust = Some(exhaust);
    game.corner_thrusters = corner_thrusters;
    game.starfield = Some(starfield);
    game.ship_position = Vec3::new(0.0, BASE_HEIGHT, 0.0);

    backdrop::spawn_backdrop(world, game);
}

fn load_ship(world: &mut World) -> Option<Entity> {
    let mut result = import_gltf_from_bytes(SHIP_BYTES).ok()?;
    nightshade::ecs::loading::queue_gltf_load(world, &mut result);
    let prefab = result.prefabs.first()?;
    Some(spawn_prefab_with_animations(
        world,
        prefab,
        &result.animations,
        Vec3::new(0.0, BASE_HEIGHT, 0.0),
    ))
}

fn spawn_exhaust(world: &mut World) -> Entity {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Fire,
        shape: EmitterShape::Point,
        position: Vec3::new(0.0, BASE_HEIGHT, 1.4),
        direction: Vec3::new(0.0, 0.0, 1.0),
        spawn_rate: 180.0,
        burst_count: 0,
        particle_lifetime_min: 0.22,
        particle_lifetime_max: 0.45,
        initial_velocity_min: 3.0,
        initial_velocity_max: 6.0,
        velocity_spread: 0.22,
        gravity: Vec3::zeros(),
        drag: 0.2,
        size_start: 0.32,
        size_end: 0.02,
        color_gradient: engine_flame_gradient(),
        emissive_strength: 6.0,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    entity
}

fn spawn_starfield(world: &mut World) -> Entity {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let lifetime = (STARFIELD_HALF_Z * 2.0 + CAMERA_DISTANCE) / STAR_SPEED;
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(STARFIELD_HALF_X, STARFIELD_HALF_Y, STARFIELD_HALF_Z),
        },
        position: Vec3::new(0.0, BASE_HEIGHT, STARFIELD_CENTER_Z),
        direction: Vec3::new(0.0, 0.0, 1.0),
        spawn_rate: STARFIELD_RATE,
        burst_count: 0,
        particle_lifetime_min: lifetime,
        particle_lifetime_max: lifetime,
        initial_velocity_min: STAR_SPEED,
        initial_velocity_max: STAR_SPEED,
        velocity_spread: 0.0,
        gravity: Vec3::zeros(),
        drag: 0.0,
        size_start: STAR_SIZE,
        size_end: STAR_SIZE,
        color_gradient: star_gradient(),
        emissive_strength: 2.5,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    entity
}

fn spawn_corner_thruster(world: &mut World) -> Entity {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Fire,
        shape: EmitterShape::Point,
        position: Vec3::zeros(),
        direction: Vec3::new(0.0, 0.0, 1.0),
        spawn_rate: 0.0,
        burst_count: 0,
        particle_lifetime_min: 0.08,
        particle_lifetime_max: 0.18,
        initial_velocity_min: 1.5,
        initial_velocity_max: 3.0,
        velocity_spread: 0.08,
        gravity: Vec3::zeros(),
        drag: 0.2,
        size_start: 0.13,
        size_end: 0.01,
        color_gradient: yellow_thruster_gradient(),
        emissive_strength: 5.5,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    entity
}

fn yellow_thruster_gradient() -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 1.0, 0.85, 1.0)),
            (0.35, vec4(1.0, 0.85, 0.3, 1.0)),
            (0.7, vec4(1.0, 0.55, 0.1, 0.6)),
            (1.0, vec4(0.5, 0.25, 0.0, 0.0)),
        ],
    }
}

fn add_accent_light(world: &mut World, position: Vec3, color: Vec3, intensity: f32, range: f32) {
    let entity = spawn_light_entity(world, position, "accent_light");
    world.core.set_light(
        entity,
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            cast_shadows: false,
            ..Default::default()
        },
    );
}

fn engine_flame_gradient() -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(0.8, 0.95, 1.0, 1.0)),
            (0.3, vec4(0.3, 0.6, 1.0, 0.9)),
            (0.7, vec4(0.1, 0.3, 0.9, 0.5)),
            (1.0, vec4(0.0, 0.1, 0.4, 0.0)),
        ],
    }
}

fn star_gradient() -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, vec4(1.0, 1.0, 1.0, 0.0)),
            (0.1, vec4(1.0, 1.0, 1.0, 1.0)),
            (0.85, vec4(0.85, 0.9, 1.0, 1.0)),
            (1.0, vec4(0.7, 0.8, 1.0, 0.0)),
        ],
    }
}
