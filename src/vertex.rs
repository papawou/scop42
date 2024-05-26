use ash::vk;

//VERTEX
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub position: glam::Vec2,
    pub normal: glam::Vec3,
    pub color: glam::Vec3,
}

impl Vertex {
    pub const fn new(position: glam::Vec2, color: glam::Vec3, normal: glam::Vec3) -> Self {
        Self {
            position,
            color,
            normal,
        }
    }

    pub fn bindings() -> [vk::VertexInputBindingDescription; 1] {
        let binding_desc = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);
        [binding_desc]
    }

    pub fn attributes() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(std::mem::size_of::<glam::Vec2>() as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(
                    (std::mem::size_of::<glam::Vec2>() + std::mem::size_of::<glam::Vec3>()) as u32,
                ),
        ]
    }
}

// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone)]
// pub struct Vec2<T>
// where
//     T: Copy,
// {
//     pub x: T,
//     pub y: T,
// }

// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone)]
// pub struct Vec3<T> {
//     pub x: T,
//     pub y: T,
//     pub z: T,
// }

// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone)]
// pub struct Vec4<T> {
//     pub x: T,
//     pub y: T,
//     pub z: T,
//     pub w: T,
// }
