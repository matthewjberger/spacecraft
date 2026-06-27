use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

pub const ASTEROID_VARIANTS: usize = 4;

pub fn asteroid_name(variant: usize) -> String {
    format!("asteroid_{variant}")
}

pub fn register_asteroid_meshes(world: &mut World) {
    for variant in 0..ASTEROID_VARIANTS {
        let seed = 1.7 + variant as f32 * 3.9;
        let (vertices, indices) = build_asteroid(seed, 0.36);
        let mesh = Mesh::new(vertices, indices);
        mesh_cache_insert(
            &mut world.resources.assets.mesh_cache,
            asteroid_name(variant),
            mesh,
        );
    }
}

fn lump(direction: Vec3, seed: f32) -> f32 {
    let x = direction.x;
    let y = direction.y;
    let z = direction.z;
    let mut value = 0.0;
    value += (x * 3.1 + seed).sin() * (y * 2.7 - seed * 1.3).cos() * 0.5;
    value += (y * 5.3 + seed * 1.7).sin() * (z * 4.1 - seed).cos() * 0.28;
    value += (z * 7.7 - seed * 0.7).sin() * (x * 6.3 + seed).cos() * 0.16;
    value += (x * 11.0 + y * 9.0 - z * 8.0 + seed).sin() * 0.08;
    value
}

fn build_asteroid(seed: f32, roughness: f32) -> (Vec<Vertex>, Vec<u32>) {
    let stacks = 18usize;
    let sectors = 28usize;
    let stride = sectors + 1;

    let mut positions: Vec<Vec3> = Vec::with_capacity((stacks + 1) * stride);
    for stack in 0..=stacks {
        let phi = std::f32::consts::PI * stack as f32 / stacks as f32;
        for sector in 0..=sectors {
            let theta = std::f32::consts::TAU * sector as f32 / sectors as f32;
            let direction = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            let radius = (1.0 + lump(direction, seed) * roughness).max(0.5);
            positions.push(direction * radius);
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
        let edge1 = positions[index1] - positions[index0];
        let edge2 = positions[index2] - positions[index0];
        let face_normal = edge1.cross(&edge2);
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
        let uv = [0.5 + position.x * 0.25, 0.5 + position.z * 0.25];
        vertices.push(Vertex::with_tex_coords(*position, normal, uv));
    }

    (vertices, indices)
}
