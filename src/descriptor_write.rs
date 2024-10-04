use std::collections::VecDeque;

use ash::vk;

struct DescriptorWriter<'a> {
    writes: Vec<DescriptorWrite<'a>>,
}
impl<'a> DescriptorWriter<'a> {
    //     void DescriptorWriter::write_buffer(int binding, VkBuffer buffer, size_t size, size_t offset, VkDescriptorType type)
    // {
    // 	VkDescriptorBufferInfo& info = bufferInfos.emplace_back(VkDescriptorBufferInfo{
    // 		.buffer = buffer,
    // 		.offset = offset,
    // 		.range = size
    // 		});

    // 	VkWriteDescriptorSet write = {.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET};

    // 	write.dstBinding = binding;
    // 	write.dstSet = VK_NULL_HANDLE; //left empty for now until we need to write it
    // 	write.descriptorCount = 1;
    // 	write.descriptorType = type;
    // 	write.pBufferInfo = &info;

    // 	writes.push_back(write);
    // }

    fn write_buffer(
        &mut self,
        binding: u32,
        buffer: vk::Buffer,
        range: u64,
        offset: u64,
        ty: vk::DescriptorType,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(offset)
            .range(range);

        let write_descriptor_set = vk::WriteDescriptorSet::default()
            .dst_binding(binding)
            .dst_set(vk::DescriptorSet::null())
            .descriptor_count(1);

        self.writes
            .push(DescriptorWrite::Buffer(DescriptorWriterBuffer(
                vec![buffer_info],
                write_descriptor_set,
            )));
    }

    // void DescriptorWriter::update_set(VkDevice device, VkDescriptorSet set)
    // {
    //     for (VkWriteDescriptorSet& write : writes) {
    //         write.dstSet = set;
    //     }

    //     vkUpdateDescriptorSets(device, (uint32_t)writes.size(), writes.data(), 0, nullptr);
    // }
    fn update_set(&'a mut self, device: &ash::Device, set: vk::DescriptorSet) {
        let write_descriptor_sets: Vec<vk::WriteDescriptorSet<'a>> = self
            .writes
            .iter_mut()
            .map(|write| write.write_descriptor_set().clone())
            .collect();

        let copy_descriptor_sets = [];
        unsafe { device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };
    }
}

trait DescriptorWriterModifierTrait<'a> {
    fn write_descriptor_set(&'a mut self) -> &vk::WriteDescriptorSet<'a>;
}

enum DescriptorWrite<'a> {
    Buffer(DescriptorWriterBuffer<'a>),
    Image(DescriptorWriterImage<'a>),
}
impl<'a> DescriptorWriterModifierTrait<'a> for DescriptorWrite<'a> {
    fn write_descriptor_set(&'a mut self) -> &vk::WriteDescriptorSet<'a> {
        match self {
            DescriptorWrite::Buffer(buffer) => buffer.write_descriptor_set(),
            DescriptorWrite::Image(image) => image.write_descriptor_set(),
        }
    }
}

struct DescriptorWriterBuffer<'a>(Vec<vk::DescriptorBufferInfo>, vk::WriteDescriptorSet<'a>);
impl<'a> DescriptorWriterModifierTrait<'a> for DescriptorWriterBuffer<'a> {
    fn write_descriptor_set(&'a mut self) -> &vk::WriteDescriptorSet<'a> {
        let buffer_info = &self.0;
        self.1 = self.1.buffer_info(buffer_info);
        &self.1
    }
}

struct DescriptorWriterImage<'a>(Vec<vk::DescriptorImageInfo>, vk::WriteDescriptorSet<'a>);
impl<'a> DescriptorWriterModifierTrait<'a> for DescriptorWriterImage<'a> {
    fn write_descriptor_set(&'a mut self) -> &vk::WriteDescriptorSet<'a> {
        let image_info = &self.0;
        self.1 = self.1.image_info(image_info);
        &self.1
    }
}
