use ash::vk;

#[derive(Debug)]
pub struct FrameData {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,

    pub fence: vk::Fence,
    pub render_semaphore: vk::Semaphore,
    pub present_semaphore: vk::Semaphore,
}

impl FrameData {
    pub fn new(device: &ash::Device, graphics_family: u32) -> Self {
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family);
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap();

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_info) }.unwrap()[0];

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let render_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };
        let present_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };

        Self {
            command_pool,
            command_buffer,
            fence,
            render_semaphore,
            present_semaphore,
        }
    }
}
