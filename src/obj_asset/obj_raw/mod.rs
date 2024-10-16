use std::{
    collections::{HashMap, HashSet},
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

use crate::vertex;

pub mod face;
mod group;
pub mod vertex_normal;
pub mod vertex_position;
pub mod vertex_texture;

#[derive(Clone)]
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
        let mut material_libs = HashSet::<String>::new();

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
                            match word {
                                "None" => {} // ignore mtllib None
                                _ => {
                                    material_libs.insert(word.to_string());
                                }
                            };
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
            material_libs,
        }
    }

    pub fn load_from_file(filepath: &Path) -> Self {
        let mut file = File::open(&filepath).unwrap();

        let mut data = String::new();
        file.read_to_string(&mut data);

        Self::parse(filepath, &data)
    }

    // /**
    //  * If some vertex_position are duplicated, vertex_attributes use the same vertex_index
    //  *
    //  * tothink! use a generic read method ? (object will stay same, only the way we read it change it)
    //  */
    pub fn optimise_positions(&self) -> Self {
        let find_first_vertex_pos = |vertex_index| -> u32 {
            let initial_pos = self.positions.get(vertex_index as usize).unwrap();

            self.positions
                .iter()
                .position(|pos| pos.position() == initial_pos.position())
                .unwrap() as u32
        };

        let faces: Vec<Face> = self
            .faces
            .iter()
            .map(|face| {
                let vertex_attributes = face
                    .vertex_attributes
                    .iter()
                    .map(|vertex_attribute| VertexAttribute {
                        vertex_index: find_first_vertex_pos(vertex_attribute.vertex_index),
                        ..(*vertex_attribute).clone()
                    })
                    .collect();
                Face {
                    vertex_attributes,
                    ..(*face).clone()
                }
            })
            .collect();
        Self {
            faces,
            ..(*self).clone()
        }
    }

    pub fn find_first_position_index() {}
}

#[derive(Debug, Clone, Copy)]
pub enum SmoothingGroup {
    On(u32),
    Off,
}
