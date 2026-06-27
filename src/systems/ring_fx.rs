use std::sync::{Arc, Mutex};

use crate::ecs::{GameState, SceneryKind};
use crate::systems::common::*;
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RingInstance {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
}

#[derive(Default)]
pub struct RingState {
    pub instances: Vec<RingInstance>,
}

pub fn sync(game: &GameState, shared: &Arc<Mutex<RingState>>) {
    let mut instances = Vec::new();
    for scenery in &game.scenery {
        if scenery.kind != SceneryKind::Ring {
            continue;
        }
        let progress = (scenery.collect_timer / RING_COLLECT_TIME).clamp(0.0, 1.0);
        let pulse = if scenery.collected {
            progress * RING_GROW
        } else {
            RING_PULSE_AMOUNT * (game.elapsed * RING_PULSE_SPEED + scenery.pulse_phase).sin()
        };
        let scale = RING_RADIUS * (1.0 + pulse);
        let fade = if scenery.collected {
            1.0 - progress
        } else {
            1.0
        };
        let position = scenery.position;
        let model = [
            [scale, 0.0, 0.0, 0.0],
            [0.0, scale, 0.0, 0.0],
            [0.0, 0.0, scale, 0.0],
            [position.x, position.y, position.z, 1.0],
        ];
        instances.push(RingInstance {
            model,
            color: [0.5, 0.95, 1.0, fade],
        });
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
    time: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

const MAX_INSTANCES: usize = 64;
const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const SHADER: &str = r#"
struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    time: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,
    @location(7) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(input.model_0, input.model_1, input.model_2, input.model_3);
    let world_position = model * vec4<f32>(input.position, 1.0);
    var output: VertexOutput;
    output.clip_position = uniforms.projection * uniforms.view * world_position;
    output.world_position = world_position.xyz;
    output.world_normal = normalize((model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time.x;
    let view_dir = normalize(uniforms.camera_position.xyz - input.world_position);
    let rim = pow(1.0 - abs(dot(view_dir, normalize(input.world_normal))), 1.2);
    let bands = sin((input.uv.x * 9.0 - time * 2.5) * 6.2831853) * 0.5 + 0.5;
    let pulse = 0.85 + 0.15 * sin(time * 3.0);
    let energy = (0.85 + 0.35 * bands) * pulse;
    let fade = input.color.a;
    let glow = input.color.rgb * (energy * 3.6 + rim * 1.6) * fade;
    return vec4<f32>(glow, fade);
}
"#;

pub struct RingFxPass {
    shared: Arc<Mutex<RingState>>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl RingFxPass {
    pub fn new(device: &wgpu::Device, shared: Arc<Mutex<RingState>>) -> Self {
        let (vertices, indices) = torus(1.0, 0.09, 64, 16);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ring_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ring_indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_count = indices.len() as u32;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ring_instances"),
            size: (std::mem::size_of::<RingInstance>() * MAX_INSTANCES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ring_uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ring_bgl"),
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
            label: Some("ring_bg"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ring_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ring_layout"),
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
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RingInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ring_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vertex_main"),
                buffers: &[vertex_layout, instance_layout],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
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

impl nightshade::render::wgpu::rendergraph::PassNode<World> for RingFxPass {
    fn name(&self) -> &str {
        "ring_fx_pass"
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
        let time = configs.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
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
                time: [time, 0.0, 0.0, 0.0],
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
                label: Some("ring_fx_pass"),
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

fn torus(
    major: f32,
    tube: f32,
    major_segments: usize,
    tube_segments: usize,
) -> (Vec<GpuVertex>, Vec<u32>) {
    let stride = tube_segments + 1;
    let mut vertices = Vec::with_capacity((major_segments + 1) * stride);
    for major_index in 0..=major_segments {
        let major_angle = std::f32::consts::TAU * major_index as f32 / major_segments as f32;
        let (major_sin, major_cos) = major_angle.sin_cos();
        for tube_index in 0..=tube_segments {
            let tube_angle = std::f32::consts::TAU * tube_index as f32 / tube_segments as f32;
            let (tube_sin, tube_cos) = tube_angle.sin_cos();
            let ring = major + tube * tube_cos;
            vertices.push(GpuVertex {
                position: [ring * major_cos, ring * major_sin, tube * tube_sin],
                normal: [tube_cos * major_cos, tube_cos * major_sin, tube_sin],
                uv: [
                    major_index as f32 / major_segments as f32,
                    tube_index as f32 / tube_segments as f32,
                ],
            });
        }
    }
    let mut indices = Vec::new();
    for major_index in 0..major_segments {
        for tube_index in 0..tube_segments {
            let a = (major_index * stride + tube_index) as u32;
            let b = ((major_index + 1) * stride + tube_index) as u32;
            let c = (major_index * stride + tube_index + 1) as u32;
            let d = ((major_index + 1) * stride + tube_index + 1) as u32;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(c);
            indices.push(b);
            indices.push(d);
        }
    }
    (vertices, indices)
}
