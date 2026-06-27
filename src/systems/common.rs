use nightshade::prelude::*;

pub const SHIP_SCALE: f32 = 0.42;
pub const SHIP_BASE_YAW: f32 = std::f32::consts::PI;
pub const SHIP_STEER_SPEED: f32 = 9.0;
pub const BOUND_X: f32 = 6.5;
pub const BOUND_Y: f32 = 3.8;
pub const BASE_HEIGHT: f32 = 0.0;

pub const MAX_BANK: f32 = 0.85;
pub const MAX_PITCH: f32 = 0.40;
pub const MAX_LEAD_YAW: f32 = 0.25;
pub const ORIENT_RESPONSE: f32 = 9.0;
pub const IDLE_BOB: f32 = 0.12;

pub const BARREL_DURATION: f32 = 0.55;

pub const BOOST_GAIN: f32 = 1.4;
pub const BRAKE_GAIN: f32 = 0.5;
pub const SPEED_RESPONSE: f32 = 4.0;

pub const STICK_DEADZONE: f32 = 0.15;

pub const CAMERA_DISTANCE: f32 = 9.0;
pub const CAMERA_HEIGHT: f32 = 1.4;
pub const CAMERA_FOLLOW_X: f32 = 0.3;
pub const CAMERA_FOLLOW_Y: f32 = 0.25;
pub const CAMERA_RESPONSE: f32 = 5.0;
pub const CAMERA_PITCH: f32 = -0.155;
pub const CAMERA_ROLL: f32 = 0.30;
pub const BASE_FOV_DEGREES: f32 = 60.0;
pub const BOOST_FOV_DEGREES: f32 = 18.0;

pub const RAIL_SPEED: f32 = 38.0;
pub const SCENERY_SPAWN_DISTANCE: f32 = 300.0;
pub const SCENERY_DESPAWN_Z: f32 = 16.0;
pub const RING_RADIUS: f32 = 2.0;
pub const RING_SPACING: f32 = 15.0;
pub const PATTERN_GAP: f32 = 24.0;
pub const COURSE_START_Z: f32 = -24.0;
pub const ASTEROID_FIELD_X: f32 = 46.0;
pub const ASTEROID_FIELD_Y: f32 = 28.0;

pub const STAR_SPEED: f32 = 68.0;
pub const STARFIELD_CENTER_Z: f32 = -70.0;
pub const STARFIELD_HALF_X: f32 = 38.0;
pub const STARFIELD_HALF_Y: f32 = 24.0;
pub const STARFIELD_HALF_Z: f32 = 95.0;
pub const STARFIELD_RATE: f32 = 720.0;
pub const STAR_SIZE: f32 = 0.05;

pub const BURST_LIFETIME: f32 = 2.5;

pub const FIRE_INTERVAL: f32 = 0.11;
pub const PROJECTILE_SPEED: f32 = 95.0;
pub const PROJECTILE_RANGE: f32 = 175.0;
pub const TURRET_OFFSET_X: f32 = 1.05;
pub const TURRET_OFFSET_Y: f32 = -0.05;
pub const TURRET_OFFSET_Z: f32 = -0.9;
pub const PROJECTILE_HIT_RADIUS: f32 = 0.7;

pub fn approach(current: f32, target: f32, rate: f32) -> f32 {
    current + (target - current) * rate.clamp(0.0, 1.0)
}

pub fn approach_vec3(current: Vec3, target: Vec3, rate: f32) -> Vec3 {
    current + (target - current) * rate.clamp(0.0, 1.0)
}

pub fn next_random(state: &mut u64) -> f32 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    *state = value;
    ((value >> 40) as f32) / ((1u64 << 24) as f32)
}

pub fn random_range(state: &mut u64, low: f32, high: f32) -> f32 {
    low + (high - low) * next_random(state)
}

pub fn apply_material(
    world: &mut World,
    entity: Entity,
    name: &str,
    base_color: [f32; 4],
    emissive_factor: [f32; 3],
    unlit: bool,
    double_sided: bool,
) {
    let material = Material {
        base_color,
        emissive_factor,
        unlit,
        double_sided,
        metallic: 0.2,
        roughness: 0.85,
        ..Default::default()
    };
    register_material(world, entity, name.to_string(), material);
}

pub fn spawn_burst(world: &mut World, position: Vec3, color: Vec3, count: u32) -> Entity {
    let entity = spawn_entities(world, PARTICLE_EMITTER | NAME, 1)[0];
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.25 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: count,
        particle_lifetime_min: 0.35,
        particle_lifetime_max: 0.8,
        initial_velocity_min: 2.5,
        initial_velocity_max: 7.0,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::zeros(),
        drag: 0.35,
        size_start: 0.12,
        size_end: 0.01,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, vec4(color.x, color.y, color.z, 1.0)),
                (0.5, vec4(color.x, color.y, color.z, 0.7)),
                (1.0, vec4(color.x * 0.4, color.y * 0.4, color.z * 0.4, 0.0)),
            ],
        },
        emissive_strength: 3.0,
        one_shot: true,
        enabled: true,
        ..Default::default()
    };
    world.core.set_particle_emitter(entity, emitter);
    entity
}
