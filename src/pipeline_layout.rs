use ash::vk;

pub fn create_default_layout(device: &ash::Device) -> vk::PipelineLayout {
    let tri_layout = unsafe {
        device
            .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)
            .unwrap()
    };

    tri_layout
}

//MESH should be another file ?
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MeshPushConstants {
    data: glam::Vec4,
    render_matrix: glam::Mat4,
}

pub fn create_mesh_layout(device: &ash::Device) -> vk::PipelineLayout {
    let push_constant_ranges = [vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        size: std::mem::size_of::<MeshPushConstants>() as u32,
        offset: 0,
    }];
    let mesh_layout = unsafe {
        device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .push_constant_ranges(&push_constant_ranges),
                None,
            )
            .unwrap()
    };

    mesh_layout
}
