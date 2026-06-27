use crate::ecs::{Backdrop, GameState, TemplateWorld};
use crate::systems::common::*;
use crate::systems::planet_texture::{self, PlanetStyle};
use nightshade::prelude::*;

const RECYCLE_Z: f32 = 260.0;
const SPAN: f32 = 1900.0;

struct Planet {
    position: Vec3,
    radius: f32,
    style: PlanetStyle,
    low: [f32; 3],
    high: [f32; 3],
    emissive: [f32; 3],
    atmosphere: Option<[f32; 3]>,
    unlit: bool,
    drift: f32,
}

fn planets() -> [Planet; 5] {
    [
        Planet {
            position: Vec3::new(-330.0, 150.0, -1150.0),
            radius: 110.0,
            style: PlanetStyle::Star,
            low: [1.0, 0.55, 0.15],
            high: [1.0, 0.97, 0.82],
            emissive: [4.5, 3.3, 1.4],
            atmosphere: None,
            unlit: true,
            drift: 5.0,
        },
        Planet {
            position: Vec3::new(300.0, 110.0, -1450.0),
            radius: 80.0,
            style: PlanetStyle::Blotchy,
            low: [0.08, 0.2, 0.52],
            high: [0.3, 0.55, 0.35],
            emissive: [0.02, 0.03, 0.06],
            atmosphere: Some([0.4, 0.62, 1.0]),
            unlit: false,
            drift: 7.0,
        },
        Planet {
            position: Vec3::new(-260.0, -120.0, -900.0),
            radius: 60.0,
            style: PlanetStyle::Blotchy,
            low: [0.45, 0.22, 0.12],
            high: [0.78, 0.5, 0.32],
            emissive: [0.04, 0.02, 0.0],
            atmosphere: Some([1.0, 0.6, 0.4]),
            unlit: false,
            drift: 9.0,
        },
        Planet {
            position: Vec3::new(250.0, -140.0, -1700.0),
            radius: 130.0,
            style: PlanetStyle::Banded,
            low: [0.32, 0.2, 0.46],
            high: [0.78, 0.58, 0.86],
            emissive: [0.04, 0.02, 0.06],
            atmosphere: Some([0.72, 0.52, 1.0]),
            unlit: false,
            drift: 6.0,
        },
        Planet {
            position: Vec3::new(-300.0, 100.0, -1900.0),
            radius: 75.0,
            style: PlanetStyle::Banded,
            low: [0.08, 0.36, 0.4],
            high: [0.45, 0.85, 0.78],
            emissive: [0.02, 0.06, 0.05],
            atmosphere: Some([0.4, 0.9, 0.92]),
            unlit: false,
            drift: 6.5,
        },
    ]
}

pub fn spawn_backdrop(world: &mut World, game: &mut GameState) {
    for (index, planet) in planets().into_iter().enumerate() {
        let texture_name = format!("planet_tex_{index}");
        planet_texture::register(
            world,
            &texture_name,
            planet.style,
            planet.low,
            planet.high,
            index as u32 * 131 + 17,
        );
        let entity = spawn_sphere_at(world, planet.position);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.scale = Vec3::new(planet.radius, planet.radius, planet.radius);
        }
        mark_local_transform_dirty(world, entity);
        let material = Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name),
            emissive_factor: planet.emissive,
            unlit: planet.unlit,
            metallic: 0.0,
            roughness: 0.95,
            ..Default::default()
        };
        register_material(world, entity, format!("planet_mat_{index}"), material);

        game.backdrop.push(Backdrop {
            entity,
            position: planet.position,
            radius: planet.radius,
            atmosphere: planet.atmosphere,
            drift_speed: planet.drift,
        });
    }
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let speed_scale = game.speed_scale;

    for index in 0..game.backdrop.len() {
        game.backdrop[index].position.z += game.backdrop[index].drift_speed * speed_scale * delta;
        if game.backdrop[index].position.z > RECYCLE_Z {
            game.backdrop[index].position.z -= SPAN;
            game.backdrop[index].position.x = random_range(&mut game.random_state, -320.0, 320.0);
            game.backdrop[index].position.y = random_range(&mut game.random_state, -150.0, 160.0);
        }
        let position = game.backdrop[index].position;
        let entity = game.backdrop[index].entity;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
        }
        mark_local_transform_dirty(world, entity);
    }
}
