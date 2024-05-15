use ash::vk;

pub struct GraphicsPipeline {
    pub vertex_input_state: vk::PipelineVertexInputStateCreateInfo,
    pub input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfo,
    pub color_blend_state: vk::PipelineColorBlendStateCreateInfo,
    pub viewport_state: vk::PipelineViewportStateCreateInfo,
}

impl GraphicsPipeline {
    pub fn builder<'a>() -> GraphicsPipelineBuilder<'a> {
        GraphicsPipelineBuilder::new()
    }

    pub fn create_pipeline_builder(&self) -> vk::GraphicsPipelineCreateInfoBuilder {
        vk::GraphicsPipelineCreateInfo::builder()
            .vertex_input_state(&self.vertex_input_state)
            .input_assembly_state(&self.input_assembly_state)
            .rasterization_state(&self.rasterization_state)
            .multisample_state(&self.multisample_state)
            .color_blend_state(&self.color_blend_state)
            .viewport_state(&self.viewport_state)
    }
}

pub struct GraphicsPipelineBuilder<'a> {
    pub vertex_input_state: vk::PipelineVertexInputStateCreateInfoBuilder<'a>,
    pub input_assembly_state: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    pub rasterization_state: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    pub multisample_state: vk::PipelineMultisampleStateCreateInfoBuilder<'a>,
    pub color_blend_state: vk::PipelineColorBlendStateCreateInfoBuilder<'a>,
    pub viewport_state: vk::PipelineViewportStateCreateInfoBuilder<'a>,
}

impl<'a> GraphicsPipelineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            vertex_input_state: vk::PipelineVertexInputStateCreateInfo::builder(),
            input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo::builder()
                .primitive_restart_enable(false),
            rasterization_state: vk::PipelineRasterizationStateCreateInfo::builder()
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
            multisample_state: vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                //multisamping defaulted to no multisampling (1 sample per pixel)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            color_blend_state: vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op(vk::LogicOp::COPY),
            viewport_state: vk::PipelineViewportStateCreateInfo::builder(),
        }
    }

    pub fn vertex_input_state(
        mut self,
        vertex_input_state: vk::PipelineVertexInputStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.vertex_input_state = vertex_input_state;
        self
    }

    pub fn input_assembly_state(
        mut self,
        input_assembly_state: vk::PipelineInputAssemblyStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.input_assembly_state = input_assembly_state;
        self
    }

    pub fn rasterization_state(
        mut self,
        rasterization_state: vk::PipelineRasterizationStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.rasterization_state = rasterization_state;
        self
    }

    pub fn multisample_state(
        mut self,
        multisample_state: vk::PipelineMultisampleStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.multisample_state = multisample_state;
        self
    }

    pub fn color_blend_state(
        mut self,
        color_blend_state: vk::PipelineColorBlendStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.color_blend_state = color_blend_state;
        self
    }

    pub fn viewport_state(
        mut self,
        viewport_state: vk::PipelineViewportStateCreateInfoBuilder<'a>,
    ) -> Self {
        self.viewport_state = viewport_state;
        self
    }

    pub fn build(self) -> GraphicsPipeline {
        GraphicsPipeline {
            vertex_input_state: self.vertex_input_state.build(),
            input_assembly_state: self.input_assembly_state.build(),
            rasterization_state: self.rasterization_state.build(),
            multisample_state: self.multisample_state.build(),
            color_blend_state: self.color_blend_state.build(),
            viewport_state: self.viewport_state.build(),
        }
    }
}