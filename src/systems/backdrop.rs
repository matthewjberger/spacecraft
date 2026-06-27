use crate::ecs::{Backdrop, GameState, Moon, TemplateWorld};
use crate::systems::planet_texture::{self, PlanetStyle};
use nightshade::prelude::*;

const ECLIPTIC_TILT: f32 = 0.12;
const SOLAR_CENTER_X: f32 = -620.0;
const SOLAR_CENTER_Y: f32 = 120.0;
const SOLAR_CENTER_Z: f32 = -1050.0;

struct Body {
    orbit_radius: f32,
    orbit_angle: f32,
    orbit_speed: f32,
    radius: f32,
    style: PlanetStyle,
    low: [f32; 3],
    high: [f32; 3],
    emissive: [f32; 3],
    atmosphere: Option<[f32; 3]>,
    unlit: bool,
    moons: u32,
}

fn bodies() -> [Body; 5] {
    [
        Body {
            orbit_radius: 0.0,
            orbit_angle: 0.0,
            orbit_speed: 0.0,
            radius: 110.0,
            style: PlanetStyle::Star,
            low: [1.0, 0.55, 0.15],
            high: [1.0, 0.97, 0.82],
            emissive: [4.5, 3.3, 1.4],
            atmosphere: None,
            unlit: true,
            moons: 0,
        },
        Body {
            orbit_radius: 230.0,
            orbit_angle: 0.6,
            orbit_speed: 0.06,
            radius: 26.0,
            style: PlanetStyle::Blotchy,
            low: [0.45, 0.22, 0.12],
            high: [0.78, 0.5, 0.32],
            emissive: [0.04, 0.02, 0.0],
            atmosphere: Some([1.0, 0.6, 0.4]),
            unlit: false,
            moons: 1,
        },
        Body {
            orbit_radius: 380.0,
            orbit_angle: 2.3,
            orbit_speed: 0.04,
            radius: 44.0,
            style: PlanetStyle::Blotchy,
            low: [0.08, 0.2, 0.52],
            high: [0.3, 0.55, 0.35],
            emissive: [0.02, 0.03, 0.06],
            atmosphere: Some([0.4, 0.62, 1.0]),
            unlit: false,
            moons: 1,
        },
        Body {
            orbit_radius: 540.0,
            orbit_angle: 4.0,
            orbit_speed: 0.028,
            radius: 64.0,
            style: PlanetStyle::Banded,
            low: [0.32, 0.2, 0.46],
            high: [0.78, 0.58, 0.86],
            emissive: [0.04, 0.02, 0.06],
            atmosphere: Some([0.72, 0.52, 1.0]),
            unlit: false,
            moons: 2,
        },
        Body {
            orbit_radius: 700.0,
            orbit_angle: 5.4,
            orbit_speed: 0.02,
            radius: 40.0,
            style: PlanetStyle::Banded,
            low: [0.08, 0.36, 0.4],
            high: [0.45, 0.85, 0.78],
            emissive: [0.02, 0.06, 0.05],
            atmosphere: Some([0.4, 0.9, 0.92]),
            unlit: false,
            moons: 1,
        },
    ]
}

fn orbit_offset(orbit_radius: f32, angle: f32, tilt: f32) -> Vec3 {
    Vec3::new(
        orbit_radius * angle.cos(),
        orbit_radius * angle.sin() * tilt.sin(),
        orbit_radius * angle.sin() * tilt.cos(),
    )
}

pub fn spawn_backdrop(world: &mut World, game: &mut GameState) {
    game.solar_center = Vec3::new(SOLAR_CENTER_X, SOLAR_CENTER_Y, SOLAR_CENTER_Z);
    let center = game.solar_center;

    for (index, body) in bodies().into_iter().enumerate() {
        let texture_name = format!("planet_tex_{index}");
        planet_texture::register(
            world,
            &texture_name,
            body.style,
            body.low,
            body.high,
            index as u32 * 131 + 17,
        );
        let position = center + orbit_offset(body.orbit_radius, body.orbit_angle, ECLIPTIC_TILT);
        let entity = spawn_sphere_at(world, position);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.scale = Vec3::new(body.radius, body.radius, body.radius);
        }
        mark_local_transform_dirty(world, entity);
        world.core.remove_components(entity, CASTS_SHADOW);
        let material = Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: Some(texture_name),
            emissive_factor: body.emissive,
            unlit: body.unlit,
            metallic: 0.0,
            roughness: 0.95,
            ..Default::default()
        };
        register_material(world, entity, format!("planet_mat_{index}"), material);

        for moon_index in 0..body.moons {
            let moon_radius = body.radius * 0.18;
            let orbit_radius = body.radius * (2.2 + moon_index as f32 * 0.9);
            let moon_position = position + orbit_offset(orbit_radius, moon_index as f32, 0.5);
            let moon_entity = spawn_sphere_at(world, moon_position);
            if let Some(transform) = world.core.get_local_transform_mut(moon_entity) {
                transform.scale = Vec3::new(moon_radius, moon_radius, moon_radius);
            }
            mark_local_transform_dirty(world, moon_entity);
            world.core.remove_components(moon_entity, CASTS_SHADOW);
            let moon_material = Material {
                base_color: [0.55, 0.54, 0.58, 1.0],
                emissive_factor: [0.02, 0.02, 0.03],
                metallic: 0.0,
                roughness: 0.95,
                ..Default::default()
            };
            register_material(world, moon_entity, "moon_mat".to_string(), moon_material);
            game.moons.push(Moon {
                entity: moon_entity,
                parent: index,
                radius: moon_radius,
                orbit_radius,
                orbit_angle: moon_index as f32 * 1.7,
                orbit_speed: 0.4 + moon_index as f32 * 0.15,
                tilt: 0.5 - moon_index as f32 * 0.7,
            });
        }

        game.backdrop.push(Backdrop {
            entity,
            position,
            radius: body.radius,
            atmosphere: body.atmosphere,
            orbit_radius: body.orbit_radius,
            orbit_angle: body.orbit_angle,
            orbit_speed: body.orbit_speed,
        });
    }
}

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    let center = game.solar_center;

    for index in 0..game.backdrop.len() {
        game.backdrop[index].orbit_angle += game.backdrop[index].orbit_speed * delta;
        let position = center
            + orbit_offset(
                game.backdrop[index].orbit_radius,
                game.backdrop[index].orbit_angle,
                ECLIPTIC_TILT,
            );
        game.backdrop[index].position = position;
        let entity = game.backdrop[index].entity;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
        }
        mark_local_transform_dirty(world, entity);
    }

    for index in 0..game.moons.len() {
        game.moons[index].orbit_angle += game.moons[index].orbit_speed * delta;
        let parent_position = game.backdrop[game.moons[index].parent].position;
        let position = parent_position
            + orbit_offset(
                game.moons[index].orbit_radius,
                game.moons[index].orbit_angle,
                game.moons[index].tilt,
            );
        let entity = game.moons[index].entity;
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
        }
        mark_local_transform_dirty(world, entity);
    }
}
