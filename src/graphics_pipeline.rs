use ash::vk::{self, Framebuffer};

use crate::{engine::Engine, pipeline_layout::PipelineLayout};

pub struct GraphicsPipeline<'a, T> {
    pub layout: &'a PipelineLayout<T>,
    //pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
}

pub struct GraphicsPipelineInfoBuilder<'a> {
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'a>,
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    rasterization: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisample: vk::PipelineMultisampleStateCreateInfo<'a>,
    color_blend: vk::PipelineColorBlendStateCreateInfo<'a>,
    color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl<'a> GraphicsPipelineInfoBuilder<'a> {
    pub fn new() -> Self {
        GraphicsPipelineInfoBuilder {
            vertex_input_state: vk::PipelineVertexInputStateCreateInfo::default(),
            input_assembly: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false),
            rasterization: vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .line_width(1.0)
                //no backface cull
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                //no depth bias
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0),
            multisample: vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                //multisamping defaulted to no multisampling (1 sample per pixel)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            color_blend_attachments: vec![vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)],
            color_blend: vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(vk::LogicOp::COPY),
        }
    }

    pub fn set_obj_compatible(&mut self) -> &mut Self {
        self.input_assembly = self
            .input_assembly
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(true);

        self
    }

    pub fn build(&'a mut self) -> vk::GraphicsPipelineCreateInfo<'a> {
        self.color_blend = self.color_blend.attachments(&self.color_blend_attachments);

        vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&self.vertex_input_state)
            .input_assembly_state(&self.input_assembly)
            .rasterization_state(&self.rasterization)
            .multisample_state(&self.multisample)
            .color_blend_state(&self.color_blend)
    }
}

//DEFAULT
pub fn create_tri_pipeline<'a>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout,
) -> GraphicsPipeline<'a, ()> {
    let main_entry = std::ffi::CString::new("main").unwrap();
    let vert_module = create_shader_module(device, "./shaders/colored_tri.vert.spv");
    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(main_entry.as_c_str());
    let frag_module = create_shader_module(device, "./shaders/colored_tri.frag.spv");
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
    let vert_module = create_shader_module(device, "./shaders/mesh_dba.vert.spv");
    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(main_entry.as_c_str());
    let frag_module = create_shader_module(device, "./shaders/mesh.frag.spv");
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

fn create_shader_module(device: &ash::Device, filename: &str) -> vk::ShaderModule {
    let mut shader_file = std::fs::File::open(filename).unwrap();
    let shader_code = ash::util::read_spv(&mut shader_file).unwrap();

    let createinfo = ash::vk::ShaderModuleCreateInfo::default().code(&shader_code);
    unsafe { device.create_shader_module(&createinfo, None).unwrap() }
}
