pub mod raw;

use std::collections::{HashMap, HashSet};

use glam::{Vec3, Vec4};
use raw::face::VertexAttribute;
pub use raw::ObjRaw;

struct ObjAsset {
    faces: Vec<Vec<Vertex>>,
    indices: Vec<usize>,
}

struct ObjAssetBuilder<'a> {
    obj_raw: &'a ObjRaw,
    normals_from_face: bool,
}

impl<'a> ObjAssetBuilder<'a> {
    fn normals_from_face(self, normals_from_face: bool) -> Self {
        Self {
            normals_from_face,
            ..self
        }
    }

    fn build(self) -> ObjAsset {
        let mut faces: Vec<Vec<&VertexAttribute>> = vec![];
        let mut indices: Vec<usize> = vec![];
        let mut indice: usize = 0;

        //normals_from_face
        let mut normal_map: HashMap<usize, Vec3> = HashMap::new();

        for face in &self.obj_raw.faces {
            let mut vertices: Vec<&VertexAttribute> = vec![];

            for vertex_attribute in face.vertex_attributes.iter() {
                indices.push(indice);
                vertices.push(vertex_attribute);
                indice += 1;
            }

            //normals_from_face
            for tri in vertices.windows(3) {
                let [(a_index, a), (b_index, b), (c_index, c)] = tri
                    .iter()
                    .map(|vertex| (vertex.vertex_index, self.vertex(vertex)))
                    .collect::<Vec<(u32, Vertex)>>()
                    .as_slice()
                else {
                    continue;
                };

                let normal = {
                    let edge_a = a.position - a.position;
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

            faces.push(vertices);
        }

        

        ObjAsset {}
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
}

struct Vertex {
    position: Vec4,
    texture: Option<Vec3>,
    normal: Option<Vec3>,
}
