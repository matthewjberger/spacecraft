use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

pub const FIGHTER_MESH: &str = "drift_fighter";
pub const MONARCH_MESH: &str = "monarch";

pub fn register_enemy_meshes(world: &mut World) {
    let (vertices, indices) = build_fighter();
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        FIGHTER_MESH.to_string(),
        Mesh::new(vertices, indices),
    );
    let (vertices, indices) = build_monarch();
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        MONARCH_MESH.to_string(),
        Mesh::new(vertices, indices),
    );
}

fn flat_faces(faces: &[[Vec3; 3]]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(faces.len() * 3);
    let mut indices: Vec<u32> = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        let edge_one = face[1] - face[0];
        let edge_two = face[2] - face[0];
        let normal = edge_one.cross(&edge_two).normalize();
        for corner in face {
            let base = vertices.len() as u32;
            let uv = [0.5 + corner.x * 0.3, 0.5 + corner.y * 0.3];
            vertices.push(Vertex::with_tex_coords(*corner, normal, uv));
            indices.push(base);
        }
    }
    (vertices, indices)
}

fn build_fighter() -> (Vec<Vertex>, Vec<u32>) {
    let nose = Vec3::new(0.0, 0.0, -1.7);
    let tail = Vec3::new(0.0, 0.0, 0.9);
    let left = Vec3::new(-1.35, 0.0, 0.2);
    let right = Vec3::new(1.35, 0.0, 0.2);
    let top = Vec3::new(0.0, 0.5, 0.2);
    let bottom = Vec3::new(0.0, -0.5, 0.2);

    let faces = [
        [nose, top, right],
        [nose, right, bottom],
        [nose, bottom, left],
        [nose, left, top],
        [tail, right, top],
        [tail, bottom, right],
        [tail, left, bottom],
        [tail, top, left],
    ];
    flat_faces(&faces)
}

fn spike(direction: Vec3, seed: f32) -> f32 {
    let base = (direction.x * 4.0 + seed).sin() * (direction.y * 3.0 - seed).cos() * 0.22;
    let ridge = (direction.z * 6.0 + seed * 1.7).sin() * 0.16;
    let crown = ((direction.y - 0.4).max(0.0)) * 0.7;
    base + ridge + crown
}

fn build_monarch() -> (Vec<Vertex>, Vec<u32>) {
    let stacks = 22usize;
    let sectors = 30usize;
    let stride = sectors + 1;
    let seed = 5.1;

    let mut positions: Vec<Vec3> = Vec::with_capacity((stacks + 1) * stride);
    for stack in 0..=stacks {
        let phi = std::f32::consts::PI * stack as f32 / stacks as f32;
        for sector in 0..=sectors {
            let theta = std::f32::consts::TAU * sector as f32 / sectors as f32;
            let direction = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            let radius = (1.0 + spike(direction, seed)).max(0.55);
            positions.push(Vec3::new(direction.x, direction.y * 0.82, direction.z) * radius);
        }
    }

    let mut indices: Vec<u32> = Vec::new();
    for stack in 0..stacks {
        for sector in 0..sectors {
            let top_left = (stack * stride + sector) as u32;
            let top_right = (stack * stride + sector + 1) as u32;
            let bottom_left = ((stack + 1) * stride + sector) as u32;
            let bottom_right = ((stack + 1) * stride + sector + 1) as u32;
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    let mut normals = vec![Vec3::zeros(); positions.len()];
    for triangle in indices.chunks_exact(3) {
        let index0 = triangle[0] as usize;
        let index1 = triangle[1] as usize;
        let index2 = triangle[2] as usize;
        let edge_one = positions[index1] - positions[index0];
        let edge_two = positions[index2] - positions[index0];
        let face_normal = edge_one.cross(&edge_two);
        normals[index0] += face_normal;
        normals[index1] += face_normal;
        normals[index2] += face_normal;
    }

    let mut vertices: Vec<Vertex> = Vec::with_capacity(positions.len());
    for (index, position) in positions.iter().enumerate() {
        let mut normal = if normals[index].magnitude() > 1.0e-5 {
            normals[index].normalize()
        } else {
            position.normalize()
        };
        if normal.dot(position) < 0.0 {
            normal = -normal;
        }
        let uv = [0.5 + position.x * 0.2, 0.5 + position.y * 0.2];
        vertices.push(Vertex::with_tex_coords(*position, normal, uv));
    }

    (vertices, indices)
}
