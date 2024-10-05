use std::collections::VecDeque;

use ash::vk;

#[derive(Debug)]
pub struct DescriptorAllocator {
    full_pools: VecDeque<vk::DescriptorPool>,
    ready_pools: VecDeque<vk::DescriptorPool>,
    pool_sizes: Vec<vk::DescriptorPoolSize>,
    max_sets: u32,
}

impl DescriptorAllocator {
    pub fn new(max_sets: u32, pool_sizes: Vec<vk::DescriptorPoolSize>) -> Self {
        Self {
            max_sets,
            pool_sizes,
            full_pools: VecDeque::new(),
            ready_pools: VecDeque::new(),
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

    pub fn allocate_descriptor_set(
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
                let pool = {
                    let pool_info = vk::DescriptorPoolCreateInfo::default()
                        .flags(vk::DescriptorPoolCreateFlags::empty())
                        .max_sets(self.max_sets)
                        .pool_sizes(&self.pool_sizes);
                    unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() }
                };
                self.max_sets = (self.max_sets + (self.max_sets / 2));

                pool
            }
        };

        new_pool
    }
}
