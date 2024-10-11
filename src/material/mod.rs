mod utils;

use ash::vk::{self, DescriptorSetLayout, Framebuffer};
use glam::Vec3;

use crate::{
    ft_vk::{
        self,
        allocated_buffer::AllocatedBuffer,
        allocated_image::{self, AllocatedImage},
        descriptor_set_layout::DescriptorSetLayoutCreateInfoBuilder,
        Engine, GraphicsPipelineInfoBuilder, PipelineLayout, ShaderModule,
    },
    material_asset::MaterialAsset,
    obj_asset::{self, MaterialLib, ObjAsset},
};

pub struct Material<'a> {
    // pub pipeline: vk::Pipeline,

    // descriptors
    pub buffer_info: Vec<vk::DescriptorBufferInfo>,

    // layout bindings
    pub bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,

    // descriptor set
    pub descriptor_set: Vec<vk::DescriptorSet>,
}

impl<'a> Material<'a> {
    pub fn new(engine: &Engine) -> Self {
        let allocated_buffer = {};

        //let buffer_info = vk::DescriptorBufferInfo::default().buffer(params.buffer);

        let params_buffer = {};
        let descriptor_set_layout = {};

        Self {
            //  buffer_info: vec![buffer_info],
            bindings: vec![vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)],
        }
    }

    pub fn write_descriptor_sets(&'a self) -> Vec<vk::WriteDescriptorSet<'a>> {
        vec![vk::WriteDescriptorSet::default()
            //.dst_set(descriptor_set.clone())
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .buffer_info(&self.buffer_info)]
    }

    pub fn descriptor_set_layout(&'a self) -> vk::DescriptorSetLayoutCreateInfo<'a> {
        vk::DescriptorSetLayoutCreateInfo::default().bindings(&self.bindings)
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
