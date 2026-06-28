use nightshade::ecs::loading::load_texture_pack_from_image_bytes;
use nightshade::ecs::material::components::{Material, TextureTransform};
use nightshade::prelude::*;
use nightshade::render::wgpu::texture_cache::{SamplerSettings, TextureUsage};

const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        "proto_dark_06",
        include_bytes!("../../assets/textures/prototype/dark/texture_06.png"),
    ),
    (
        "proto_dark_03",
        include_bytes!("../../assets/textures/prototype/dark/texture_03.png"),
    ),
    (
        "proto_light_01",
        include_bytes!("../../assets/textures/prototype/light/texture_01.png"),
    ),
    (
        "proto_light_03",
        include_bytes!("../../assets/textures/prototype/light/texture_03.png"),
    ),
    (
        "proto_orange_01",
        include_bytes!("../../assets/textures/prototype/orange/texture_01.png"),
    ),
    (
        "proto_purple_02",
        include_bytes!("../../assets/textures/prototype/purple/texture_02.png"),
    ),
];

pub fn load(world: &mut World) {
    load_texture_pack_from_image_bytes(
        world,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
}

pub fn proto_material(texture: &str, tint: Vec3, tiling: f32) -> Material {
    Material {
        base_color: [tint.x, tint.y, tint.z, 1.0],
        base_texture: Some(texture.to_string()),
        base_texture_transform: TextureTransform {
            scale: [tiling, tiling],
            ..Default::default()
        },
        unlit: true,
        ..Default::default()
    }
}
