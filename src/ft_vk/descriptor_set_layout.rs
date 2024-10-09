use std::{collections::VecDeque, ops::Mul};

use ash::vk;

pub struct DescriptorSetLayoutCreateInfoBuilder<'a> {
    stage_flags: vk::ShaderStageFlags, // binding.stage_flags |= stage_flags
    bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,
}

impl<'a> DescriptorSetLayoutCreateInfoBuilder<'a> {
    pub fn new() -> Self {
        Self {
            stage_flags: vk::ShaderStageFlags::empty(),
            bindings: vec![],
        }
    }

    pub fn add_binding(&'a mut self, binding: vk::DescriptorSetLayoutBinding<'a>) -> &'a mut Self {
        self.bindings.push(binding);
        self
    }

    pub fn build(&'a mut self) -> vk::DescriptorSetLayoutCreateInfo<'a> {
        self.bindings = self
            .bindings
            .iter()
            .map(|binding| binding.stage_flags(binding.stage_flags | self.stage_flags))
            .collect();

        vk::DescriptorSetLayoutCreateInfo::default().bindings(self.bindings.as_slice())
    }
}
