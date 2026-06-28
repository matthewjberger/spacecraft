use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use crate::systems::textures::proto_material;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

const DAIS_MESH: &str = "hangar_dais";

pub fn spawn_room(world: &mut World) -> Vec<(Entity, Vec3, Vec3)> {
    let mut parts: Vec<(Entity, Vec3, Vec3)> = Vec::new();
    let base = BASE_HEIGHT;

    add_surface(
        world,
        &mut parts,
        Vec3::new(0.0, base - 2.2, -7.0),
        Vec3::new(34.0, 0.6, 52.0),
        "proto_dark_06",
        Vec3::new(0.66, 0.72, 0.82),
        9.0,
    );
    add_surface(
        world,
        &mut parts,
        Vec3::new(0.0, base + 12.0, -7.0),
        Vec3::new(34.0, 0.6, 52.0),
        "proto_dark_03",
        Vec3::new(0.42, 0.46, 0.58),
        9.0,
    );
    add_surface(
        world,
        &mut parts,
        Vec3::new(0.0, base + 4.8, -31.0),
        Vec3::new(34.0, 29.0, 0.6),
        "proto_light_01",
        Vec3::new(0.82, 0.86, 0.96),
        7.0,
    );
    add_surface(
        world,
        &mut parts,
        Vec3::new(-17.0, base + 4.8, -7.0),
        Vec3::new(0.6, 29.0, 52.0),
        "proto_light_03",
        Vec3::new(0.74, 0.8, 0.92),
        7.0,
    );
    add_surface(
        world,
        &mut parts,
        Vec3::new(17.0, base + 4.8, -7.0),
        Vec3::new(0.6, 29.0, 52.0),
        "proto_light_03",
        Vec3::new(0.74, 0.8, 0.92),
        7.0,
    );

    add_trim(
        world,
        &mut parts,
        Vec3::new(-16.6, base - 1.6, -7.0),
        Vec3::new(0.4, 0.4, 52.0),
        [0.5, 0.2, 1.2],
    );
    add_trim(
        world,
        &mut parts,
        Vec3::new(16.6, base - 1.6, -7.0),
        Vec3::new(0.4, 0.4, 52.0),
        [0.5, 0.2, 1.2],
    );
    add_trim(
        world,
        &mut parts,
        Vec3::new(0.0, base - 1.6, -30.6),
        Vec3::new(34.0, 0.4, 0.4),
        [1.3, 0.6, 0.15],
    );
    parts
}

fn add_surface(
    world: &mut World,
    parts: &mut Vec<(Entity, Vec3, Vec3)>,
    position: Vec3,
    scale: Vec3,
    texture: &str,
    tint: Vec3,
    tiling: f32,
) {
    let entity = spawn_mesh(world, "Cube", position, scale);
    let name = format!("hangar_{}", entity.id);
    register_material(world, entity, name, proto_material(texture, tint, tiling));
    parts.push((entity, position, scale));
}

fn add_trim(
    world: &mut World,
    parts: &mut Vec<(Entity, Vec3, Vec3)>,
    position: Vec3,
    scale: Vec3,
    emissive: [f32; 3],
) {
    let entity = spawn_mesh(world, "Cube", position, scale);
    let name = format!("hangar_trim_{}", entity.id);
    let material = Material {
        base_color: [0.03, 0.03, 0.05, 1.0],
        emissive_factor: emissive,
        emissive_strength: 2.6,
        metallic: 0.0,
        roughness: 0.5,
        ..Default::default()
    };
    register_material(world, entity, name, material);
    parts.push((entity, position, scale));
}

pub fn register_mesh(world: &mut World) {
    let (vertices, indices) = build_dais();
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        DAIS_MESH.to_string(),
        Mesh::new(vertices, indices),
    );
}

pub fn spawn(world: &mut World) -> Entity {
    let entity = spawn_mesh(
        world,
        DAIS_MESH,
        Vec3::new(0.0, BASE_HEIGHT - 1.2, -1.5),
        Vec3::zeros(),
    );
    apply_material(
        world,
        entity,
        "dais",
        [0.16, 0.5, 0.72, 1.0],
        [0.12, 0.5, 0.85],
        false,
        true,
    );
    entity
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let game = &game_world.resources.game;
    let room_visible = game.mode == GameMode::Shop;
    let dais_visible = matches!(
        game.mode,
        GameMode::Title | GameMode::Settings | GameMode::LevelSelect | GameMode::Shop
    );
    let elapsed = game.elapsed;

    for index in 0..game.hangar_parts.len() {
        let (entity, position, scale) = game.hangar_parts[index];
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.scale = if room_visible { scale } else { Vec3::zeros() };
        }
        mark_local_transform_dirty(world, entity);
    }

    let Some(dais) = game.dais else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(dais) {
        transform.translation = Vec3::new(0.0, BASE_HEIGHT - 1.2, -1.5);
        if dais_visible {
            let pulse = 4.7 + (elapsed * 1.5).sin() * 0.15;
            transform.scale = Vec3::new(pulse, pulse, pulse);
            transform.rotation =
                nalgebra_glm::quat_angle_axis(elapsed * 0.2, &Vec3::new(0.0, 1.0, 0.0));
        } else {
            transform.scale = Vec3::zeros();
        }
    }
    mark_local_transform_dirty(world, dais);
}

fn build_dais() -> (Vec<Vertex>, Vec<u32>) {
    let segments = 56usize;
    let inner = 0.74;
    let outer = 1.0;
    let mut vertices: Vec<Vertex> = Vec::with_capacity((segments + 1) * 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 6);
    for segment in 0..=segments {
        let angle = std::f32::consts::TAU * segment as f32 / segments as f32;
        let cos = angle.cos();
        let sin = angle.sin();
        vertices.push(Vertex::with_tex_coords(
            Vec3::new(cos * inner, 0.0, sin * inner),
            Vec3::new(0.0, 1.0, 0.0),
            [0.0, 0.0],
        ));
        vertices.push(Vertex::with_tex_coords(
            Vec3::new(cos * outer, 0.0, sin * outer),
            Vec3::new(0.0, 1.0, 0.0),
            [1.0, 0.0],
        ));
    }
    for segment in 0..segments {
        let base = (segment * 2) as u32;
        indices.extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }
    (vertices, indices)
}
