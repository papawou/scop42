use ash::vk;

use crate::mesh::Mesh;

//MESH should be another file ?
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

pub trait IntoOwned {
    type Owned;
    fn into_owned(&self) -> Self::Owned;
}

impl<'a> IntoOwned for MeshConstants<'a> {
    type Owned = MeshConstantsOwned;

    fn into_owned(&self) -> Self::Owned {
        MeshConstantsOwned {
            render_matrix: self.render_matrix,
            vertex_buffer: *self.vertex_buffer,
        }
    }
}

pub fn create_default_layout(device: &ash::Device) -> vk::PipelineLayout {
    let layout = unsafe {
        device
            .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)
            .unwrap()
    };

    layout
}

pub fn create_mesh_layout<T>(device: &ash::Device) -> vk::PipelineLayout {
    let push_constant_ranges = [vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        size: std::mem::size_of::<T>() as u32,
        offset: 0,
    }];
    let layout = unsafe {
        device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&push_constant_ranges),
                None,
            )
            .unwrap()
    };

    layout
}
