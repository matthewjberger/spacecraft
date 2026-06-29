use crate::ecs::{Fragment, GameState, SceneryKind, TemplateWorld};
use crate::systems::asteroid_mesh;
use crate::systems::common::*;
use nightshade::prelude::*;

pub fn update(game_world: &mut TemplateWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let game = &mut game_world.resources.game;
    if game.ship.is_none() {
        return;
    }
    ensure_beam(world, game);

    let ship = game.ship_position;
    let lance = game.mods.lance as f32;
    let nose = Vec3::new(ship.x, ship.y, ship.z - 1.6);
    let aim_dir = (aim_point(game) - nose).normalize();

    if game.laser_cooldown > 0.0 {
        game.laser_cooldown -= delta;
    }
    if game.laser_timer > 0.0 {
        game.laser_timer -= delta;
    }
    if game.mods.lance > 0
        && laser_pressed(world)
        && game.laser_timer <= 0.0
        && game.laser_cooldown <= 0.0
    {
        game.laser_timer = LASER_DURATION;
        game.laser_cooldown = LASER_DURATION + LASER_COOLDOWN;
        game.cam_kick += LASER_KICK;
        game.cam_fov_pop = game.cam_fov_pop.max(FOV_POP_LASER);
    }

    let active = game.laser_timer > 0.0;
    let strength = if active {
        let progress = 1.0 - (game.laser_timer / LASER_DURATION).clamp(0.0, 1.0);
        (std::f32::consts::PI * progress).sin().max(0.0)
    } else {
        0.0
    };

    if let Some(beam_entity) = game.beam
        && let Some(beam) = world.core.get_beam_mut(beam_entity)
    {
        if strength > 0.0 {
            beam.start = nose;
            beam.end = nose + aim_dir * LASER_LENGTH;
            beam.width = strength * (0.85 + lance * 0.3);
            beam.alpha = (strength * 1.4).min(1.0);
            beam.intensity = 3.5 + strength * 7.0;
            beam.color = Vec3::new(
                1.1 + strength * 1.0,
                2.2 + strength * 2.0,
                3.2 + strength * 1.4,
            );
            beam.strands = 12 + game.mods.lance as u32 * 3;
            beam.flicker = 0.16;
            beam.flicker_speed = 55.0;
        } else {
            beam.start = nose;
            beam.end = nose;
            beam.width = 0.0;
            beam.alpha = 0.0;
            beam.intensity = 0.0;
        }
    }

    if strength > LASER_SLICE_STRENGTH {
        let radius = LASER_SLICE_RADIUS + lance * 0.5;
        slice_asteroids(world, game, nose, aim_dir, radius);
        vaporize_enemies(world, game, nose, aim_dir, radius);
        burn_boss(game, nose, aim_dir, radius, lance, delta);
    } else {
        game.lance_boss_accum = 0.0;
    }

    update_fragments(world, game, delta);
}

fn ensure_beam(world: &mut World, game: &mut GameState) {
    if game.beam.is_some() {
        return;
    }
    let handle = spawn_vfx(world, VfxPreset::Laser, Vec3::new(0.0, BASE_HEIGHT, -50.0));
    let mut beam_entity = None;
    for entity in handle.entities {
        if beam_entity.is_none() && world.core.get_beam_mut(entity).is_some() {
            beam_entity = Some(entity);
        } else {
            despawn_recursive_immediate(world, entity);
        }
    }
    if let Some(entity) = beam_entity
        && let Some(beam) = world.core.get_beam_mut(entity)
    {
        beam.alpha = 0.0;
        beam.width = 0.0;
    }
    game.beam = beam_entity;
}

fn ray_hits(nose: Vec3, direction: Vec3, point: Vec3, radius: f32) -> bool {
    let to_point = point - nose;
    let along = to_point.dot(&direction);
    if !(2.0..=LASER_LENGTH).contains(&along) {
        return false;
    }
    (to_point - direction * along).magnitude() < radius
}

fn slice_asteroids(world: &mut World, game: &mut GameState, nose: Vec3, aim: Vec3, radius: f32) {
    let mut sliced: Vec<usize> = Vec::new();
    for index in 0..game.scenery.len() {
        if game.scenery[index].kind != SceneryKind::Asteroid {
            continue;
        }
        let rock = game.scenery[index].radius;
        if ray_hits(nose, aim, game.scenery[index].position, radius + rock) {
            sliced.push(index);
        }
    }
    for index in sliced.into_iter().rev() {
        let item = game.scenery.remove(index);
        spawn_fragments(world, game, item.position, item.radius);
        let burst = spawn_burst(world, item.position, Vec3::new(0.6, 1.4, 1.9), 26);
        game.bursts.push((burst, 0.0));
        crate::systems::pickups::maybe_drop(world, game, item.position);
        award(game, 2);
        despawn_recursive_immediate(world, item.entity);
    }
}

fn burn_boss(game: &mut GameState, nose: Vec3, aim: Vec3, radius: f32, lance: f32, delta: f32) {
    let Some((position, body)) = game
        .boss
        .as_ref()
        .map(|boss| (boss.position, boss.kind.stats().radius))
    else {
        game.lance_boss_accum = 0.0;
        return;
    };
    if !ray_hits(nose, aim, position, radius + body) {
        return;
    }
    game.lance_boss_accum += (LANCE_BOSS_DPS + lance * 6.0) * delta;
    let whole = game.lance_boss_accum.floor();
    if whole >= 1.0 {
        game.lance_boss_accum -= whole;
        if let Some(boss) = game.boss.as_mut() {
            boss.health -= whole as i32;
        }
    }
}

fn vaporize_enemies(world: &mut World, game: &mut GameState, nose: Vec3, aim: Vec3, radius: f32) {
    let mut killed: Vec<usize> = Vec::new();
    for index in 0..game.enemies.len() {
        let body = game.enemies[index].radius;
        if ray_hits(nose, aim, game.enemies[index].position, radius + body) {
            killed.push(index);
        }
    }
    for index in killed.into_iter().rev() {
        let enemy = game.enemies.remove(index);
        spawn_fragments(world, game, enemy.position, enemy.radius);
        let burst = spawn_burst(world, enemy.position, Vec3::new(0.7, 1.5, 2.0), 30);
        game.bursts.push((burst, 0.0));
        award(game, ENEMY_SCORE);
        despawn_recursive_immediate(world, enemy.entity);
        if let Some(thruster) = enemy.thruster {
            despawn_recursive_immediate(world, thruster);
        }
    }
}

pub fn spawn_fragments(world: &mut World, game: &mut GameState, position: Vec3, rock: f32) {
    for _ in 0..3 {
        let variant = ((next_random(&mut game.random_state)
            * asteroid_mesh::ASTEROID_VARIANTS as f32) as usize)
            .min(asteroid_mesh::ASTEROID_VARIANTS - 1);
        let scale = rock * random_range(&mut game.random_state, 0.32, 0.55);
        let entity = spawn_mesh(
            world,
            &asteroid_mesh::asteroid_name(variant),
            position,
            Vec3::new(scale, scale, scale),
        );
        apply_material(
            world,
            entity,
            "rock",
            [0.54, 0.52, 0.58, 1.0],
            [0.18, 0.22, 0.3],
            false,
            true,
        );
        let direction = Vec3::new(
            random_range(&mut game.random_state, -1.0, 1.0),
            random_range(&mut game.random_state, -1.0, 1.0),
            random_range(&mut game.random_state, -0.4, 0.4),
        )
        .normalize();
        let velocity = direction * random_range(&mut game.random_state, 4.0, 9.5);
        let spin_axis = Vec3::new(
            random_range(&mut game.random_state, -1.0, 1.0),
            random_range(&mut game.random_state, -1.0, 1.0),
            random_range(&mut game.random_state, -1.0, 1.0),
        )
        .normalize();
        game.fragments.push(Fragment {
            entity,
            position,
            velocity,
            spin_axis,
            angle: 0.0,
            spin_speed: random_range(&mut game.random_state, 1.5, 4.5),
            life: FRAGMENT_LIFE,
            scale,
        });
    }
}

fn update_fragments(world: &mut World, game: &mut GameState, delta: f32) {
    let rail = RAIL_SPEED * game.speed_scale;
    let mut remove: Vec<usize> = Vec::new();
    for index in 0..game.fragments.len() {
        game.fragments[index].life -= delta;
        let velocity = game.fragments[index].velocity;
        game.fragments[index].position += velocity * delta;
        game.fragments[index].position.z += rail * delta;
        game.fragments[index].angle += game.fragments[index].spin_speed * delta;

        let life = game.fragments[index].life;
        let position = game.fragments[index].position;
        let angle = game.fragments[index].angle;
        let axis = game.fragments[index].spin_axis;
        let base_scale = game.fragments[index].scale;
        let entity = game.fragments[index].entity;
        let fade = (life / FRAGMENT_LIFE).clamp(0.0, 1.0);

        let rotation = nalgebra_glm::quat_angle_axis(angle, &axis);
        if let Some(transform) = world.core.get_local_transform_mut(entity) {
            transform.translation = position;
            transform.rotation = rotation;
            let scale = base_scale * fade;
            transform.scale = Vec3::new(scale, scale, scale);
        }
        mark_local_transform_dirty(world, entity);

        if life <= 0.0 || position.z > SCENERY_DESPAWN_Z {
            remove.push(index);
        }
    }
    for index in remove.into_iter().rev() {
        let fragment = game.fragments.remove(index);
        despawn_recursive_immediate(world, fragment.entity);
    }
}

fn laser_pressed(world: &World) -> bool {
    world.resources.input.keyboard.just_pressed(KeyCode::KeyF)
        || world
            .resources
            .input
            .gamepad
            .just_pressed(gilrs::Button::North)
}
