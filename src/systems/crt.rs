use std::sync::{Arc, Mutex};

use crate::ecs::GameState;
use nightshade::prelude::*;

#[derive(Default)]
pub struct CrtState {
    pub strength: f32,
}

pub fn sync(game: &GameState, shared: &Arc<Mutex<CrtState>>) {
    let mut state = shared.lock().unwrap();
    state.strength = if game.crt_enabled { 1.0 } else { 0.0 };
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    params0: [f32; 4],
    params1: [f32; 4],
}

const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

const SHADER: &str = r#"
struct Uniforms {
    params0: vec4<f32>,
    params1: vec4<f32>,
};
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0 - y);
    return out;
}

fn barrel(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let radius = dot(centered, centered);
    return uv + centered * radius * amount;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.params0.x;
    let strength = uniforms.params0.y;
    let resolution = uniforms.params0.zw;
    let resolve = uniforms.params1.x;

    if (resolve > 0.5) {
        return textureSample(input_texture, input_sampler, in.uv);
    }

    let curve = 0.32 * strength;
    let uv = barrel(in.uv, curve);

    let centered = uv - 0.5;
    let edge = dot(centered, centered);
    let direction = centered / max(sqrt(edge), 0.0001);
    let aberration = 0.0045 * strength * edge * 4.0;

    let red = textureSample(input_texture, input_sampler, uv + direction * aberration).r;
    let green = textureSample(input_texture, input_sampler, uv).g;
    let blue = textureSample(input_texture, input_sampler, uv - direction * aberration).b;
    var color = vec3<f32>(red, green, blue);

    let inside = step(0.0, uv.x) * step(uv.x, 1.0) * step(0.0, uv.y) * step(uv.y, 1.0);
    color = color * inside;

    let scan = sin(uv.y * 320.0 * 6.2831853);
    let scanline = 1.0 - 0.35 * strength * scan * scan;

    let column = i32(floor(uv.x * resolution.x));
    var mask = vec3<f32>(1.0, 1.0, 1.0);
    let phase = column % 3;
    if (phase == 0) {
        mask = vec3<f32>(1.0, 0.7, 0.7);
    } else if (phase == 1) {
        mask = vec3<f32>(0.7, 1.0, 0.7);
    } else {
        mask = vec3<f32>(0.7, 0.7, 1.0);
    }
    let grille = mix(vec3<f32>(1.0), mask, 0.30 * strength);

    let vignette = 1.0 - edge * 2.3 * strength;
    let flicker = 1.0 - 0.04 * strength * sin(time * 110.0);

    color = color * scanline * grille * vignette * flicker;
    color = color * (1.0 + 0.14 * strength);

    return vec4<f32>(color, 1.0);
}
"#;

pub struct CrtPass {
    name: String,
    resolve: bool,
    shared: Arc<Mutex<CrtState>>,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
}

impl CrtPass {
    pub fn new(device: &wgpu::Device, resolve: bool, shared: Arc<Mutex<CrtState>>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("crt_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("crt_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crt_uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crt_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("crt_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("crt_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let name = if resolve {
            "crt_resolve_pass".to_string()
        } else {
            "crt_warp_pass".to_string()
        };

        Self {
            name,
            resolve,
            shared,
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
        }
    }
}

impl nightshade::render::wgpu::rendergraph::PassNode<World> for CrtPass {
    fn name(&self) -> &str {
        &self.name
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        if self.resolve { vec![] } else { vec!["output"] }
    }

    fn reads_writes(&self) -> Vec<&str> {
        if self.resolve { vec!["output"] } else { vec![] }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: nightshade::render::wgpu::rendergraph::PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }

        let (width, height) = context.get_texture_size("input").unwrap_or((1, 1));
        let time = context.configs.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let strength = self.shared.lock().unwrap().strength;
        let resolve = if self.resolve { 1.0 } else { 0.0 };
        let uniforms = GpuUniforms {
            params0: [time, strength, width as f32, height as f32],
            params1: [resolve, 0.0, 0.0, 0.0],
        };
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let input_view = context.get_texture_view("input")?;
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("crt_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let (color_view, load_op, store_op) = context.get_color_attachment("output")?;
        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("crt_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
