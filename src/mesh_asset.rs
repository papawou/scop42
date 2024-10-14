use glam::Vec3;

use crate::{obj_asset::ObjAsset, vertex::Vertex};

#[derive(Debug)]
pub struct MeshAsset<T> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
}

impl MeshAsset<Vertex> {
    pub fn default() -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: glam::Vec3::new(0.0, 0.0, 0.0),
                    uv_x: 0f32,
                    color: glam::Vec3::new(0.0, 0.0, 0.0),
                    uv_y: 0f32,
                    normal: glam::Vec3::ZERO,
                    _padding_hack: 0.0f32,
                },
                Vertex {
                    position: glam::Vec3::new(1.0, 0.0, 0.0),
                    uv_x: 0f32,
                    color: glam::Vec3::new(1.0, 0.0, 0.0),
                    uv_y: 0f32,
                    normal: glam::Vec3::ZERO,
                    _padding_hack: 0.0f32,
                },
                Vertex {
                    position: glam::Vec3::new(0.0, 1.0, 0.0),
                    uv_x: 0f32,
                    color: glam::Vec3::new(0.0, 1.0, 0.0),
                    uv_y: 0f32,
                    normal: glam::Vec3::ZERO,
                    _padding_hack: 0.0f32,
                },
                Vertex {
                    position: glam::Vec3::new(1.0, 1.0, 0.0),
                    uv_x: 0f32,
                    color: glam::Vec3::new(1.0, 1.0, 0.0),
                    uv_y: 0f32,
                    normal: glam::Vec3::ZERO,
                    _padding_hack: 0.0f32,
                },
            ],
            indices: vec![0, 1, 2, 2, 1, 3],
        }
    }

    pub fn from_obj(obj: &ObjAsset) -> Self {
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        let mut indice: u32 = 0;
        for face in obj.faces() {
            for vertex in face {
                indices.push(indice);

                vertices.push(Vertex {
                    position: vertex.position.truncate(),
                    normal: vertex.normal.unwrap_or(Vec3::ZERO),
                    uv_x: vertex.texture.unwrap_or_default().x,
                    uv_y: vertex.texture.unwrap_or_default().y,
                    color: Vec3::ZERO,
                    ..Default::default()
                });

                indice += 1;
            }
            indices.push(u32::MAX); // triangle_strip but actually obj is triangle_list ready
        }

        MeshAsset { vertices, indices }
    }
}

// boilerplate for generic from_obj
// impl<T> MeshAsset<T>
// where
//     T: From<Vertex>,
// {
//     pub fn from_obj(obj: &ObjAsset) -> Self {
//         let mut vertices: Vec<T> = Vec::new();
//         let mut indices: Vec<u32> = Vec::new();

//         let mut indice: u32 = 0;
//         for face in obj.faces() {
//             for vertex in face {
//                 indices.push(indice);

//                 // Convert Vertex to T using the From trait
//                 vertices.push(T::from(Vertex {
//                     position: vertex.position.truncate(),
//                     normal: vertex.normal.unwrap_or(Vec3::ZERO),
//                     uv_x: vertex.texture.unwrap_or_default().x,
//                     uv_y: vertex.texture.unwrap_or_default().y,
//                     color: Vec3::ZERO,
//                     ..Default::default()
//                 }));

//                 indice += 1;
//             }
//             indices.push(u32::MAX); // triangle_strip but actually obj is triangle_list ready
//         }

//         MeshAsset { vertices, indices }
//     }
// }
