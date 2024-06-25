use ash::vk;

//MESH should be another file ?
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshPushConstants {
    pub data: glam::Vec4,
    pub render_matrix: glam::Mat4,
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
