use glam::Vec3;

use crate::material_asset::MaterialAsset;

/**
 * struct MaterialParams
 * data is intended to be use with Vulkan directly
 * _pad for glsl interpreting a Vec3 as Vec4
 */
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Params {
    pub ambient: Vec3,
    pub shininess_exponent: f32,
    pub diffuse: Vec3,
    pub optical_density: f32,
    pub specular: Vec3,
    pub dissolve: f32,
    pub emission: Vec3,
    pub illumination: i32,
}

impl From<&MaterialAsset> for Params {
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
