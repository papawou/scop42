mod mtl;
pub mod raw;
use std::collections::{HashMap, HashSet};

use glam::{Vec3, Vec4};
use raw::face::VertexAttribute;
pub use raw::ObjRaw;

pub struct ObjAsset(Vec<Vec<Vertex>>);

impl ObjAsset {
    pub fn faces(&self) -> &Vec<Vec<Vertex>> {
        &self.0
    }
}

pub struct ObjAssetBuilder<'a> {
    obj_raw: &'a ObjRaw,
    normals_from_face: bool,
}

impl<'a> ObjAssetBuilder<'a> {
    pub fn new(raw: &'a ObjRaw) -> ObjAssetBuilder<'a> {
        ObjAssetBuilder {
            obj_raw: raw,
            normals_from_face: false,
        }
    }

    pub fn obj_raw(self, raw: &'a ObjRaw) -> ObjAssetBuilder<'a> {
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
        //normals_from_face
        let normal_map = self.get_normals_from_face();

        // generate faces
        let mut faces: Vec<Vec<Vertex>> = vec![];
        for face in &self.obj_raw.faces {
            let tri: Vec<Vertex> = face
                .vertex_attributes
                .iter()
                .map(|vertex_attribute| Vertex {
                    normal: {
                        let vertex_index = vertex_attribute.vertex_index as usize;

                        //normals_from_face
                        if self.normals_from_face {
                            normal_map
                                .get(&vertex_index)
                                .and_then(|normal| Some(normal.normalize()))
                        } else {
                            self.vertex(vertex_attribute).normal
                        }
                    },
                    ..self.vertex(vertex_attribute)
                })
                .collect();
            faces.push(tri);
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

    fn get_normals_from_face(&self) -> HashMap<usize, Vec3> {
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
