use glam::Vec3;

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

    pub fn parse(line: &str) -> Self {
        let lines = lines.lines().filter(|line| !line.trim().is_empty());

        let material = Material {
            ..Default::default()
        };

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();
            if let Some(word) = words.next() {
                match word {
                    "v" => positions.push(VertexPosition::parse(words)),
                    "vn" => normals.push(VertexNormal::parse(words)),
                    "vt" => textures.push(VertexTexture::parse(words)),
                    "mtllib" => {
                        mtllib = Some(Mtllib::parse(line));
                    }
                    "f" => faces.push(Face::parse(line, mtllib.clone())),
                    "o" => {
                        group = group.or(Some(Group::parse(line)));
                    }
                    _ => (),
                }
            }
        }

        material
    }
}
