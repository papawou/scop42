mod material_params;

use ash::vk::{self, DescriptorSetLayout, Framebuffer};
use glam::Vec3;

use crate::{
    ft_vk::{
        self,
        allocated_buffer::AllocatedBuffer,
        allocated_image::{self, AllocatedImage},
        descriptor_allocator::DescriptorAllocator,
        descriptor_set_layout::DescriptorSetLayoutCreateInfoBuilder,
        Engine, GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule,
    },
    helpers::{buffer::load_buffer, default_viewports_and_scissors},
    material_asset::MaterialAsset,
    obj_asset::{self, MaterialLib, ObjAsset},
};

pub struct Material<TPipeline = NoPipeline> {
    pub descriptor_set: vk::DescriptorSet,
    params: AllocatedBuffer,
    pub pipeline: TPipeline,
}

impl Material<NoPipeline> {
    pub fn new(
        engine: &mut Engine,
        asset: &MaterialAsset,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Self {
        let device = &engine.device;

        let params = {
            let params: material_params::Params = asset.into();
            let allocator = engine.allocator.as_mut().unwrap();
            let command_pool = engine.frames[0].command_pool;
            let command_buffer = engine.frames[0].command_buffer;
            let graphics_queue = engine.graphics_queue;

            load_buffer(
                device,
                allocator,
                command_pool,
                command_buffer,
                engine.graphics_queue,
                &params,
            )
        };

        let descriptor_set = {
            let descriptor_allocator = &mut engine.descriptor_allocator;
            descriptor_allocator.allocate_descriptor_set(&engine.device, descriptor_set_layout)
        };

        // fn update_descriptor_set
        unsafe {
            device.update_descriptor_sets(
                &[
                    // params
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(&[vk::DescriptorBufferInfo::default()
                            .buffer(params.buffer)
                            .range(vk::WHOLE_SIZE)]),
                ],
                &[],
            )
        };

        Self {
            params,
            descriptor_set,
            pipeline: NoPipeline,
        }
    }

    pub fn load_pipeline<'a, TPushConstants>(
        self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        layout: &'a PipelineLayout<TPushConstants>,
    ) -> Material<Pipeline> {
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
        unsafe { device.destroy_shader_module(vert_module, None) };

        Material {
            descriptor_set: self.descriptor_set,
            params: self.params,
            pipeline: Pipeline(pipelines[0]),
        }
    }

    pub fn destroy(mut self, allocator: &vk_mem::Allocator) {
        unsafe { allocator.destroy_buffer(self.params.buffer, &mut self.params.allocation) };
    }
}

impl Material<Pipeline> {
    pub fn unload_pipeline(self, device: &ash::Device) -> Material<NoPipeline> {
        unsafe {
            device.destroy_pipeline(self.pipeline.0, None);
        }

        Material {
            descriptor_set: self.descriptor_set,
            params: self.params,
            pipeline: NoPipeline,
        }
    }
}

// Material.pipeline states
pub struct NoPipeline; // no pipeline
pub struct Pipeline(pub vk::Pipeline); // pipeline is loaded in vk
impl Pipeline {
    pub fn as_vk(&self) -> vk::Pipeline {
        self.0
    }
}

pub fn descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
    let bindings = vec![
        // params
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
    ];

    let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

    unsafe { device.create_descriptor_set_layout(&info, None).unwrap() }
}
