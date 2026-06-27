use std::sync::{Arc, Mutex};

use crate::ecs::GameState;
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtmosphereInstance {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

#[derive(Default)]
pub struct AtmosphereState {
    pub instances: Vec<AtmosphereInstance>,
}

pub fn sync(game: &GameState, shared: &Arc<Mutex<AtmosphereState>>) {
    let mut instances = Vec::new();
    for backdrop in &game.backdrop {
        if let Some(atmosphere) = backdrop.atmosphere {
            let radius = backdrop.radius * 1.18;
            let position = backdrop.position;
            let model = [
                [radius, 0.0, 0.0, 0.0],
                [0.0, radius, 0.0, 0.0],
                [0.0, 0.0, radius, 0.0],
                [position.x, position.y, position.z, 1.0],
            ];
            instances.push(AtmosphereInstance {
                model,
                color: [atmosphere[0], atmosphere[1], atmosphere[2], 0.45],
            });
        }
    }
    let mut state = shared.lock().unwrap();
    state.instances = instances;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

const MAX_INSTANCES: usize = 16;
const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const SHADER: &str = r#"
struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) model_0: vec4<f32>,
    @location(3) model_1: vec4<f32>,
    @location(4) model_2: vec4<f32>,
    @location(5) model_3: vec4<f32>,
    @location(6) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) center: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) shell_radius: f32,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(input.model_0, input.model_1, input.model_2, input.model_3);
    let world_position = model * vec4<f32>(input.position, 1.0);
    var output: VertexOutput;
    output.clip_position = uniforms.projection * uniforms.view * world_position;
    output.world_position = world_position.xyz;
    output.center = input.model_3.xyz;
    output.shell_radius = length(input.model_0.xyz);
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ray = normalize(input.world_position - uniforms.camera_position.xyz);
    let to_center = input.center - uniforms.camera_position.xyz;
    let approach = max(dot(to_center, ray), 0.0);
    let closest = uniforms.camera_position.xyz + ray * approach;
    let impact = length(input.center - closest);

    let shell = input.shell_radius;
    let surface = shell / 1.18;

    let peak = sqrt(max(shell * shell - surface * surface, 0.0));
    let outer = sqrt(max(shell * shell - impact * impact, 0.0));
    let inner = sqrt(max(surface * surface - impact * impact, 0.0));
    let halo = (outer - inner) / peak;
    let exterior = smoothstep(surface * 0.8, surface, impact);
    let density = halo * mix(0.18, 1.0, exterior);

    let glow = input.color.rgb * density * input.color.a;
    return vec4<f32>(glow, density);
}
"#;

pub struct AtmospherePass {
    shared: Arc<Mutex<AtmosphereState>>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl AtmospherePass {
    pub fn new(device: &wgpu::Device, shared: Arc<Mutex<AtmosphereState>>) -> Self {
        let (vertices, indices) = unit_sphere(28, 18);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("atmosphere_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("atmosphere_indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_count = indices.len() as u32;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("atmosphere_instances"),
            size: (std::mem::size_of::<AtmosphereInstance>() * MAX_INSTANCES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("atmosphere_uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("atmosphere_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atmosphere_bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("atmosphere_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("atmosphere_layout"),
            bind_group_layouts: &[Some(&uniform_bind_group_layout)],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<AtmosphereInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("atmosphere_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[vertex_layout, instance_layout],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            shared,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instance_count: 0,
        }
    }
}

impl nightshade::render::wgpu::rendergraph::PassNode<World> for AtmospherePass {
    fn name(&self) -> &str {
        "atmosphere_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["depth"]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["hdr"]
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, configs: &World) {
        if let Some(matrices) = query_active_camera_matrices(configs) {
            let uniforms = GpuUniforms {
                view: mat_to_array(&matrices.view),
                projection: mat_to_array(&matrices.projection),
                camera_position: [
                    matrices.camera_position.x,
                    matrices.camera_position.y,
                    matrices.camera_position.z,
                    1.0,
                ],
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        }

        let instances = {
            let state = self.shared.lock().unwrap();
            let count = state.instances.len().min(MAX_INSTANCES);
            state.instances[..count].to_vec()
        };
        self.instance_count = instances.len() as u32;
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: nightshade::render::wgpu::rendergraph::PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if !context.is_pass_enabled() || self.instance_count == 0 {
            return Ok(context.into_sub_graph_commands());
        }

        let (hdr_view, load_op, store_op) = context.get_color_attachment("hdr")?;
        let depth_view = context.get_texture_view("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("atmosphere_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: hdr_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

fn mat_to_array(matrix: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let slice = matrix.as_slice();
    [
        [slice[0], slice[1], slice[2], slice[3]],
        [slice[4], slice[5], slice[6], slice[7]],
        [slice[8], slice[9], slice[10], slice[11]],
        [slice[12], slice[13], slice[14], slice[15]],
    ]
}

fn unit_sphere(sectors: usize, stacks: usize) -> (Vec<GpuVertex>, Vec<u32>) {
    let stride = sectors + 1;
    let mut vertices = Vec::with_capacity((stacks + 1) * stride);
    for stack in 0..=stacks {
        let phi = std::f32::consts::PI * stack as f32 / stacks as f32;
        for sector in 0..=sectors {
            let theta = std::f32::consts::TAU * sector as f32 / sectors as f32;
            let normal = [phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin()];
            vertices.push(GpuVertex {
                position: normal,
                normal,
            });
        }
    }
    let mut indices = Vec::new();
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
    (vertices, indices)
}
