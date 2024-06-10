use ash::vk;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub color: glam::Vec3,
}

impl Vertex {
    pub const fn new(position: glam::Vec3, color: glam::Vec3, normal: glam::Vec3) -> Self {
        Self {
            position,
            color,
            normal,
        }
    }
}

pub trait VertexHelpers {
    fn bindings() -> [vk::VertexInputBindingDescription; 1];
    fn attributes() -> [vk::VertexInputAttributeDescription; 3];
}

impl VertexHelpers for Vertex {
    fn bindings() -> [vk::VertexInputBindingDescription; 1] {
        let binding_desc = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);
        [binding_desc]
    }

    fn attributes() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(std::mem::size_of::<glam::Vec3>() as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(
                    (std::mem::size_of::<glam::Vec3>() + std::mem::size_of::<glam::Vec3>()) as u32,
                ),
        ]
    }
}
