use ash::vk;

#[derive(Debug)]
pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub device_address: Option<vk::DeviceAddress>,
    pub buffer_size: usize,
    pub allocation: vk_mem::Allocation,
}
