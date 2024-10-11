use ash::vk;
use vk_mem::Alloc;

use crate::ft_vk::allocated_buffer::AllocatedBuffer;

use super::{arr_to_bytes, struct_to_bytes};

pub fn load_buffer<'a, T>(
    device: &ash::Device,
    allocator: &mut vk_mem::Allocator,
    command_pool: vk::CommandPool,
    cmd: vk::CommandBuffer,
    graphics_queue: vk::Queue,
    data: impl Loadable,
) -> AllocatedBuffer {
    let data = data.as_bytes();

    let mut staging_buffer = load_staging_buffer(allocator, data);

    let (buffer, buffer_size, allocation) = create_buffer(allocator, data.len() as vk::DeviceSize);

    copy_buffer(
        device,
        command_pool,
        graphics_queue,
        staging_buffer.buffer,
        buffer,
        buffer_size,
    );

    unsafe {
        allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation);
    }

    AllocatedBuffer {
        buffer,
        device_address: None,
        buffer_size,
        allocation,
    }
}

pub fn load_staging_buffer(allocator: &vk_mem::Allocator, data: &[u8]) -> AllocatedBuffer {
    let buffer_size = data.len() as vk::DeviceAddress;
    let buffer_create_info = vk::BufferCreateInfo::default()
        .size(buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
            | vk_mem::AllocationCreateFlags::MAPPED,
        usage: vk_mem::MemoryUsage::Auto,
        ..vk_mem::AllocationCreateInfo::default()
    };

    let (buffer, mut allocation) = unsafe {
        allocator
            .create_buffer(&buffer_create_info, &allocation_create_info)
            .unwrap()
    };

    unsafe {
        let allocation_info = allocator.get_allocation_info(&allocation);
        let data_ptr = allocation_info.mapped_data;

        if data_ptr.is_null() {
            panic!("Mapped data pointer is null");
        }

        std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut u8, data.len());
    }

    AllocatedBuffer {
        allocation,
        buffer,
        buffer_size,
        device_address: None,
    }
}

pub fn create_buffer(
    allocator: &vk_mem::Allocator,
    buffer_size: vk::DeviceAddress,
) -> (vk::Buffer, vk::DeviceSize, vk_mem::Allocation) {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(buffer_size as vk::DeviceSize)
        .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);

    let (buffer, allocation) = unsafe {
        let allocation_info = vk_mem::AllocationCreateInfo {
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM,
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        allocator
            .create_buffer(&buffer_info, &allocation_info)
            .unwrap()
    };

    (buffer, buffer_size as vk::DeviceSize, allocation)
}

pub fn copy_buffer(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let allocation_info = vk::CommandBufferAllocateInfo::default()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    // RECORD
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

    // SEND
    let command_buffers = [command_buffer];
    let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
    unsafe { device.queue_submit(queue, &[submit_info], vk::Fence::null()) }.unwrap();

    unsafe { device.queue_wait_idle(queue) }.unwrap(); // !warn wait idle

    unsafe { device.free_command_buffers(command_pool, &[command_buffer]) }
}

// LOADABLE
pub trait Loadable {
    fn as_bytes(&self) -> &[u8];
}

impl<T> Loadable for &T {
    fn as_bytes(&self) -> &[u8] {
        struct_to_bytes(*self)
    }
}

impl<T> Loadable for &[T] {
    fn as_bytes(&self) -> &[u8] {
        arr_to_bytes(*self)
    }
}
