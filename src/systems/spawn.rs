use nightshade::prelude::*;

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
