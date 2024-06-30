use ash::vk::{self};
use vk_mem::Alloc;

use crate::{vertex::Vertex, AllocatedBuffer};

pub struct Mesh<T> {
    pub vertices: Vec<T>,
    pub vertex_buffer: Option<AllocatedBuffer>,
    pub index_buffer: Option<AllocatedBuffer>,
}

impl<T> Mesh<T> {
    pub fn load_vertex_buffer(&mut self, device: &ash::Device, allocator: &vk_mem::Allocator) {
        if self.vertex_buffer.is_some() {
            panic!("vertex buffer already allocated");
        }

        let (buffer, buffer_size, mut allocation) = {
            let buffer_size = self.vertices.len() * std::mem::size_of::<Vertex>();
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size as vk::DeviceSize)
                .usage(
                    vk::BufferUsageFlags::STORAGE_BUFFER
                        | vk::BufferUsageFlags::TRANSFER_DST
                        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                );
            let allocation_info = vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::MAPPED,
                usage: vk_mem::MemoryUsage::GpuOnly,
                ..vk_mem::AllocationCreateInfo::default()
            };

            let (buffer, allocation) = unsafe {
                allocator
                    .create_buffer(&buffer_info, &allocation_info)
                    .unwrap()
            };

            (buffer, buffer_size, allocation)
        };

        let device_address = {
            let device_address_info = vk::BufferDeviceAddressInfo::default().buffer(buffer);
            unsafe { device.get_buffer_device_address(&device_address_info) }
        };

        // copy data to buffer
        let data_ptr = unsafe { allocator.map_memory(&mut allocation).unwrap() };
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.vertices.as_ptr() as *const u8,
                data_ptr,
                allocator.get_allocation_info(&allocation).size as usize,
            );
        }
        unsafe { allocator.unmap_memory(&mut allocation) };

        let allocated_buffer = AllocatedBuffer {
            buffer,
            device_address: Some(device_address),
            buffer_size,
            allocation,
        };

        self.index_buffer = Some(allocated_buffer);
    }

    pub fn load_index_buffer(&mut self, allocator: &vk_mem::Allocator) {
        if self.index_buffer.is_some() {
            panic!("vertex buffer already allocated");
        }

        let (buffer, buffer_size, allocation) = {
            let buffer_size = self.vertices.len() * std::mem::size_of::<u32>();
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocation_info = vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::MAPPED,
                usage: vk_mem::MemoryUsage::GpuOnly,
                ..vk_mem::AllocationCreateInfo::default()
            };

            let (buffer, allocation) = unsafe {
                allocator
                    .create_buffer(&buffer_info, &allocation_info)
                    .unwrap()
            };

            (buffer, buffer_size, allocation)
        };

        let allocated_buffer = AllocatedBuffer {
            buffer,
            device_address: None,
            buffer_size,
            allocation,
        };

        self.index_buffer = Some(allocated_buffer);
    }

    pub fn unload(&mut self, allocator: &vk_mem::Allocator) {
        match &mut self.index_buffer.take() {
            Some(allocated_buffer) => unsafe {
                allocator.destroy_buffer(allocated_buffer.buffer, &mut allocated_buffer.allocation);
            },
            _ => {}
        }

        match &mut self.index_buffer.take() {
            Some(allocated_buffer) => unsafe {
                allocator.destroy_buffer(allocated_buffer.buffer, &mut allocated_buffer.allocation);
            },
            _ => {}
        }
    }
}

const DEFAULT_MESH: [Vertex; 3] = [
    Vertex {
        position: glam::Vec3::new(0.0, 0.0, 0.0),
        uv_x: 0.0,
        normal: glam::Vec3::new(0.0, 0.0, 1.0),
        uv_y: 0.0,
        color: glam::Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        position: glam::Vec3::new(1.0, 0.0, 0.0),
        uv_x: 1.0,
        normal: glam::Vec3::new(0.0, 0.0, 1.0),
        uv_y: 0.0,
        color: glam::Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: glam::Vec3::new(0.0, 1.0, 0.0),
        uv_x: 0.0,
        normal: glam::Vec3::new(0.0, 0.0, 1.0),
        uv_y: 1.0,
        color: glam::Vec3::new(0.0, 0.0, 1.0),
    },
];

pub fn load_default_mesh(device: &ash::Device, allocator: &vk_mem::Allocator) -> Mesh<Vertex> {
    let mut mesh = Mesh {
        vertices: DEFAULT_MESH.to_vec(),
        index_buffer: None,
        vertex_buffer: None,
    };

    mesh.load_vertex_buffer(device, allocator);
    mesh.load_index_buffer(allocator);
    mesh
}
