pub mod material_lib;
pub mod obj_raw;
mod utils;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use glam::{Vec3, Vec4, Vec4Swizzles};
pub use material_lib::{material::Material, MaterialLib};
pub use obj_raw::ObjRaw;
use obj_raw::{
    face::{Face, VertexAttribute},
    SmoothingGroup,
};
use utils::calculate_tri_normal;

pub struct ObjAsset(Vec<[Vertex; 3]>);
impl ObjAsset {
    pub fn faces(&self) -> &Vec<[Vertex; 3]> {
        &self.0
    }
}

pub struct ObjAssetBuilder<'a> {
    obj_raw: &'a ObjRaw,
}
impl<'a> ObjAssetBuilder<'a> {
    pub fn new(raw: &'a ObjRaw) -> Self {
        Self { obj_raw: raw }
    }

    pub fn obj_raw(self, raw: &'a ObjRaw) -> Self {
        Self {
            obj_raw: raw,
            ..self
        }
    }

    pub fn build(&self) -> ObjAsset {
        let face_tris: Vec<(&Face, Vec<([&VertexAttribute; 3], Vec3)>)> = self.triangulate_faces();

        let mut hash_smooth: HashMap<(u32, u32), Vec3> = HashMap::new(); //(smoothing_group, vertex_index), acc_vertex_normal

        // populate hash_smooth
        for (face, tris) in &face_tris {
            match face.smoothing_group {
                SmoothingGroup::On(smoothing_group_id) => {
                    for (tri, normal) in tris {
                        tri.iter().for_each(|vertex_attribute| {
                            let vertex_normal =
                                self.vertex(vertex_attribute).normal.unwrap_or(*normal); // default to tri_normal
                            hash_smooth
                                .entry((smoothing_group_id, vertex_attribute.vertex_index))
                                .and_modify(|entry| {
                                    *entry += vertex_normal;
                                })
                                .or_insert(vertex_normal);
                        });
                    }
                }
                _ => {}
            }
        }

        // build vertex
        let tris: Vec<[Vertex; 3]> = face_tris
            .iter()
            .flat_map(|(face, tris)| {
                tris.iter()
                    .map(|(tri, normal)| {
                        tri.iter()
                            .map(|vertex_attribute| {
                                let vertex_normal = match face.smoothing_group {
                                    SmoothingGroup::On(smoothing_group_id) => hash_smooth
                                        .get(&(smoothing_group_id, vertex_attribute.vertex_index))
                                        .unwrap()
                                        .normalize(),
                                    SmoothingGroup::Off => {
                                        // default to tri_normal
                                        self.vertex(vertex_attribute).normal.unwrap_or(*normal)
                                    }
                                };

                                Vertex {
                                    normal: Some(vertex_normal.normalize()),
                                    ..self.vertex(vertex_attribute)
                                }
                            })
                            .collect::<Vec<Vertex>>()
                            .try_into()
                            .unwrap()
                    })
                    .collect::<Vec<[Vertex; 3]>>()
            })
            .collect();
        ObjAsset(tris)
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

    // fan triangulation (todo! incompatible with concave gon)
    fn triangulate_faces(&self) -> Vec<(&Face, Vec<([&VertexAttribute; 3], Vec3)>)> {
        self.obj_raw
            .faces
            .iter()
            .map(|face| {
                let mut vertices = face.vertex_attributes.iter();
                let fan_origin = vertices.next().unwrap();
                let fan_vertices: Vec<&VertexAttribute> = vertices.collect();

                let face_tris: Vec<([&VertexAttribute; 3], Vec3)> = fan_vertices
                    .windows(2)
                    .filter_map(|window| {
                        if let [b, c] = window {
                            let normal = calculate_tri_normal(
                                self.vertex(fan_origin).position.truncate(),
                                self.vertex(b).position.truncate(),
                                self.vertex(c).position.truncate(),
                            );
                            Some(([fan_origin, b, c], normal))
                        } else {
                            None
                        }
                    })
                    .collect();
                (face, face_tris)
            })
            .collect()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: Vec4,
    pub texture: Option<Vec3>,
    pub normal: Option<Vec3>,
}

pub fn load_materials(obj_raw: &ObjRaw) -> HashMap<String, MaterialLib> {
    let dirname = obj_raw.filepath.parent().unwrap();
    let mut material_libs = HashMap::<String, MaterialLib>::new(); //todo! rework uniqueness of sources (can be same file with different filepath)

    for material_lib_name in &obj_raw.material_libs {
        let filepath = dirname.join(material_lib_name);
        let material_lib = MaterialLib::load_from_file(&filepath);
        material_libs.insert(material_lib_name.clone(), material_lib);
    }

    material_libs
}
