use std::{collections::VecDeque, ops::Mul};

use ash::vk;

pub struct DescriptorSetLayout(vk::DescriptorSetLayout);

impl DescriptorSetLayout {
    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.0
    }
}

struct DescriptorSetLayoutCreateInfoBuilder<'a> {
    stage_flags: vk::ShaderStageFlags, // binding.stage_flags |= stage_flags
    bindings: Vec<vk::DescriptorSetLayoutBinding<'a>>,
}

impl<'a> DescriptorSetLayoutCreateInfoBuilder<'a> {
    fn new() -> Self {
        Self {
            stage_flags: vk::ShaderStageFlags::empty(),
            bindings: vec![],
        }
    }

    fn build(&'a mut self) -> vk::DescriptorSetLayoutCreateInfo<'a> {
        self.bindings = self
            .bindings
            .iter()
            .map(|binding| binding.stage_flags(binding.stage_flags | self.stage_flags))
            .collect();

        vk::DescriptorSetLayoutCreateInfo::default().bindings(self.bindings.as_slice())
    }
}

// DescriptorAllocator

struct DescriptorAllocator {
    full_pools: VecDeque<vk::DescriptorPool>,
    ready_pools: VecDeque<vk::DescriptorPool>,
    ratios: Vec<vk::DescriptorPoolSize>,
    sets_per_pool: u32,
}

impl DescriptorAllocator {
    pub fn new(max_sets: u32, pool_ratios: Vec<vk::DescriptorPoolSize>) -> Self {
        Self {
            sets_per_pool: max_sets,
            ratios: pool_ratios,
            full_pools: VecDeque::new(),
            ready_pools: VecDeque::new(),
        }
    }

    // why this func ? ignored until further tutorial
    pub fn init(device: &ash::Device, max_sets: u32, pool_ratios: Vec<vk::DescriptorPoolSize>) {
        let test:vk::DescriptorSet

        //why resetting the ratios ??
        { // ratios.clear();

            // for (auto r : poolRatios) {
            //     ratios.push_back(r);
            // }
        }

        // get_pool seems same
        {
            // VkDescriptorPool newPool = create_pool(device, maxSets, poolRatios);

            // setsPerPool = maxSets * 1.5; //grow it next allocation

            // readyPools.push_back(newPool);
        }
    }

    pub fn reset_pools(&mut self, device: &ash::Device) {
        for &pool in &self.ready_pools {
            unsafe {
                device
                    .reset_descriptor_pool(pool, vk::DescriptorPoolResetFlags::empty())
                    .unwrap()
            };
        }

        for &pool in &self.full_pools {
            unsafe {
                device
                    .reset_descriptor_pool(pool, vk::DescriptorPoolResetFlags::empty())
                    .unwrap()
            };
            self.ready_pools.push_back(pool);
        }
        self.full_pools.clear();
    }

    pub fn destroy_pools(&mut self, device: &ash::Device) {
        for &pool in &self.ready_pools {
            unsafe { device.destroy_descriptor_pool(pool, None) };
        }
        self.ready_pools.clear();

        for &pool in &self.full_pools {
            unsafe { device.destroy_descriptor_pool(pool, None) };
        }
        self.full_pools.clear();
    }

    pub fn allocate(
        &mut self,
        device: &ash::Device,
        layout: vk::DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        let pool = self.get_pool(device);

        let layouts = [layout];
        let descriptor_set_info = vk::DescriptorSetAllocateInfo::default()
            .set_layouts(&layouts)
            .descriptor_pool(pool);

        let (pool, descriptor_sets) = match unsafe {
            device.allocate_descriptor_sets(&descriptor_set_info)
        } {
            Ok(destriptor_sets) => {
                self.ready_pools.push_back(pool);
                (pool, destriptor_sets)
            }
            Err(error) => {
                println!(
                        "Descriptor.allocate: can't allocate descriptor_sets error=`{}`, retrying by creating a new pool...",
                        error
                    );
                self.full_pools.push_back(pool);

                let pool = self.get_pool(device);
                let description_set_info = descriptor_set_info.descriptor_pool(pool);
                let descriptor_sets = unsafe {
                    device
                        .allocate_descriptor_sets(&description_set_info)
                        .unwrap()
                };

                (pool, descriptor_sets)
            }
        };

        self.ready_pools.push_back(pool);

        descriptor_sets.first().copied().unwrap()
    }

    fn get_pool(&mut self, device: &ash::Device) -> vk::DescriptorPool {
        let new_pool: vk::DescriptorPool = match self.ready_pools.pop_back() {
            Some(pool) => pool,
            None => {
                // fn create_pool(VkDevice device, uint32_t setCount, std::span<PoolSizeRatio> poolRatios)
                let pool = {
                    let pool_info = vk::DescriptorPoolCreateInfo::default()
                        .flags(vk::DescriptorPoolCreateFlags::empty())
                        .max_sets(self.sets_per_pool)
                        .pool_sizes(&self.ratios);
                    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
                };
                // setsPerPool * 1.5; -- for next create_pool
                self.sets_per_pool = (self.sets_per_pool + (self.sets_per_pool / 2));

                pool
            }
        };

        new_pool
    }
}

// DescriptorWriter

// typedef struct VkWriteDescriptorSet {
//     VkStructureType                  sType;
//     const void*                      pNext;
//     VkDescriptorSet                  dstSet;
//     uint32_t                         dstBinding;
//     uint32_t                         dstArrayElement;
//     uint32_t                         descriptorCount;
//     VkDescriptorType                 descriptorType;
//     const VkDescriptorImageInfo*     pImageInfo;
//     const VkDescriptorBufferInfo*    pBufferInfo;
//     const VkBufferView*              pTexelBufferView;
// } VkWriteDescriptorSet;
