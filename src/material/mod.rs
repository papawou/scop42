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

pub struct Material {
    pub descriptor_set: vk::DescriptorSet,
    params: AllocatedBuffer,
}

impl Material {
    pub fn new(engine: &mut Engine, asset: &MaterialAsset) -> Self {
        let device = &engine.device;

        let params = {
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
                asset,
            )
        };

        let descriptor_set = {
            // create descriptor set layout
            let descriptor_set_layout = {
                let bindings = vec![vk::DescriptorSetLayoutBinding::default() // params
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)];
                let info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

                unsafe {
                    engine
                        .device
                        .create_descriptor_set_layout(&info, None)
                        .unwrap()
                }
            };

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
                        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                        .buffer_info(&[vk::DescriptorBufferInfo::default().buffer(params.buffer)]),
                ],
                &[],
            )
        };

        Self {
            params,
            descriptor_set,
        }
    }
}

// // should be not here, its descriptor_set_layout <-> pipeline_layout
// // **descriptor_set_layout contains bindings related to material if configured by it
//
// pub fn modify_pipeline_layout(
//     device: &ash::Device,
//     pipeline_layout_create_info: &'a mut vk::PipelineLayoutCreateInfo,
// ) -> vk::PipelineLayoutCreateInfo<'a> {
//     //extract it
//     // {
//     //     let push_constant_ranges = [vk::PushConstantRange {
//     //         stage_flags: vk::ShaderStageFlags::VERTEX,
//     //         size: std::mem::size_of::<TPushConstants>() as u32,
//     //         offset: 0,
//     //     }];
//     // }

//     //pipeline_layout_create_info.set_layouts(set_layouts)

//     pipeline_layout_create_info.set_layouts([set_layouts]))
// }

pub fn create_pipeline<'a, TPushConstants>(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    layout: &'a PipelineLayout<TPushConstants>,
    //material: &Material for shaders module ?
) -> vk::Pipeline {
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

    pipelines[0]
}
