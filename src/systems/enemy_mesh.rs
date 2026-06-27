use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

pub const DRONE_MESH: &str = "drift_drone";
pub const FIGHTER_MESH: &str = "drift_fighter";
pub const GUNSHIP_MESH: &str = "drift_gunship";
pub const HARVESTER_MESH: &str = "harvester";
pub const WARDEN_MESH: &str = "warden";
pub const SENTINEL_MESH: &str = "sentinel";
pub const MONARCH_MESH: &str = "monarch";

pub fn register_enemy_meshes(world: &mut World) {
    insert(world, DRONE_MESH, build_drone());
    insert(world, FIGHTER_MESH, build_fighter());
    insert(world, GUNSHIP_MESH, build_gunship());
    insert(world, HARVESTER_MESH, build_hulk(3.4, 0.5, 0.7));
    insert(world, WARDEN_MESH, build_hulk(8.7, 1.25, 0.85));
    insert(world, SENTINEL_MESH, build_hulk(2.2, 1.5, 0.62));
    insert(world, MONARCH_MESH, build_hulk(5.1, 0.82, 1.0));
}

fn insert(world: &mut World, name: &str, mesh: (Vec<Vertex>, Vec<u32>)) {
    mesh_cache_insert(
        &mut world.resources.assets.mesh_cache,
        name.to_string(),
        Mesh::new(mesh.0, mesh.1),
    );
}

fn flat_faces(faces: &[[Vec3; 3]]) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(faces.len() * 3);
    let mut indices: Vec<u32> = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        let edge_one = face[1] - face[0];
        let edge_two = face[2] - face[0];
        let mut normal = edge_one.cross(&edge_two).normalize();
        let centroid = (face[0] + face[1] + face[2]) / 3.0;
        if normal.dot(&centroid) < 0.0 {
            normal = -normal;
        }
        for corner in face {
            let base = vertices.len() as u32;
            let uv = [0.5 + corner.x * 0.3, 0.5 + corner.y * 0.3];
            vertices.push(Vertex::with_tex_coords(*corner, normal, uv));
            indices.push(base);
        }
    }
    (vertices, indices)
}

fn build_drone() -> (Vec<Vertex>, Vec<u32>) {
    let px = Vec3::new(1.2, 0.0, 0.0);
    let nx = Vec3::new(-1.2, 0.0, 0.0);
    let py = Vec3::new(0.0, 0.9, 0.0);
    let ny = Vec3::new(0.0, -0.9, 0.0);
    let pz = Vec3::new(0.0, 0.0, 1.2);
    let nz = Vec3::new(0.0, 0.0, -1.2);
    let faces = [
        [py, pz, px],
        [py, px, nz],
        [py, nz, nx],
        [py, nx, pz],
        [ny, px, pz],
        [ny, nz, px],
        [ny, nx, nz],
        [ny, pz, nx],
    ];
    flat_faces(&faces)
}

fn build_fighter() -> (Vec<Vertex>, Vec<u32>) {
    let nose = Vec3::new(0.0, 0.0, -1.7);
    let tail = Vec3::new(0.0, 0.0, 0.9);
    let left = Vec3::new(-1.35, 0.0, 0.2);
    let right = Vec3::new(1.35, 0.0, 0.2);
    let top = Vec3::new(0.0, 0.78, 0.1);
    let bottom = Vec3::new(0.0, -0.78, 0.1);
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

fn build_gunship() -> (Vec<Vertex>, Vec<u32>) {
    let nose = Vec3::new(0.0, 0.0, -1.5);
    let tail = Vec3::new(0.0, 0.0, 1.5);
    let segments = 6usize;
    let mut ring: Vec<Vec3> = Vec::with_capacity(segments);
    for segment in 0..segments {
        let angle = std::f32::consts::TAU * segment as f32 / segments as f32;
        ring.push(Vec3::new(angle.cos() * 1.25, angle.sin() * 0.8, 0.0));
    }
    let mut faces: Vec<[Vec3; 3]> = Vec::with_capacity(segments * 2);
    for segment in 0..segments {
        let current = ring[segment];
        let next = ring[(segment + 1) % segments];
        faces.push([nose, current, next]);
        faces.push([tail, next, current]);
    }
    flat_faces(&faces)
}

const BOSS_HORNS: [Vec3; 8] = [
    Vec3::new(0.0, 0.15, -1.0),
    Vec3::new(0.6, 0.55, -0.6),
    Vec3::new(-0.6, 0.55, -0.6),
    Vec3::new(0.95, -0.1, 0.1),
    Vec3::new(-0.95, -0.1, 0.1),
    Vec3::new(0.0, -0.8, -0.2),
    Vec3::new(0.0, 0.95, 0.2),
    Vec3::new(0.0, 0.1, 1.0),
];

fn boss_displacement(direction: Vec3, seed: f32, spike_scale: f32) -> f32 {
    let lobe_a = (direction.x * 5.0 + seed).sin();
    let lobe_b = (direction.y * 4.0 - seed * 0.6).sin();
    let lobe_c = (direction.z * 6.0 + seed * 1.4).sin();
    let ridged = (1.0 - (lobe_a * lobe_b * lobe_c).abs()) * 0.3;

    let unit = direction.normalize();
    let mut horn = 0.0;
    for (index, spike) in BOSS_HORNS.iter().enumerate() {
        let bias = ((seed * 0.7 + index as f32).sin() * 0.5 + 0.5) * 0.7 + 0.35;
        let aim = unit.dot(&spike.normalize()).max(0.0);
        horn += aim.powf(16.0) * bias;
    }

    ridged * spike_scale + horn * (0.85 + spike_scale * 0.7)
}

fn push_face(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, corners: [Vec3; 3]) {
    let mut normal = (corners[1] - corners[0])
        .cross(&(corners[2] - corners[0]))
        .normalize();
    let centroid = (corners[0] + corners[1] + corners[2]) / 3.0;
    if normal.dot(&centroid) < 0.0 {
        normal = -normal;
    }
    for corner in corners {
        let base = vertices.len() as u32;
        let uv = [0.5 + corner.x * 0.2, 0.5 + corner.y * 0.2];
        vertices.push(Vertex::with_tex_coords(corner, normal, uv));
        indices.push(base);
    }
}

fn build_hulk(seed: f32, y_squash: f32, spike_scale: f32) -> (Vec<Vertex>, Vec<u32>) {
    let stacks = 26usize;
    let sectors = 34usize;
    let stride = sectors + 1;

    let mut positions: Vec<Vec3> = Vec::with_capacity((stacks + 1) * stride);
    for stack in 0..=stacks {
        let phi = std::f32::consts::PI * stack as f32 / stacks as f32;
        for sector in 0..=sectors {
            let theta = std::f32::consts::TAU * sector as f32 / sectors as f32;
            let direction = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            let radius = (1.0 + boss_displacement(direction, seed, spike_scale)).max(0.5);
            positions.push(Vec3::new(direction.x, direction.y * y_squash, direction.z) * radius);
        }
    }

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for stack in 0..stacks {
        for sector in 0..sectors {
            let top_left = positions[stack * stride + sector];
            let top_right = positions[stack * stride + sector + 1];
            let bottom_left = positions[(stack + 1) * stride + sector];
            let bottom_right = positions[(stack + 1) * stride + sector + 1];
            push_face(
                &mut vertices,
                &mut indices,
                [top_left, bottom_left, top_right],
            );
            push_face(
                &mut vertices,
                &mut indices,
                [top_right, bottom_left, bottom_right],
            );
        }
    }

    (vertices, indices)
}
