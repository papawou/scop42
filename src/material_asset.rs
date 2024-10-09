use glam::Vec3;

use crate::obj_asset;

pub struct MaterialAsset {
    pub material_name: String, // newmtl (Material Group Name)

    pub shininess_exponent: f32, // Ns (Shininess Exponent)
    pub ambient: Vec3,           // Ka (Ambient RGB)
    pub diffuse: Vec3,           // Kd (Diffuse RGB)
    pub specular: Vec3,          // Ks (Specular RGB)
    pub emission: Vec3,          // Ke (Emission RGB)
    pub optical_density: f32,    // Ni (Optical Density)
    pub dissolve: f32,           // d (Dissolve)
    pub illumination: i32,       // illum (Illumination Model)

    // Texture maps
    pub ambient_map: Option<String>,         // map_Ka
    pub diffuse_map: Option<String>,         // map_Kd
    pub specular_map: Option<String>,        // map_Ks
    pub optical_density_map: Option<String>, // map_Ns
    pub dissolve_map: Option<String>,        // map_d
    pub displacement_map: Option<String>,    // disp
    pub decal_map: Option<String>,           // decal
    pub bump_map: Option<String>,            // bump
}

impl From<obj_asset::Material> for MaterialAsset {
    fn from(value: obj_asset::Material) -> Self {
        let obj_asset::Material {
            ambient,
            ambient_map,
            bump_map,
            decal_map,
            diffuse,
            diffuse_map,
            displacement_map,
            dissolve,
            dissolve_map,
            emission,
            illumination,
            material_name,
            optical_density,
            optical_density_map,
            shininess_exponent,
            specular,
            specular_map,
        } = value;

        Self {
            ambient,
            ambient_map,
            bump_map,
            decal_map,
            diffuse,
            diffuse_map,
            displacement_map,
            dissolve,
            dissolve_map,
            emission,
            illumination,
            material_name,
            optical_density,
            optical_density_map,
            shininess_exponent,
            specular,
            specular_map,
        }
    }
}
