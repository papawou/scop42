use ash::vk;

use crate::ft_vk::{GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule};

use super::{utils, Material};

//DEFAULT
pub fn create_tri_material<'a>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout,
) -> Material<'a, ()> {
    let main_entry = std::ffi::CString::new("main").unwrap();
    let vert_module = ShaderModule::create_from_file(device, "./shaders/colored_tri.vert.spv");
    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(main_entry.as_c_str());
    let frag_module = ShaderModule::create_from_file(device, "./shaders/colored_tri.frag.spv");
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
        .build()
        .stages(&stages)
        .viewport_state(&viewport_state)
        .layout(layout.as_vk())
        .render_pass(render_pass);

    //GRAPHICS_PIPELINE
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
