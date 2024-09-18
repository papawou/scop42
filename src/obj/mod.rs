pub mod raw;

use glam::{Vec3, Vec4};
pub use raw::ObjRaw;

struct Vertex {
    position: Vec4,
    texture: Option<Vec3>,
    normal: Option<Vec3>,
}

struct ObjAsset {
    faces: Vec<Vec<Vertex>>,
    indices: Vec<usize>,
}

struct ObjAssetBuilder<'a> {
    obj_raw: &'a ObjRaw,
    normals_from_face: bool,
}

impl<'a> ObjAssetBuilder<'a> {
    fn new(obj_raw: &'a ObjRaw) -> Self {
        Self {
            obj_raw,
            normals_from_face: false,
        }
    }

    fn normals_from_face(self, normals_from_face: bool) -> Self {
        Self {
            normals_from_face,
            ..self
        }
    }

    fn build(self) -> ObjAsset {
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        let mut indice: u32 = 0;
        for face in &self.obj_raw.faces {
            for vertex_attribute in &face.vertex_attributes {
                indice += 1;
                indices.push(indice);

                vertices.push(Vertex {
                    position: self
                        .obj_raw
                        .vertices
                        .get(vertex_attribute.vertex_index as usize)
                        .and_then(|vertex| {
                            Some(glam::Vec3 {
                                x: vertex.x,
                                y: vertex.y,
                                z: vertex.z,
                            })
                        })
                        .unwrap(),
                    normal: if let Some(normal_index) = vertex_attribute.vertex_normal_index {
                        self.obj_raw
                            .textures
                            .get(normal_index as usize)
                            .and_then(|normal| {
                                Some(glam::Vec3 {
                                    x: normal.x,
                                    y: normal.y,
                                    z: normal.z,
                                })
                            })
                            .unwrap()
                    } else {
                        glam::Vec3::NAN
                    },
                    uv_x: if let Some(texture_index) = vertex_attribute.vertex_texture_index {
                        self.obj_raw
                            .textures
                            .get(texture_index as usize)
                            .and_then(|texture_uv| Some(texture_uv.x))
                            .unwrap()
                    } else {
                        0.0f32
                    },
                    uv_y: if let Some(texture_index) = vertex_attribute.vertex_texture_index {
                        self.obj_raw
                            .textures
                            .get(texture_index as usize)
                            .and_then(|texture_uv| Some(texture_uv.y))
                            .unwrap()
                    } else {
                        0.0f32
                    },
                    ..Default::default()
                });
            }
        }

        ObjAsset {
            faces: vertices,
            indices: indices,
        }
    }
}
