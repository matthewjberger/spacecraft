use crate::ecs::{GameMode, TemplateWorld};
use crate::systems::atmosphere::AtmosphereState;
use crate::systems::crt::CrtState;
use crate::systems::ring_fx::RingState;
use crate::systems::{
    abilities, atmosphere, backdrop, boss, camera, combat, crt, director, enemies, flight, game,
    hangar, hud, laser, missiles, pickups, reticle, ring_fx, scenery, setup, shield, weapons,
};
use nightshade::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Spacecraft {
    pub template_world: TemplateWorld,
    pub atmosphere: Arc<Mutex<AtmosphereState>>,
    pub rings: Arc<Mutex<RingState>>,
    pub crt: Arc<Mutex<CrtState>>,
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

        let crt_buffer = render_graph_add_color_texture(graph, "crt_buffer")
            .format(wgpu::TextureFormat::Rgba16Float)
            .size(
                resources.surface_width.max(1),
                resources.surface_height.max(1),
            )
            .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
            .transient();

        let crt_warp = crt::CrtPass::new(device, false, self.crt.clone());
        let _ = render_graph_pass(graph, Box::new(crt_warp))
            .read("input", resources.scene_color)
            .write("output", crt_buffer)
            .add();

        let crt_resolve = crt::CrtPass::new(device, true, self.crt.clone());
        let _ = render_graph_pass(graph, Box::new(crt_resolve))
            .read("input", crt_buffer)
            .slot("output", resources.scene_color)
            .add();
    }

    fn run_systems(&mut self, world: &mut World) {
        game::update(&mut self.template_world, world);
        let mode = self.template_world.resources.game.mode;

        let frozen = {
            let game = &mut self.template_world.resources.game;
            if game.hitstop > 0.0 {
                game.hitstop -= world.resources.window.timing.delta_time;
                true
            } else {
                false
            }
        };

        if !frozen {
            flight::update(&mut self.template_world, world);
        }
        reticle::update(&mut self.template_world, world);
        shield::update(&mut self.template_world, world);
        if mode == GameMode::Playing && !frozen {
            director::update(&mut self.template_world, world);
            scenery::update(&mut self.template_world, world);
            enemies::update(&mut self.template_world, world);
            boss::update(&mut self.template_world, world);
            pickups::update(&mut self.template_world, world);
            weapons::update(&mut self.template_world, world);
            laser::update(&mut self.template_world, world);
            missiles::update(&mut self.template_world, world);
            abilities::update(&mut self.template_world, world);
            combat::update(&mut self.template_world, world);
        }
        backdrop::update(&mut self.template_world, world);
        camera::update(&mut self.template_world, world);
        hangar::update(&mut self.template_world, world);
        hud::update(&mut self.template_world, world);

        atmosphere::sync(&self.template_world.resources.game, &self.atmosphere);
        ring_fx::sync(&self.template_world.resources.game, &self.rings);
        crt::sync(&self.template_world.resources.game, &self.crt);
    }
}
