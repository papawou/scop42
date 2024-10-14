use ash::vk;
use vk_mem::Alloc;

pub mod buffer;
mod pipeline;

use crate::ft_vk::allocated_buffer::AllocatedBuffer;

pub fn arr_to_bytes<T>(arr: &[T]) -> &[u8] {
    let size = std::mem::size_of::<T>() * arr.len();
    unsafe { std::slice::from_raw_parts(arr.as_ptr() as *const u8, size) }
}

pub fn struct_to_bytes<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())
    }
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

pub fn default_viewports_and_scissors(
    extent: vk::Extent2D,
) -> (Vec<vk::Viewport>, Vec<vk::Rect2D>) {
    let viewport = vk::Viewport::default()
        .width(extent.width as f32)
        .height(extent.height as f32)
        .max_depth(1.0);
    let scissor = vk::Rect2D::default().extent(extent);

    let viewports = vec![viewport];
    let scissors = vec![scissor];
    return (viewports, scissors);
}
