use ash::vk;

//VERTEX
#[derive(Debug, Default, Copy, Clone)]
pub struct Vertex {
    pub pos: glam::Vec2,
    pub color: glam::Vec3,
}

impl Vertex {
    pub fn new(pos: glam::Vec2, color: glam::Vec3) -> Self {
        Self { pos, color }
    }

    pub fn vkVertexInputBindingDescription() -> ash::vk::VertexInputBindingDescription {
        let binding_description = ash::vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        binding_description
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
