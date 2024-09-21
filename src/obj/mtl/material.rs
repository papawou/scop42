use glam::Vec3;

use crate::obj::utils;

#[derive(Debug, Default)]
pub struct Material {
    pub group: String, // newmtl (Material Group Name)

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

impl Material {
    pub fn new(group: &str) -> Self {
        Material {
            group: group.to_string(),
            ..Default::default()
        }
    }

    pub fn parse(&mut self, line: &str) {
        let mut words = line.split_whitespace();

        if let Some(word) = words.next() {
            match word {
                "Ns" => {
                    if let Some(value) = words.next() {
                        self.shininess_exponent = value.parse::<f32>().unwrap_or(0.0);
                    }
                }
                "Ka" => {
                    self.ambient = utils::parse_vec3_with_default(&mut words, Some(Vec3::ZERO));
                }
                "Kd" => {
                    self.diffuse = utils::parse_vec3_with_default(&mut words, Some(Vec3::ZERO));
                }
                "Ks" => {
                    self.specular = utils::parse_vec3_with_default(&mut words, Some(Vec3::ZERO));
                }
                "Ke" => {
                    self.emission = utils::parse_vec3_with_default(&mut words, Some(Vec3::ZERO));
                }
                "Ni" => {
                    if let Some(value) = words.next() {
                        self.optical_density = value.parse::<f32>().unwrap_or(1.0);
                    }
                }
                "d" => {
                    if let Some(value) = words.next() {
                        self.dissolve = value.parse::<f32>().unwrap_or(1.0);
                    }
                }
                "illum" => {
                    if let Some(value) = words.next() {
                        self.illumination = value.parse::<i32>().unwrap_or(0);
                    }
                }
                "map_Ka" => {
                    self.ambient_map = words.next().map(|s| s.to_string());
                }
                "map_Kd" => {
                    self.diffuse_map = words.next().map(|s| s.to_string());
                }
                "map_Ks" => {
                    self.specular_map = words.next().map(|s| s.to_string());
                }
                "map_Ns" => {
                    self.optical_density_map = words.next().map(|s| s.to_string());
                }
                "map_d" => {
                    self.dissolve_map = words.next().map(|s| s.to_string());
                }
                "disp" => {
                    self.displacement_map = words.next().map(|s| s.to_string());
                }
                "decal" => {
                    self.decal_map = words.next().map(|s| s.to_string());
                }
                "bump" | "map_Bump" => {
                    self.bump_map = words.next().map(|s| s.to_string());
                }
                // Add more cases here if needed
                _ => (),
            }
        }
    }
}
