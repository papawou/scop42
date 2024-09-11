use ash::vk;

use crate::{ft_vk::PipelineLayout, mesh::Mesh};

pub fn create_default_layout(device: &ash::Device) -> PipelineLayout {
    let layout = unsafe {
        device
            .create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)
            .unwrap()
    };

    PipelineLayout {
        layout,
        _marker: std::marker::PhantomData,
    }
}

pub fn create_mesh_layout<T>(device: &ash::Device) -> PipelineLayout<T> {
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

    PipelineLayout::<T> {
        layout,
        _marker: std::marker::PhantomData,
    }
}
