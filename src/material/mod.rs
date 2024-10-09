mod utils;

use ash::vk::{self, Framebuffer};
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

pub struct Material {
    // struct MaterialPipeline
    // pub layout: &'a PipelineLayout<TPushConstants>, // hold DescriptorSetLayout
    // pub pipeline: vk::Pipeline,

    // descriptors
    // pub descriptor_sets: Vec<vk::DescriptorSet>,
    // pub asset: &'a MaterialAsset,
    pub buffer_info: Vec<vk::DescriptorBufferInfo>,
    pub image_info: Vec<vk::DescriptorImageInfo>,
}

impl Material {
    pub fn new(
        params: &AllocatedBuffer,
        allocated_image: &AllocatedImage,
        sampler: &vk::Sampler,
    ) -> Self {
        let buffer_info = vk::DescriptorBufferInfo::default().buffer(params.buffer);
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::default())
            .image_view(allocated_image.image_view)
            .sampler(sampler.clone());

        Self {
            buffer_info: vec![buffer_info],
            image_info: vec![image_info],
        }
    }

    pub fn descriptor_set_layouts<'a>(
        descriptor_set_layout_builder: &'a mut DescriptorSetLayoutCreateInfoBuilder<'a>,
    ) -> &'a mut DescriptorSetLayoutCreateInfoBuilder<'a> {
        descriptor_set_layout_builder
            .add_binding(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE),
            )
            .add_binding(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            )
    }

    pub fn write_descriptor_sets<'a>(
        &'a self,
        // descriptor_set_layout_builder: &vk::DescriptorSet,
    ) -> Vec<vk::WriteDescriptorSet<'a>> {
        vec![
            vk::WriteDescriptorSet::default()
                //.dst_set(descriptor_set.clone())
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .buffer_info(&self.buffer_info),
            vk::WriteDescriptorSet::default()
                //.dst_set(descriptor_set.clone())
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(&self.image_info),
        ]
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
