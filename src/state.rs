use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::atmosphere::AtmosphereState;
use crate::systems::ring_fx::RingState;
use crate::systems::{
    atmosphere, backdrop, boss, camera, combat, enemies, flight, game, hud, ring_fx, scenery,
    setup, weapons,
};
use nightshade::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Spacecraft {
    pub template_world: TemplateWorld,
    pub atmosphere: Arc<Mutex<AtmosphereState>>,
    pub rings: Arc<Mutex<RingState>>,
}

impl State for Spacecraft {
    fn initialize(&mut self, world: &mut World) {
        setup::build(&mut self.template_world, world);
        hud::build(&mut self.template_world, world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        _surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let atmosphere_pass = atmosphere::AtmospherePass::new(device, self.atmosphere.clone());
        let _ = render_graph_pass(graph, Box::new(atmosphere_pass))
            .read("depth", resources.depth)
            .slot("hdr", resources.scene_color)
            .add();

        let ring_pass = ring_fx::RingFxPass::new(device, self.rings.clone());
        let _ = render_graph_pass(graph, Box::new(ring_pass))
            .read("depth", resources.depth)
            .slot("hdr", resources.scene_color)
            .add();
    }

    fn run_systems(&mut self, world: &mut World) {
        if world.resources.input.keyboard.just_pressed(KeyCode::Escape) {
            world.resources.window.should_exit = true;
        }

        game::update(&mut self.template_world, world);
        let mode = self.template_world.resources.game.mode;

        flight::update(&mut self.template_world, world);
        if mode == GameMode::Playing {
            scenery::update(&mut self.template_world, world);
            enemies::update(&mut self.template_world, world);
            boss::update(&mut self.template_world, world);
            weapons::update(&mut self.template_world, world);
            combat::update(&mut self.template_world, world);
        }
        backdrop::update(&mut self.template_world, world);
        camera::update(&mut self.template_world, world);
        hud::update(&mut self.template_world, world);

        atmosphere::sync(&self.template_world.resources.game, &self.atmosphere);
        ring_fx::sync(&self.template_world.resources.game, &self.rings);
    }
}
