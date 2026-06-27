use std::sync::{Arc, Mutex};

use crate::ecs::{GameMode, GameState};
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::prelude::*;

#[derive(Default)]
pub struct SynthState {
    pub enabled: bool,
}

pub fn sync(game: &GameState, shared: &Arc<Mutex<SynthState>>) {
    let mut state = shared.lock().unwrap();
    state.enabled = matches!(
        game.mode,
        GameMode::Playing | GameMode::Paused | GameMode::Cinematic
    );
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    inv_view_proj: [[f32; 4]; 4],
    camera: [f32; 4],
}

const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

const SHADER: &str = r#"
struct Uniforms {
    inv_view_proj: mat4x4<f32>,
    camera: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    let cx = x * 2.0 - 1.0;
    let cy = y * 2.0 - 1.0;
    out.position = vec4<f32>(cx, cy, 0.0, 1.0);
    out.ndc = vec2<f32>(cx, cy);
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, w.x), mix(c, d, w.x), w.y);
}

fn terrain(p: vec2<f32>) -> f32 {
    let valley = smoothstep(0.0, 9.0, abs(p.x));
    var h = 0.0;
    var amp = 1.0;
    var freq = 0.035;
    for (var o = 0; o < 5; o = o + 1) {
        let n = noise(p * freq);
        h = h + (1.0 - abs(n * 2.0 - 1.0)) * amp;
        amp = amp * 0.5;
        freq = freq * 2.0;
    }
    return h * 4.5 * valley;
}

const GROUND_Y: f32 = -9.0;
const MAX_HEIGHT: f32 = 9.0;

fn sky(rd: vec3<f32>) -> vec3<f32> {
    // Output only the sun on black so the Max blend keeps the real space sky.
    let sun_dir = normalize(vec3<f32>(0.0, 0.07, -1.0));
    let delta = distance(rd, sun_dir);
    let disc = smoothstep(0.36, 0.345, delta);
    let band = rd.y - sun_dir.y;
    let stripes = step(0.45, fract(band * 90.0));
    let cut = select(1.0, stripes, band < 0.0);
    let sun_color = mix(vec3<f32>(1.0, 0.2, 0.5), vec3<f32>(1.0, 0.78, 0.3), smoothstep(-0.04, 0.1, rd.y));
    var color = sun_color * disc * cut * 2.6;
    color = color + vec3<f32>(1.0, 0.3, 0.55) * smoothstep(0.58, 0.0, delta) * 0.14;

    return color;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = u.camera.w;
    let scroll = time * 6.0;
    let ro = u.camera.xyz;
    let far = u.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
    let world = far.xyz / far.w;
    let rd = normalize(world - ro);

    var color: vec3<f32>;
    var t = 0.0;
    var hit = false;
    var hit_pos = vec3<f32>(0.0);
    if (rd.y < -0.001) {
        t = max((ro.y - (GROUND_Y + MAX_HEIGHT)) / -rd.y, 0.0);
        for (var i = 0; i < 220; i = i + 1) {
            let pos = ro + rd * t;
            let height = GROUND_Y + terrain(vec2<f32>(pos.x, -pos.z + scroll));
            if (pos.y < height) {
                hit = true;
                hit_pos = pos;
                break;
            }
            t = t + max(0.22, t * 0.014);
            if (t > 650.0) {
                break;
            }
        }
    }

    if (hit) {
        let coord = vec2<f32>(hit_pos.x, -hit_pos.z + scroll) * 0.5;
        let derivative = fwidth(coord);
        let grid = abs(fract(coord - 0.5) - 0.5) / max(derivative, vec2<f32>(0.0015));
        let line = 1.0 - min(min(grid.x, grid.y), 1.0);
        let height01 = clamp((hit_pos.y - GROUND_Y) / MAX_HEIGHT, 0.0, 1.0);
        let magenta = vec3<f32>(0.95, 0.12, 0.62);
        let cyan = vec3<f32>(0.12, 0.72, 0.95);
        let base = mix(magenta, cyan, height01);
        let glow = pow(line, 1.5) * 2.6;
        color = base * 0.08 + base * glow;
        let dist = length(hit_pos.xz - ro.xz);
        let fog = smoothstep(70.0, 340.0, dist);
        color = mix(color, vec3<f32>(0.30, 0.06, 0.42), fog);
    } else {
        color = sky(rd);
    }

    return vec4<f32>(color, 1.0);
}
"#;

pub struct SynthwavePass {
    shared: Arc<Mutex<SynthState>>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl SynthwavePass {
    pub fn new(device: &wgpu::Device, shared: Arc<Mutex<SynthState>>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("synth_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("synth_uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("synth_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("synth_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("synth_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("synth_pipeline"),
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
                            operation: wgpu::BlendOperation::Max,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
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
            bind_group,
        }
    }
}

impl nightshade::render::wgpu::rendergraph::PassNode<World> for SynthwavePass {
    fn name(&self) -> &str {
        "synthwave_pass"
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

    fn execute<'r, 'e>(
        &mut self,
        context: nightshade::render::wgpu::rendergraph::PassExecutionContext<'r, 'e, World>,
    ) -> Result<
        Vec<nightshade::render::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        nightshade::render::wgpu::rendergraph::RenderGraphError,
    > {
        if !context.is_pass_enabled() || !self.shared.lock().unwrap().enabled {
            return Ok(context.into_sub_graph_commands());
        }

        let Some(matrices) = query_active_camera_matrices(context.configs) else {
            return Ok(context.into_sub_graph_commands());
        };
        let view_proj = matrices.projection * matrices.view;
        let inverse = nalgebra_glm::inverse(&view_proj);
        let time = context.configs.resources.window.timing.uptime_milliseconds as f32 / 1000.0;
        let position = matrices.camera_position;
        let uniforms = GpuUniforms {
            inv_view_proj: mat_to_array(&inverse),
            camera: [position.x, position.y, position.z, time],
        };
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let (color_view, load_op, store_op) = context.get_color_attachment("hdr")?;
        let depth_view = context.get_texture_view("depth")?;
        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("synthwave_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
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
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
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
