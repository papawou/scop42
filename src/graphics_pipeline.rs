use ash::vk;

trait GraphicsPipelineInfoExt {
    fn set_defaults<'a>(
        &mut self,
        viewports: &'a [vk::Viewport],
        scissors: &'a [vk::Rect2D],
    ) -> Self;
}

impl GraphicsPipelineInfoExt for vk::GraphicsPipelineCreateInfo<'_> {
    fn set_defaults<'a>(
        &mut self,
        viewports: &'a [vk::Viewport],
        scissors: &'a [vk::Rect2D],
    ) -> Self {
        self.rasterization_state()
    }
}

pub struct GraphicsPipeline<'a> {
    pub layout: &'a vk::PipelineLayout,
    //pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
}

impl<'a> GraphicsPipelineInfo<'a> {
    fn new() -> Self {
        Self {
            input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo::default()
                .primitive_restart_enable(false),
            rasterization_state: vk::PipelineRasterizationStateCreateInfo::default()
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
            multisample_state: vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                //multisamping defaulted to no multisampling (1 sample per pixel)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            color_blend_state: vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(vk::LogicOp::COPY),

            vertex_input_state: vk::PipelineVertexInputStateCreateInfo::default(),
            viewport_state: vk::PipelineViewportStateCreateInfo::default(),
            assembly_state: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
        }
    }

    fn set_defaults(&mut self, viewports: &'a [vk::Viewport], scissors: &'a [vk::Rect2D]) {
        self.viewport_state = self
            .viewport_state
            .viewports(&viewports)
            .scissors(&scissors);

        self.input_assembly_state = self
            .input_assembly_state
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        self.rasterization_state = self.rasterization_state.polygon_mode(vk::PolygonMode::FILL);
    }
}

fn create_mesh_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain: &crate::swapchain_scop::SwapchainScop,
    layout: vk::PipelineLayout,
) -> vk::Pipeline {
    let main_entry = std::ffi::CString::new("main").unwrap();

    let mesh_vert_module = crate::create_shader_module(device, "./shaders/mesh.vert.spv").unwrap();
    let mesh_vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(mesh_vert_module)
        .name(main_entry.as_c_str());

    let colored_tri_frag_module =
        create_shader_module(device, "./shaders/colored_tri.frag.spv").unwrap();
    let colored_tri_frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(colored_tri_frag_module)
        .name(main_entry.as_c_str());

    let viewport = vk::Viewport::default()
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .max_depth(1.0);
    let viewports = [viewport];
    let scissor = vk::Rect2D::default().extent(swapchain.extent);
    let scissors = [scissor];

    let color_blend_attachments_state = [vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments_state);

    let stages = [mesh_vert_stage, colored_tri_frag_stage];
    let bindings = crate::vertex::Vertex::bindings();
    let attributes = crate::vertex::Vertex::attributes();

    let pipeline_build = GraphicsPipeline::builder()
        .set_defaults(&viewports, &scissors)
        .vertex_input_state(
            vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(&attributes),
        )
        .build();

    let mesh_pipeline_info = pipeline_build
        .create_pipeline_builder()
        .stages(&stages)
        .layout(layout) //should be kept as a reference in graphicspipelinebuilder and use as clone here, vk::Pipeline tied to vk::PipelineLayout
        .render_pass(render_pass)
        .color_blend_state(&color_blend_state);

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[mesh_pipeline_info], None)
            .unwrap()
    };

    unsafe { device.destroy_shader_module(colored_tri_frag_module, None) };
    unsafe { device.destroy_shader_module(mesh_vert_module, None) }

    pipelines[0]
}
