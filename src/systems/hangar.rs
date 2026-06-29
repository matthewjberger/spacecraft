use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

const DAIS_MESH: &str = "hangar_dais";

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
    let dais_visible = matches!(
        game.mode,
        GameMode::Title | GameMode::Settings | GameMode::LevelSelect
    );
    let elapsed = game.elapsed;

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
