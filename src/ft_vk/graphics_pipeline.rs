use ash::vk;

pub struct GraphicsPipelineInfoBuilder<'a> {
    vertex_input_state: vk::PipelineVertexInputStateCreateInfo<'a>,
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    rasterization: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisample: vk::PipelineMultisampleStateCreateInfo<'a>,
    color_blend: vk::PipelineColorBlendStateCreateInfo<'a>,
    color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
    depth_stencil: vk::PipelineDepthStencilStateCreateInfo<'a>,
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
                // backface cull
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
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
            depth_stencil: vk::PipelineDepthStencilStateCreateInfo::default(),
        }
    }

    pub fn set_obj_compatible(&mut self) -> &mut Self {
        self.input_assembly = self
            .input_assembly
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(true);

        self
    }

    pub fn set_depth_stencil(&mut self) -> &mut Self {
        self.depth_stencil = self
            .depth_stencil
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0f32)
            .max_depth_bounds(1.0f32)
            .stencil_test_enable(false);

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
            .depth_stencil_state(&self.depth_stencil)
    }
}
