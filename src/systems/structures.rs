use crate::ecs::{GameState, Structure, TemplateWorld};
use crate::systems::common::*;
use nightshade::prelude::*;

fn hull_material() -> Material {
    Material {
        base_color: [0.22, 0.24, 0.3, 1.0],
        emissive_factor: [0.03, 0.04, 0.06],
        emissive_strength: 1.0,
        metallic: 0.72,
        roughness: 0.4,
        ..Default::default()
    }
}

fn window_material(color: [f32; 3], strength: f32) -> Material {
    Material {
        base_color: [0.05, 0.05, 0.07, 1.0],
        emissive_factor: color,
        emissive_strength: strength,
        metallic: 0.0,
        roughness: 0.6,
        ..Default::default()
    }
}

fn add_part(
    world: &mut World,
    parts: &mut Vec<(Entity, Vec3, Vec3)>,
    mesh: &str,
    offset: Vec3,
    scale: Vec3,
    material: Material,
) {
    let entity = spawn_mesh(world, mesh, offset, scale);
    register_material(world, entity, format!("derelict_{}", entity.id), material);
    parts.push((entity, offset, scale));
}

pub fn spawn_derelict(world: &mut World, game: &mut GameState, position: Vec3) {
    let mut parts: Vec<(Entity, Vec3, Vec3)> = Vec::new();
    let length = random_range(&mut game.random_state, 3.6, 7.2);
    let warm = next_random(&mut game.random_state) < 0.45;
    let window = if warm {
        [1.6, 0.7, 0.18]
    } else {
        [0.25, 0.95, 1.5]
    };

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::zeros(),
        Vec3::new(1.2, 1.0, length),
        hull_material(),
    );

    let strip = Vec3::new(0.08, 0.42, length * 0.82);
    for side in [-1.0_f32, 1.0] {
        add_part(
            world,
            &mut parts,
            "Cube",
            Vec3::new(side * 1.22, 0.05, 0.0),
            strip,
            window_material(window, 2.6),
        );
    }

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::new(0.0, 1.02, length * 0.12),
        Vec3::new(0.85, 0.07, length * 0.5),
        window_material(window, 2.2),
    );

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::new(0.0, 0.0, length * 0.55),
        Vec3::new(1.5, 1.22, 0.55),
        hull_material(),
    );

    add_part(
        world,
        &mut parts,
        "Cylinder",
        Vec3::new(0.0, 0.0, -length * 0.55),
        Vec3::new(1.7, 1.7, 0.22),
        hull_material(),
    );

    add_part(
        world,
        &mut parts,
        "Cylinder",
        Vec3::new(0.45, 1.25, -length * 0.28),
        Vec3::new(0.05, 1.25, 0.05),
        hull_material(),
    );

    let spin_axis = Vec3::new(
        random_range(&mut game.random_state, -1.0, 1.0),
        random_range(&mut game.random_state, -1.0, 1.0),
        random_range(&mut game.random_state, -1.0, 1.0),
    )
    .normalize();
    game.structures.push(Structure {
        parts,
        position,
        spin_axis,
        spin_speed: random_range(&mut game.random_state, 0.08, 0.4),
        angle: random_range(&mut game.random_state, 0.0, std::f32::consts::TAU),
        drift: Vec3::new(
            random_range(&mut game.random_state, -1.0, 1.0),
            random_range(&mut game.random_state, -0.5, 0.5),
            0.0,
        ),
    });
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let rail = RAIL_SPEED * game.speed_scale;

    let mut remove: Vec<usize> = Vec::new();
    for index in 0..game.structures.len() {
        game.structures[index].angle += game.structures[index].spin_speed * delta;
        let drift = game.structures[index].drift;
        game.structures[index].position += drift * delta;
        game.structures[index].position.z += rail * delta;

        let position = game.structures[index].position;
        let rotation = nalgebra_glm::quat_angle_axis(
            game.structures[index].angle,
            &game.structures[index].spin_axis,
        );
        for (entity, offset, scale) in &game.structures[index].parts {
            let world_pos = position + nalgebra_glm::quat_rotate_vec3(&rotation, offset);
            if let Some(transform) = world.core.get_local_transform_mut(*entity) {
                transform.translation = world_pos;
                transform.rotation = rotation;
                transform.scale = *scale;
            }
            mark_local_transform_dirty(world, *entity);
        }

        if position.z > SCENERY_DESPAWN_Z + 24.0 {
            remove.push(index);
        }
    }

    for index in remove.into_iter().rev() {
        let structure = game.structures.remove(index);
        for (entity, _, _) in structure.parts {
            despawn_recursive_immediate(world, entity);
        }
    }
}

pub fn clear(world: &mut World, game: &mut GameState) {
    for structure in game.structures.drain(..) {
        for (entity, _, _) in structure.parts {
            despawn_recursive_immediate(world, entity);
        }
    }
}
