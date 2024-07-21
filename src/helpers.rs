use ash::vk;

pub fn copy_buffer(
    device: &ash::Device,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) {
    let allocation_info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&allocation_info) }.unwrap()[0];

    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();
    let copy_region = vk::BufferCopy::default()
        .src_offset(0)
        .dst_offset(0)
        .size(size);
    unsafe { device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]) };
    unsafe { device.end_command_buffer(command_buffer) }.unwrap();

    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

    unsafe { device.queue_submit(queue, &[submit_info], vk::Fence::null()) }.unwrap();
    unsafe { device.queue_wait_idle(queue) }.unwrap();

    unsafe { device.free_command_buffers(command_pool, &[command_buffer]) }
}
