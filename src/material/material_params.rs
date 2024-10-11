use glam::Vec3;

use crate::material_asset::MaterialAsset;

/**
 * struct MaterialParams
 * data is intended to be use with Vulkan directly
 * _pad for glsl interpreting a Vec3 as Vec4
 */
#[derive(Default, Debug)]
struct MaterialParams {
    pub shininess_exponent: f32,
    pub ambient: Vec3,
    _pad_1: f32,
    pub diffuse: Vec3,
    _pad_2: f32,
    pub specular: Vec3,
    _pad_3: f32,
    pub emission: Vec3,
    _pad_4: f32,
    pub optical_density: f32,
    pub dissolve: f32,
    pub illumination: i32,
}

impl From<&MaterialAsset> for MaterialParams {
    fn from(
        &MaterialAsset {
            ambient,
            diffuse,
            dissolve,
            optical_density,
            shininess_exponent,
            emission,
            specular,
            illumination,
            ..
        }: &MaterialAsset,
    ) -> Self {
        Self {
            ambient,
            diffuse,
            dissolve,
            optical_density,
            shininess_exponent,
            emission,
            specular,
            illumination,
            ..Default::default()
        }
    }
}
