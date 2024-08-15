use ash::vk;
use vk_mem::Alloc;

use crate::AllocatedBuffer;

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

pub fn create_staging_buffer(
    data: &[u8],
    buffer_size: vk::DeviceSize,
    allocator: &vk_mem::Allocator,
) -> AllocatedBuffer {
    let buffer_create_info = vk::BufferCreateInfo::default()
        .size(buffer_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC);

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
            | vk_mem::AllocationCreateFlags::MAPPED,
        usage: vk_mem::MemoryUsage::Auto,
        ..vk_mem::AllocationCreateInfo::default()
    };

    let (staging_buffer, mut allocation) = unsafe {
        allocator
            .create_buffer(&buffer_create_info, &allocation_create_info)
            .unwrap()
    };

    unsafe {
        let allocation_info = allocator.get_allocation_info(&allocation);
        let data_ptr = allocation_info.mapped_data;

        // Explicitly check for null
        if data_ptr.is_null() {
            panic!("Mapped data pointer is null");
        } else {
            println!(
                "Mapped data pointer is valid: {:?} {:?}",
                data_ptr, buffer_size
            );
        }

        std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut u8, buffer_size as usize);
    }

    AllocatedBuffer {
        allocation,
        buffer: staging_buffer,
        buffer_size: buffer_size as usize,
        device_address: None,
    }
}

pub fn vec_to_bytes<T>(v: &Vec<T>) -> &[u8] {
    let size = std::mem::size_of::<T>() * v.len();
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, size) }
}

pub fn struct_to_bytes<T>(s: &T) -> &[u8] {
    let size = std::mem::size_of::<T>();
    unsafe { std::slice::from_raw_parts((s as *const T) as *const u8, size) }
}

pub fn print_bytes_in_hex(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        if i % 16 == 0 {
            print!("\n{:04x}: ", i); // Print the offset in the array
        }
        print!("{:02x} ", byte);
    }
    println!(); // Final newline
}
