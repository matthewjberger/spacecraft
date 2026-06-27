use crate::ecs::TemplateWorld;
use crate::systems::atmosphere::AtmosphereState;
use crate::systems::{atmosphere, backdrop, camera, flight, scenery, setup, weapons};
use nightshade::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Spacecraft {
    pub template_world: TemplateWorld,
    pub atmosphere: Arc<Mutex<AtmosphereState>>,
}

impl State for Spacecraft {
    fn initialize(&mut self, world: &mut World) {
        setup::build(&mut self.template_world, world);
    }

    fn configure_render_graph(
        &mut self,
        graph: &mut RenderGraph<World>,
        device: &wgpu::Device,
        _surface_format: wgpu::TextureFormat,
        resources: RenderResources,
    ) {
        let pass = atmosphere::AtmospherePass::new(device, self.atmosphere.clone());
        let _ = render_graph_pass(graph, Box::new(pass))
            .slot("hdr", resources.scene_color)
            .add();
    }

    fn run_systems(&mut self, world: &mut World) {
        if world.resources.input.keyboard.just_pressed(KeyCode::Escape) {
            world.resources.window.should_exit = true;
        }

        flight::update(&mut self.template_world, world);
        scenery::update(&mut self.template_world, world);
        weapons::update(&mut self.template_world, world);
        backdrop::update(&mut self.template_world, world);
        camera::update(&mut self.template_world, world);
        atmosphere::sync(&self.template_world.resources.game, &self.atmosphere);
    }
}
