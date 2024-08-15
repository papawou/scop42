use ash::vk;

#[repr(C, align(16))]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: glam::Vec3,
}
