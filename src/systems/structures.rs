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
    let length = random_range(&mut game.random_state, 24.0, 46.0);
    let width = random_range(&mut game.random_state, 3.6, 5.6);
    let height = width * 0.82;
    let warm = next_random(&mut game.random_state) < 0.45;
    let window = if warm {
        [1.9, 0.85, 0.2]
    } else {
        [0.3, 1.1, 1.8]
    };

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::zeros(),
        Vec3::new(width, height, length),
        hull_material(),
    );

    let ribs = 5;
    for rib in 0..ribs {
        let along = ((rib as f32 + 0.5) / ribs as f32 - 0.5) * length;
        add_part(
            world,
            &mut parts,
            "Cube",
            Vec3::new(0.0, 0.0, along),
            Vec3::new(width * 1.14, height * 1.14, length * 0.035),
            hull_material(),
        );
    }

    let strip = Vec3::new(0.2, height * 0.5, length * 0.88);
    for side in [-1.0_f32, 1.0] {
        add_part(
            world,
            &mut parts,
            "Cube",
            Vec3::new(side * width * 0.52, 0.0, 0.0),
            strip,
            window_material(window, 3.0),
        );
    }

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::new(0.0, height * 0.55, length * 0.04),
        Vec3::new(width * 0.42, 0.16, length * 0.62),
        window_material(window, 2.4),
    );

    add_part(
        world,
        &mut parts,
        "Cube",
        Vec3::new(0.0, height * 0.14, length * 0.52),
        Vec3::new(width * 1.28, height * 1.1, length * 0.12),
        hull_material(),
    );

    add_part(
        world,
        &mut parts,
        "Cylinder",
        Vec3::new(0.0, 0.0, -length * 0.52),
        Vec3::new(width * 1.5, width * 1.5, 0.6),
        hull_material(),
    );

    add_part(
        world,
        &mut parts,
        "Cylinder",
        Vec3::new(width * 0.3, height * 1.05, -length * 0.18),
        Vec3::new(0.14, height * 1.5, 0.14),
        hull_material(),
    );

    let spin_axis = Vec3::new(
        random_range(&mut game.random_state, -0.2, 0.2),
        1.0,
        random_range(&mut game.random_state, -0.2, 0.2),
    )
    .normalize();
    game.structures.push(Structure {
        parts,
        position,
        spin_axis,
        spin_speed: random_range(&mut game.random_state, 0.0, 0.05),
        angle: random_range(&mut game.random_state, 0.0, std::f32::consts::TAU),
        drift: Vec3::zeros(),
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
        let bend = course_bend(game, position);
        let rotation = nalgebra_glm::quat_angle_axis(
            game.structures[index].angle,
            &game.structures[index].spin_axis,
        );
        for (entity, offset, scale) in &game.structures[index].parts {
            let world_pos = position + bend + nalgebra_glm::quat_rotate_vec3(&rotation, offset);
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
