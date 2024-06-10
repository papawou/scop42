use ash::vk;

use crate::vertex::VertexHelpers;

pub struct GraphicsPipeline<'a> {
    pub layout: &'a vk::PipelineLayout,
    //pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
}

impl<'a> GraphicsPipeline<'a> {
    pub fn begin_render() {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
            },
        }];

        let renderpass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .framebuffer(framebuffer)
            .clear_values(&clear_values);
        self.device
            .cmd_begin_render_pass(cmd, &renderpass_info, vk::SubpassContents::INLINE);
    }

    pub fn end_render() {
        self.device
            .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
        self.device.cmd_end_render_pass(cmd);

        self.device.end_command_buffer(cmd).unwrap();
    }

    //after this call object should be dropped
    pub unsafe fn destroy(self, device: &ash::Device) -> Self {
        device.destroy_pipeline(self.pipeline, None);
        self
    }
}

pub struct GraphicsPipelineInfoBuilder<'a> {
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    rasterization: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisample: vk::PipelineMultisampleStateCreateInfo<'a>,
    color_blend: vk::PipelineColorBlendStateCreateInfo<'a>,
    color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl<'a> GraphicsPipelineInfoBuilder<'a> {
    pub fn new() -> Self {
        GraphicsPipelineInfoBuilder {
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

    pub fn build(&self) -> vk::GraphicsPipelineCreateInfo {
        self.color_blend.attachments(&self.color_blend_attachments);

        vk::GraphicsPipelineCreateInfo::default()
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
    layout: &'a vk::PipelineLayout,
) -> GraphicsPipeline<'a> {
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

    let default_pipeline_info = GraphicsPipelineInfoBuilder::new();
    let pipeline_info = default_pipeline_info
        .build()
        .stages(&stages)
        .viewport_state(&viewport_state)
        .layout(layout.clone())
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

pub fn create_mesh_pipeline<'a, T: VertexHelpers>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a vk::PipelineLayout,
) -> GraphicsPipeline<'a> {
    let main_entry = std::ffi::CString::new("main").unwrap();
    let vert_module = create_shader_module(device, "./shaders/mesh.vert.spv");
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

    let bindings = T::bindings();
    let attributes = T::attributes();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&bindings)
        .vertex_attribute_descriptions(&attributes);

    let default_pipeline_info = GraphicsPipelineInfoBuilder::new();
    let pipeline_info = default_pipeline_info
        .build()
        .stages(&stages)
        .viewport_state(&viewport_state)
        .vertex_input_state(&vertex_input_state)
        .layout(layout.clone())
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
