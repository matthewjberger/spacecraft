use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::{SamplerSettings, TextureUsage};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 128;

#[derive(Clone, Copy)]
pub enum PlanetStyle {
    Banded,
    Blotchy,
    Star,
}

fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut value = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_246_822_519));
    value = (value ^ (value >> 13)).wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let smooth_x = xf * xf * (3.0 - 2.0 * xf);
    let smooth_y = yf * yf * (3.0 - 2.0 * yf);
    let a = hash(xi, yi, seed);
    let b = hash(xi + 1, yi, seed);
    let c = hash(xi, yi + 1, seed);
    let d = hash(xi + 1, yi + 1, seed);
    let top = a + (b - a) * smooth_x;
    let bottom = c + (d - c) * smooth_x;
    top + (bottom - top) * smooth_y
}

fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for _ in 0..5 {
        sum += value_noise(x * frequency, y * frequency, seed) * amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    sum
}

fn periodic_fbm(u: f32, v: f32, scale: f32, seed: u32) -> f32 {
    let angle = u * std::f32::consts::TAU;
    0.5 * (fbm(angle.cos() * scale + 11.0, v * scale, seed)
        + fbm(angle.sin() * scale + 23.0, v * scale, seed))
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

pub fn register(
    world: &mut World,
    name: &str,
    style: PlanetStyle,
    low: [f32; 3],
    high: [f32; 3],
    seed: u32,
) {
    let mut rgba = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    for py in 0..HEIGHT {
        let v = py as f32 / HEIGHT as f32;
        for px in 0..WIDTH {
            let u = px as f32 / WIDTH as f32;
            let color = match style {
                PlanetStyle::Banded => {
                    let warp = (periodic_fbm(u, v, 2.5, seed.wrapping_add(41)) - 0.5) * 0.55;
                    let latitude = v + warp;
                    let bands = (latitude * 5.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
                    let swirl =
                        periodic_fbm(u + warp * 0.4, latitude, 13.0, seed.wrapping_add(7)) - 0.5;
                    let storm = ((periodic_fbm(u, v, 7.0, seed.wrapping_add(9)) - 0.66) * 5.0)
                        .clamp(0.0, 1.0);
                    let tone = (bands * 0.6 + swirl * 0.7 + 0.2).clamp(0.0, 1.0);
                    let base = mix(low, high, tone);
                    mix(base, [1.0, 0.97, 0.9], storm * 0.35)
                }
                PlanetStyle::Blotchy => {
                    let landmass = periodic_fbm(u, v, 6.0, seed);
                    let land = ((landmass - 0.45) * 3.2).clamp(0.0, 1.0);
                    let ice = ((v - 0.5).abs() * 2.0).powi(3);
                    let surface = mix(low, high, land);
                    mix(surface, [0.92, 0.95, 1.0], (ice * 0.8).clamp(0.0, 0.75))
                }
                PlanetStyle::Star => {
                    let spots = periodic_fbm(u, v, 9.0, seed);
                    let darken = ((spots - 0.55) * 2.0).clamp(0.0, 0.55);
                    mix(high, low, darken)
                }
            };
            let index = ((py * WIDTH + px) * 4) as usize;
            rgba[index] = (color[0].clamp(0.0, 1.0) * 255.0) as u8;
            rgba[index + 1] = (color[1].clamp(0.0, 1.0) * 255.0) as u8;
            rgba[index + 2] = (color[2].clamp(0.0, 1.0) * 255.0) as u8;
            rgba[index + 3] = 255;
        }
    }
    nightshade::ecs::loading::queue_decoded_texture(
        world,
        name.to_string(),
        rgba,
        WIDTH,
        HEIGHT,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
    texture_cache_add_reference(&mut world.resources.texture_cache, name);
}
