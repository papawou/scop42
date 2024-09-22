use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use face::{Face, VertexAttribute};
use glam::{Vec3, Vec4};
use group::Group;
use vertex_normal::VertexNormal;
use vertex_position::VertexPosition;
use vertex_texture::VertexTexture;

pub mod face;
mod group;
pub mod vertex_normal;
pub mod vertex_position;
pub mod vertex_texture;

pub struct ObjRaw {
    pub filepath: PathBuf,

    pub group: Option<Group>,
    pub faces: Vec<Face>,
    pub positions: Vec<VertexPosition>,
    pub textures: Vec<VertexTexture>,
    pub normals: Vec<VertexNormal>,

    pub material_libs: HashSet<String>,
}

impl ObjRaw {
    fn parse(filepath: &Path, data: &str) -> Self {
        let lines = data.lines().filter(|line| !line.trim().is_empty());

        let mut group: Option<Group> = None;
        let mut materials_lib = HashSet::<String>::new();

        let mut positions: Vec<VertexPosition> = vec![];
        let mut normals: Vec<VertexNormal> = vec![];
        let mut textures: Vec<VertexTexture> = vec![];

        let mut faces: Vec<Face> = vec![];

        let mut material_name: Option<String> = None;
        let mut smoothing_group: SmoothingGroup = SmoothingGroup::Off;

        for line in lines.map(|line| line.trim()) {
            let mut words = line.split_whitespace();

            if let Some(word) = words.next() {
                match word {
                    "#" => {}
                    "v" => positions.push(VertexPosition::parse(line)),
                    "vn" => normals.push(VertexNormal::parse(line)),
                    "vt" => textures.push(VertexTexture::parse(line)),
                    "mtllib" => {
                        for word in words.next() {
                            materials_lib.insert(word.to_string());
                        }
                    }
                    "s" => match words.next() {
                        Some(group) => match group {
                            "off" => smoothing_group = SmoothingGroup::Off,
                            _ => {
                                smoothing_group = SmoothingGroup::On(group.parse::<u32>().unwrap());
                            }
                        },
                        _ => panic!(),
                    },
                    "usemtl" => {
                        material_name = Some(words.next().unwrap().to_string());
                    }
                    "f" => faces.push(Face::parse(line, material_name.clone(), smoothing_group)),
                    "o" => {
                        group = group.or(Some(Group::parse(line)));
                    }
                    _ => {}
                }
            }
        }

        Self {
            filepath: filepath.to_path_buf(),
            faces,
            positions,
            textures,
            normals,
            group,
            material_libs: materials_lib,
        }
    }

    pub fn load_from_file(filepath: &Path) -> Self {
        let mut file = File::open(&filepath).unwrap();

        let mut data = String::new();
        file.read_to_string(&mut data);

        Self::parse(filepath, &data)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SmoothingGroup {
    On(u32),
    Off,
}
