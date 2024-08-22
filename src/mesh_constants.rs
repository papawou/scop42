use ash::vk;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshConstants<'a> {
    pub render_matrix: glam::Mat4,
    pub vertex_buffer: &'a vk::DeviceAddress,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshConstantsOwned {
    pub render_matrix: glam::Mat4,
    pub vertex_buffer: vk::DeviceAddress,
}

impl<'a> crate::traits::IntoOwned for MeshConstants<'a> {
    type Owned = MeshConstantsOwned;

    fn into_owned(&self) -> Self::Owned {
        MeshConstantsOwned {
            render_matrix: self.render_matrix,
            vertex_buffer: *self.vertex_buffer,
        }
    }
}
