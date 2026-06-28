use crate::content::ModKind;
use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::common::*;
use crate::systems::shop;
use crate::systems::textures::proto_material;
use nightshade::ecs::mesh::components::{Mesh, Vertex};
use nightshade::prelude::*;

const DAIS_MESH: &str = "hangar_dais";
const SHOWCASE_SCALE: f32 = 2.4;

fn showcase_pos() -> Vec3 {
    Vec3::new(0.0, BASE_HEIGHT + 4.4, -1.0)
}

pub fn spawn_upgrade_props(world: &mut World) -> Vec<(Entity, ModKind, Vec3, Vec3)> {
    let mut props: Vec<(Entity, ModKind, Vec3, Vec3)> = Vec::new();
    glow_prop(
        world,
        &mut props,
        ModKind::Hull,
        "Cube",
        Vec3::new(0.0, -0.55, 0.1),
        Vec3::new(1.8, 0.22, 1.4),
        glow_material([0.35, 0.55, 1.0], 1.6),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Rapid,
        "Cube",
        Vec3::new(0.0, -0.02, -1.5),
        Vec3::new(1.6, 0.16, 0.36),
        glow_material([1.8, 0.85, 0.2], 2.4),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Seeker,
        "Cylinder",
        Vec3::new(0.95, -0.32, 0.15),
        Vec3::new(0.3, 0.78, 0.3),
        glow_material([1.4, 0.5, 0.2], 2.2),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Nova,
        "Sphere",
        Vec3::new(0.0, 0.82, 0.2),
        Vec3::new(0.5, 0.5, 0.5),
        glow_material([0.7, 0.95, 1.8], 2.6),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Aegis,
        "Cylinder",
        Vec3::new(0.0, 0.0, 0.1),
        Vec3::new(2.3, 0.04, 2.3),
        glow_material([0.25, 0.85, 1.2], 1.4),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Lance,
        "Cube",
        Vec3::new(0.0, 0.12, -1.7),
        Vec3::new(0.18, 0.18, 0.9),
        glow_material([1.5, 0.3, 1.3], 2.6),
    );
    glow_prop(
        world,
        &mut props,
        ModKind::Magnet,
        "Cylinder",
        Vec3::new(0.0, -0.28, 1.0),
        Vec3::new(1.2, 0.12, 1.2),
        glow_material([0.3, 1.3, 0.55], 2.0),
    );
    props
}

fn glow_material(emissive: [f32; 3], strength: f32) -> Material {
    Material {
        base_color: [0.04, 0.04, 0.06, 1.0],
        emissive_factor: emissive,
        emissive_strength: strength,
        metallic: 0.1,
        roughness: 0.45,
        ..Default::default()
    }
}

fn glow_prop(
    world: &mut World,
    props: &mut Vec<(Entity, ModKind, Vec3, Vec3)>,
    kind: ModKind,
    mesh: &str,
    offset: Vec3,
    scale: Vec3,
    material: Material,
) {
    let entity = spawn_mesh(world, mesh, offset, Vec3::zeros());
    let name = format!("prop_{}", entity.id);
    register_material(world, entity, name, material);
    props.push((entity, kind, offset, scale));
}

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

    let showcase = game.mode == GameMode::Shop;
    let rotation =
        nalgebra_glm::quat_angle_axis(SHIP_BASE_YAW + elapsed * 0.5, &Vec3::new(0.0, 1.0, 0.0))
            * nalgebra_glm::quat_angle_axis(0.14, &Vec3::new(1.0, 0.0, 0.0));

    if showcase {
        if let Some(ship) = game.ship {
            if let Some(transform) = world.core.get_local_transform_mut(ship) {
                transform.translation = showcase_pos();
                transform.rotation = rotation;
                transform.scale = Vec3::new(
                    SHIP_SCALE * SHOWCASE_SCALE,
                    SHIP_SCALE * SHOWCASE_SCALE,
                    SHIP_SCALE * SHOWCASE_SCALE,
                );
            }
            mark_local_transform_dirty(world, ship);
        }
        if let Some(exhaust) = game.exhaust
            && let Some(emitter) = world.core.get_particle_emitter_mut(exhaust)
        {
            emitter.spawn_rate = 0.0;
        }
        for &thruster in &game.corner_thrusters {
            if let Some(emitter) = world.core.get_particle_emitter_mut(thruster) {
                emitter.spawn_rate = 0.0;
            }
        }
    }

    for index in 0..game.upgrade_props.len() {
        let (entity, kind, offset, scale) = game.upgrade_props[index];
        let owned = showcase && shop::item_level(&game.mods, kind) > 0;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            if owned {
                let world_off = showcase_pos()
                    + nalgebra_glm::quat_rotate_vec3(&rotation, &(offset * SHOWCASE_SCALE));
                transform.translation = world_off;
                transform.rotation = rotation;
                transform.scale = scale * SHOWCASE_SCALE;
            } else {
                transform.scale = Vec3::zeros();
            }
        }
        mark_local_transform_dirty(world, entity);
    }

    let Some(dais) = game.dais else {
        return;
    };
    if let Some(transform) = world.core.get_local_transform_mut(dais) {
        transform.translation = if showcase {
            showcase_pos() + Vec3::new(0.0, -2.0, 0.0)
        } else {
            Vec3::new(0.0, BASE_HEIGHT - 1.2, -1.5)
        };
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
