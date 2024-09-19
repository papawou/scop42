use face::{Face, VertexAttribute};
use glam::{Vec3, Vec4};
use vertex_normal::VertexNormal;
use vertex_position::VertexPosition;
use vertex_texture::VertexTexture;

pub mod face;
pub mod vertex_normal;
pub mod vertex_position;
pub mod vertex_texture;

pub struct ObjRaw {
    pub group: String,
    pub faces: Vec<Face>,
    pub positions: Vec<VertexPosition>,
    pub textures: Vec<VertexTexture>,
    pub normals: Vec<VertexNormal>,
}

impl ObjRaw {
    pub fn parse(str: &str) -> Self {
        let lines = str.lines().filter(|line| !line.trim().is_empty());

        let mut faces: Vec<Face> = vec![];
        let mut positions: Vec<VertexPosition> = vec![];
        let mut textures: Vec<VertexTexture> = vec![];
        let mut normals: Vec<VertexNormal> = vec![];
        let mut group = String::new();

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();
            if let Some(word) = words.next() {
                match word {
                    "f" => faces.push(Face::parse(line)),
                    "v" => positions.push(VertexPosition::parse(line)),
                    "vt" => textures.push(VertexTexture::parse(line)),
                    "vn" => normals.push(VertexNormal::parse(line)),
                    "o" => {
                        group = words.collect::<Vec<&str>>().join(" ");
                    }
                    _ => (),
                }
            }
        }

        Self {
            faces,
            positions,
            textures,
            normals,
            group,
        }
    }

    pub fn load_from_file(path: &str) -> Self {
        let install_path = std::path::Path::new(path);
        let content = std::fs::read_to_string(install_path).unwrap();

        Self::parse(&content)
    }
}
