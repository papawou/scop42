use ash::vk;

use crate::ft_vk::{GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule};

use super::{utils, Material};

pub fn create_material<'a, TPushConstants>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout<TPushConstants>,
) -> Material<'a, TPushConstants> {
    let main_entry = std::ffi::CString::new("main").unwrap();
    let vert_module = ShaderModule::create_from_file(device, "./shaders/mesh_dba.vert.spv");
    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(main_entry.as_c_str());
    let frag_module = ShaderModule::create_from_file(device, "./shaders/mesh.frag.spv");
    let frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(main_entry.as_c_str());
    let stages = [vert_stage, frag_stage];

    let (viewports, scissors) = utils::default_viewports_and_scissors(extent);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(&viewports)
        .scissors(&scissors);

    let mut default_pipeline_info = GraphicsPipelineInfoBuilder::new();
    let pipeline_info = default_pipeline_info
        .set_obj_compatible()
        .set_depth_stencil()
        //.enable_blending_additive()
        .build()
        .stages(&stages)
        .viewport_state(&viewport_state)
        .layout(layout.as_vk())
        .render_pass(render_pass);

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .unwrap()
    };

    unsafe { device.destroy_shader_module(frag_module, None) };
    unsafe { device.destroy_shader_module(vert_module, None) }

    Material {
        pipeline: pipelines[0],
        layout,
    }
}

pub fn create_layout<TPushConstants>(device: &ash::Device) -> PipelineLayout<TPushConstants> {
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
