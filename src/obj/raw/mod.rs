use face::Face;
use vertex_normal::VertexNormal;
use vertex_texture::VertexTexture;
use vertice::Vertex;

pub mod face;
pub mod vertex_normal;
pub mod vertex_texture;
pub mod vertice;

pub struct ObjRaw {
    pub group: String,
    pub faces: Vec<Face>,
    pub vertices: Vec<Vertex>,
    pub textures: Vec<VertexTexture>,
    pub normals: Vec<VertexNormal>,
}

impl ObjRaw {
    pub fn parse(str: &str) -> Self {
        let lines = str.lines().filter(|line| !line.trim().is_empty());

        let mut faces: Vec<Face> = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut textures: Vec<VertexTexture> = vec![];
        let mut normals: Vec<VertexNormal> = vec![];
        let mut group = String::new();

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();
            if let Some(word) = words.next() {
                match word {
                    "f" => faces.push(Face::parse(line)),
                    "v" => vertices.push(Vertex::parse(line)),
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
            vertices,
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
