use std::sync::{Arc, Mutex};

use crate::ecs::{GameMode, GameState};
use nightshade::ecs::camera::queries::query_active_camera_matrices;
use nightshade::prelude::*;

#[derive(Default)]
pub struct SynthState {
    pub enabled: bool,
    pub style: u32,
    pub curve_x: f32,
    pub curve_y: f32,
    pub scroll_time: f32,
}

pub fn sync(game: &GameState, shared: &Arc<Mutex<SynthState>>) {
    let mut state = shared.lock().unwrap();
    state.enabled = matches!(
        game.mode,
        GameMode::Playing | GameMode::Paused | GameMode::Cinematic
    );
    state.style = game.sector as u32;
    state.curve_x = game.curve_x;
    state.curve_y = game.curve_y;
    state.scroll_time = game.course_time;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuUniforms {
    inv_view_proj: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    camera: [f32; 4],
    extra: [f32; 4],
}

const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

const SHADER: &str = r#"
struct Uniforms {
    inv_view_proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    camera: vec4<f32>,
    extra: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var scene_copy: texture_2d<f32>;

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

fn ridged(p: vec2<f32>) -> f32 {
    var h = 0.0;
    var amp = 1.0;
    var freq = 1.0;
    for (var o = 0; o < 5; o = o + 1) {
        h = h + (1.0 - abs(noise(p * freq) * 2.0 - 1.0)) * amp;
        amp = amp * 0.5;
        freq = freq * 2.0;
    }
    return h;
}

const GROUND_Y: f32 = -9.0;
const MAX_HEIGHT: f32 = 9.0;

fn terrain(p: vec2<f32>, style: i32) -> f32 {
    let valley = smoothstep(0.0, 9.0, abs(p.x));
    if (style == 1) {
        let h = ridged(p * 0.05);
        return (h * 5.0 - 1.4) * valley;
    } else if (style == 2) {
        let base = ridged(p * 0.04);
        let spike = pow(ridged(p * 0.1 + vec2<f32>(7.0, 3.0)), 2.2) * 3.6;
        return (base * 2.0 + spike) * valley;
    }
    return ridged(p * 0.035) * 4.5 * valley;
}

fn surface(hit_pos: vec3<f32>, ro: vec3<f32>, style: i32, scroll: f32) -> vec3<f32> {
    let height01 = clamp((hit_pos.y - GROUND_Y) / MAX_HEIGHT, 0.0, 1.0);
    let dist = length(hit_pos.xz - ro.xz);
    let fog = smoothstep(70.0, 340.0, dist);

    if (style == 1) {
        let coord = vec2<f32>(hit_pos.x, -hit_pos.z + scroll);
        let crack = pow(1.0 - height01, 2.4);
        let vein = smoothstep(0.42, 0.52, noise(coord * 0.25));
        let rock = vec3<f32>(0.10, 0.07, 0.07);
        let lava = vec3<f32>(4.2, 0.9, 0.12);
        var col = mix(rock, lava, clamp(crack * 0.7 + vein * crack, 0.0, 1.0));
        col = col + lava * pow(crack, 4.0) * 0.6;
        return mix(col, vec3<f32>(0.24, 0.05, 0.03), fog);
    } else if (style == 2) {
        let coord = vec2<f32>(hit_pos.x, -hit_pos.z + scroll) * 0.5;
        let deriv = vec2<f32>(max(0.0018, dist * 0.00065));
        let grid = abs(fract(coord - 0.5) - 0.5) / max(deriv, vec2<f32>(0.002));
        let line = 1.0 - min(min(grid.x, grid.y), 1.0);
        let base = mix(vec3<f32>(0.1, 0.25, 0.6), vec3<f32>(0.72, 0.45, 1.0), height01);
        let glow = pow(line, 1.5) * 2.2;
        var col = base * 0.16 + base * glow + vec3<f32>(0.3, 0.6, 1.0) * pow(height01, 3.0) * 1.6;
        return mix(col, vec3<f32>(0.12, 0.1, 0.32), fog);
    }

    let coord = vec2<f32>(hit_pos.x, -hit_pos.z + scroll) * 0.5;
    let deriv = vec2<f32>(max(0.0015, dist * 0.00065));
    let grid = abs(fract(coord - 0.5) - 0.5) / max(deriv, vec2<f32>(0.0015));
    let line = 1.0 - min(min(grid.x, grid.y), 1.0);
    let base = mix(vec3<f32>(0.95, 0.12, 0.62), vec3<f32>(0.12, 0.72, 0.95), height01);
    let glow = pow(line, 1.5) * 2.6;
    var col = base * 0.08 + base * glow;
    return mix(col, vec3<f32>(0.30, 0.06, 0.42), fog);
}

fn foreground(bg: vec3<f32>) -> vec3<f32> {
    let lum = max(bg.r, max(bg.g, bg.b));
    let mask = smoothstep(1.2, 2.6, lum);
    return bg * mask;
}

fn ground_fog(style: i32) -> vec3<f32> {
    if (style == 1) {
        return vec3<f32>(0.24, 0.05, 0.03);
    } else if (style == 2) {
        return vec3<f32>(0.12, 0.1, 0.32);
    }
    return vec3<f32>(0.30, 0.06, 0.42);
}

fn sky(rd: vec3<f32>, style: i32) -> vec3<f32> {
    if (style == 1) {
        let sun_dir = normalize(vec3<f32>(0.0, 0.05, -1.0));
        let d = distance(rd, sun_dir);
        let disc = 1.0 - smoothstep(0.285, 0.30, d);
        var c = vec3<f32>(3.2, 0.5, 0.08) * disc;
        c = c + vec3<f32>(1.0, 0.2, 0.04) * (1.0 - smoothstep(0.0, 0.62, d)) * 0.22;
        return c;
    } else if (style == 2) {
        let up = clamp(rd.y, 0.0, 1.0);
        let wave = sin(rd.x * 6.0 + rd.y * 3.0) * 0.5 + 0.5;
        let aurora_color = mix(vec3<f32>(0.1, 0.8, 0.55), vec3<f32>(0.5, 0.2, 0.95), sin(rd.x * 4.0) * 0.5 + 0.5);
        var c = aurora_color * wave * smoothstep(0.18, 0.85, up) * 0.6;
        let moon_dir = normalize(vec3<f32>(0.0, 0.16, -1.0));
        let dm = distance(rd, moon_dir);
        c = c + vec3<f32>(0.8, 0.85, 1.0) * (1.0 - smoothstep(0.15, 0.16, dm)) * 1.6;
        return c;
    }
    let sun_dir = normalize(vec3<f32>(0.0, 0.07, -1.0));
    let d = distance(rd, sun_dir);
    let disc = 1.0 - smoothstep(0.345, 0.36, d);
    let band = rd.y - sun_dir.y;
    let stripes = step(0.45, fract(band * 90.0));
    let cut = select(1.0, stripes, band < 0.0);
    let sun_color = mix(vec3<f32>(1.0, 0.2, 0.5), vec3<f32>(1.0, 0.78, 0.3), smoothstep(-0.04, 0.1, rd.y));
    var c = sun_color * disc * cut * 2.6;
    c = c + vec3<f32>(1.0, 0.3, 0.55) * (1.0 - smoothstep(0.0, 0.58, d)) * 0.14;
    return c;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scroll = u.camera.w;
    let style = i32(u.extra.x);
    let ro = u.camera.xyz;
    let far = u.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
    let world = far.xyz / far.w;
    let rd = normalize(world - ro);

    let pixel = vec2<i32>(in.position.xy);
    let scene_depth = textureLoad(depth_tex, pixel, 0);
    let has_geometry = scene_depth > 0.0;
    let fg = foreground(textureLoad(scene_copy, pixel, 0).rgb);

    var color: vec3<f32>;
    var t = 0.0;
    var hit = false;
    var hit_pos = vec3<f32>(0.0);
    if (rd.y < -0.001) {
        t = max((ro.y - (GROUND_Y + MAX_HEIGHT)) / -rd.y, 0.0);
        for (var i = 0; i < 320; i = i + 1) {
            let pos = ro + rd * t;
            let depth = max(0.0, -pos.z);
            let bx = u.extra.y * depth * depth;
            let by = u.extra.z * depth * depth;
            let height = GROUND_Y + by + terrain(vec2<f32>(pos.x - bx, -pos.z + scroll), style);
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
        let clip = u.view_proj * vec4<f32>(hit_pos, 1.0);
        let terrain_depth = clip.z / clip.w;
        if (has_geometry && terrain_depth <= scene_depth) {
            return vec4<f32>(0.0);
        }
        let hd = max(0.0, -hit_pos.z);
        let hbx = u.extra.y * hd * hd;
        let hby = u.extra.z * hd * hd;
        color = surface(vec3<f32>(hit_pos.x - hbx, hit_pos.y - hby, hit_pos.z), ro, style, scroll);
        if (has_geometry) {
            return vec4<f32>(color, 1.0);
        }
        return vec4<f32>(color + fg, 1.0);
    }

    if (rd.y < -0.001) {
        if (has_geometry) {
            return vec4<f32>(0.0);
        }
        return vec4<f32>(ground_fog(style) + fg, 1.0);
    }

    if (has_geometry) {
        return vec4<f32>(0.0);
    }
    color = sky(rd, style);
    return vec4<f32>(color, 0.0);
}
"#;

pub struct SynthwavePass {
    shared: Arc<Mutex<SynthState>>,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl SynthwavePass {
    pub fn new(device: &wgpu::Device, shared: Arc<Mutex<SynthState>>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("synth_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("synth_uniforms"),
            size: std::mem::size_of::<GpuUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: HDR_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
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
            bind_group_layout,
        }
    }
}

impl nightshade::render::wgpu::rendergraph::PassNode<World> for SynthwavePass {
    fn name(&self) -> &str {
        "synthwave_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["depth", "scene_copy"]
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
        let (enabled, style, curve_x, curve_y, scroll_time) = {
            let state = self.shared.lock().unwrap();
            (
                state.enabled,
                state.style,
                state.curve_x,
                state.curve_y,
                state.scroll_time,
            )
        };
        if !context.is_pass_enabled() || !enabled {
            return Ok(context.into_sub_graph_commands());
        }

        let Some(matrices) = query_active_camera_matrices(context.configs) else {
            return Ok(context.into_sub_graph_commands());
        };
        let view_proj = matrices.projection * matrices.view;
        let inverse = nalgebra_glm::inverse(&view_proj);
        let position = matrices.camera_position;
        let uniforms = GpuUniforms {
            inv_view_proj: mat_to_array(&inverse),
            view_proj: mat_to_array(&view_proj),
            camera: [position.x, position.y, position.z, scroll_time],
            extra: [style as f32, curve_x, curve_y, 0.0],
        };
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let depth_view = context.get_texture_view("depth")?;
        let scene_copy_view = context.get_texture_view("scene_copy")?;
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("synth_bg"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scene_copy_view),
                    },
                ],
            });

        let (color_view, load_op, store_op) = context.get_color_attachment("hdr")?;
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

fn mat_to_array(matrix: &nalgebra_glm::Mat4) -> [[f32; 4]; 4] {
    let slice = matrix.as_slice();
    [
        [slice[0], slice[1], slice[2], slice[3]],
        [slice[4], slice[5], slice[6], slice[7]],
        [slice[8], slice[9], slice[10], slice[11]],
        [slice[12], slice[13], slice[14], slice[15]],
    ]
}

const COPY_SHADER: &str = r#"
@group(0) @binding(0) var src_tex: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureLoad(src_tex, vec2<i32>(in.position.xy), 0);
}
"#;

pub struct SceneCopyPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl SceneCopyPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene_copy_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene_copy_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(COPY_SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene_copy_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene_copy_pipeline"),
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

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl nightshade::render::wgpu::rendergraph::PassNode<World> for SceneCopyPass {
    fn name(&self) -> &str {
        "synth_scene_copy_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["input"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
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

        let input_view = context.get_texture_view("input")?;
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene_copy_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                }],
            });

        let (color_view, _, store_op) = context.get_color_attachment("output")?;
        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("synth_scene_copy_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
