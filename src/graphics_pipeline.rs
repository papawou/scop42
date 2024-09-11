use ash::vk::{self, Framebuffer};

use crate::ft_vk::{self, Engine, GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule};

pub struct GraphicsPipeline<'a, T> {
    pub layout: &'a PipelineLayout<T>,
    //pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
}

//DEFAULT
pub fn create_tri_pipeline<'a>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout,
) -> GraphicsPipeline<'a, ()> {
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

    let (viewports, scissors) = default_viewports_and_scissors(extent);
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

    GraphicsPipeline {
        pipeline: pipelines[0],
        layout,
    }
}

pub fn create_mesh_pipeline<'a, T>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout<T>,
) -> GraphicsPipeline<'a, T> {
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

    let (viewports, scissors) = default_viewports_and_scissors(extent);
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(&viewports)
        .scissors(&scissors);

    let mut default_pipeline_info = GraphicsPipelineInfoBuilder::new();
    let pipeline_info = default_pipeline_info
        .set_obj_compatible()
        .set_depth_stencil()
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

    GraphicsPipeline {
        pipeline: pipelines[0],
        layout,
    }
}

fn default_viewports_and_scissors(extent: vk::Extent2D) -> (Vec<vk::Viewport>, Vec<vk::Rect2D>) {
    let viewport = vk::Viewport::default()
        .width(extent.width as f32)
        .height(extent.height as f32)
        .max_depth(1.0);
    let scissor = vk::Rect2D::default().extent(extent);

    let viewports = vec![viewport];
    let scissors = vec![scissor];
    return (viewports, scissors);
}
