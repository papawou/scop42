use crate::ft_vk::{GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule};
use ash::vk;

use super::default_viewports_and_scissors;

pub fn create_layout(device: &ash::Device) -> PipelineLayout {
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

pub fn create_layout_with_constants<TPushConstants>(
    device: &ash::Device,
) -> PipelineLayout<TPushConstants> {
    let push_constant_ranges = [vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        size: std::mem::size_of::<TPushConstants>() as u32,
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

    PipelineLayout::<TPushConstants> {
        layout,
        _marker: std::marker::PhantomData,
    }
}
