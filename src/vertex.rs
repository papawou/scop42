use ash::vk;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: glam::Vec3,
    pub uv_x: f32,
    pub color: glam::Vec3,
    pub uv_y: f32,
    pub normal: glam::Vec3,
}
