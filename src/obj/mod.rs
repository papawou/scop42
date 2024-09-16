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

impl ObjAsset {
    fn to_mesh(obj_file: &ObjRaw) {}
}
