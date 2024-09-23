mod material_lib;
pub mod obj_raw;
mod utils;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use glam::{Vec3, Vec4};
use material_lib::MaterialLib;
use obj_raw::face::VertexAttribute;
pub use obj_raw::ObjRaw;

pub struct ObjAsset(Vec<Vec<Vertex>>);
impl ObjAsset {
    pub fn faces(&self) -> &Vec<Vec<Vertex>> {
        &self.0
    }
}

pub struct ObjAssetBuilder<'a> {
    obj_raw: &'a ObjRaw,
    normals_from_face: bool,
    material_libs: HashMap<String, MaterialLib>,
}
impl<'a> ObjAssetBuilder<'a> {
    pub fn new(raw: &'a ObjRaw) -> Self {
        let material_libs = parse_materials(raw);

        Self {
            obj_raw: raw,
            normals_from_face: false,
            material_libs,
        }
    }

    pub fn obj_raw(self, raw: &'a ObjRaw) -> Self {
        Self {
            obj_raw: raw,
            ..self
        }
    }

    pub fn normals_from_face(self, normals_from_face: bool) -> Self {
        Self {
            normals_from_face,
            ..self
        }
    }

    pub fn build(self) -> ObjAsset {
        // generate faces
        let mut faces: Vec<Vec<Vertex>> = vec![];
        for face in &self.obj_raw.faces {
            let face_vertices: Vec<Vertex> = face
                .vertex_attributes
                .iter()
                .map(|vertex_attribute| Vertex {
                    ..self.vertex(vertex_attribute)
                })
                .collect();
            faces.push(face_vertices);
        }

        ObjAsset(faces)
    }

    fn vertex(&self, vertex_attribute: &VertexAttribute) -> Vertex {
        let position = self
            .obj_raw
            .positions
            .get(vertex_attribute.vertex_index as usize)
            .unwrap()
            .position();

        let texture = vertex_attribute
            .vertex_texture_index
            .and_then(|vertex_texture_index| {
                Some(
                    self.obj_raw
                        .textures
                        .get(vertex_texture_index as usize)
                        .unwrap()
                        .uv(),
                )
            });

        let normal = vertex_attribute
            .vertex_normal_index
            .and_then(|vertex_normal_index| {
                Some(
                    self.obj_raw
                        .normals
                        .get(vertex_normal_index as usize)
                        .unwrap()
                        .normal(),
                )
            });

        Vertex {
            position,
            texture,
            normal,
        }
    }

    fn calculate_normals(&self) -> HashMap<usize, Vec3> {
        let mut normal_map: HashMap<usize, Vec3> = HashMap::new();

        for face in &self.obj_raw.faces {
            for tri in face.vertex_attributes.windows(3) {
                let tri: Vec<(u32, Vertex)> = tri
                    .iter()
                    .map(|vertex| (vertex.vertex_index, self.vertex(vertex)))
                    .collect();

                let [(a_index, a), (b_index, b), (c_index, c)] = tri.as_slice() else {
                    continue;
                };

                // Calculate tri normal
                let normal = {
                    let edge_a = b.position - a.position;
                    let edge_b = c.position - a.position;
                    edge_a.truncate().cross(edge_b.truncate()).normalize()
                };

                normal_map
                    .entry(*a_index as usize)
                    .and_modify(|n| *n += normal)
                    .or_insert(normal);
                normal_map
                    .entry(*b_index as usize)
                    .and_modify(|n| *n += normal)
                    .or_insert(normal);
                normal_map
                    .entry(*c_index as usize)
                    .and_modify(|n| *n += normal)
                    .or_insert(normal);
            }
        }

        normal_map
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: Vec4,
    pub texture: Option<Vec3>,
    pub normal: Option<Vec3>,
}

fn parse_materials(raw: &ObjRaw) -> HashMap<String, MaterialLib> {
    let dirname = raw.filepath.parent().unwrap();
    let mut material_libs = HashMap::<String, MaterialLib>::new();

    for material_lib_name in &raw.material_libs {
        let filepath = dirname.join(material_lib_name);
        let material_lib = MaterialLib::load_from_file(&filepath);
        material_libs.insert(material_lib_name.clone(), material_lib);
    }

    material_libs
}
