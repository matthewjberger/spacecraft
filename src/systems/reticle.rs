use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

const RETICLE_MESH: &str = "reticle";

pub fn register_mesh(world: &mut World) {
    let (vertices, indices) = build_reticle();
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        RETICLE_MESH.to_string(),
        Mesh::new(vertices, indices),
    );
}

pub fn spawn(world: &mut World) -> Entity {
    let entity = spawn_mesh(world, RETICLE_MESH, Vec3::zeros(), Vec3::zeros());
    apply_material(
        world,
        entity,
        "reticle",
        [0.3, 1.2, 0.6, 1.0],
        [0.25, 1.1, 0.55],
        true,
        true,
    );
    entity
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let game = &game_world.resources.game;
    let visible = game.mode == GameMode::Playing;
    let ship = game.ship_position;
    let lead_x = -game.roll / MAX_BANK;
    let lead_y = game.pitch / MAX_PITCH;
    let near = game.reticle_near;
    let far = game.reticle_far;

    place(
        world,
        near,
        Vec3::new(ship.x + lead_x * 1.4, ship.y + lead_y * 1.2, ship.z - 16.0),
        0.55,
        visible,
    );
    place(
        world,
        far,
        Vec3::new(ship.x + lead_x * 3.6, ship.y + lead_y * 3.0, ship.z - 44.0),
        1.2,
        visible,
    );
}

fn place(world: &mut World, entity: Option<Entity>, position: Vec3, scale: f32, visible: bool) {
    let Some(entity) = entity else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.translation = position;
        transform.scale = if visible {
            Vec3::new(scale, scale, scale)
        } else {
            Vec3::zeros()
        };
    }
    mark_local_transform_dirty(world, entity);
}

fn build_reticle() -> (Vec<Vertex>, Vec<u32>) {
    let segments = 8usize;
    let inner = 0.7;
    let outer = 1.0;
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    for segment in 0..segments {
        let span = std::f32::consts::TAU / segments as f32;
        let gap = span * 0.32;
        let start = span * segment as f32 + gap * 0.5;
        let end = span * (segment as f32 + 1.0) - gap * 0.5;
        let base = vertices.len() as u32;
        for &(radius, angle) in &[(inner, start), (outer, start), (inner, end), (outer, end)] {
            vertices.push(Vertex::with_tex_coords(
                Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0),
                normal,
                [0.0, 0.0],
            ));
        }
        indices.extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }
    (vertices, indices)
}
