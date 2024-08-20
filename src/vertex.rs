use ash::vk;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: glam::Vec3,
    pub _padding_position: f32, //added because glsl std430 consider vec3 as vec4 (float)
}
