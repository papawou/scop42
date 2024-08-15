use ash::vk::{self};
use vk_mem::Alloc;

use crate::helpers::vec_to_bytes;
use crate::{
    engine::Engine,
    helpers::{copy_buffer, struct_to_bytes},
    vertex::{self, Vertex},
    AllocatedBuffer,
};

pub struct Mesh<T> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<AllocatedBuffer>,
    pub index_buffer: Option<AllocatedBuffer>,
}

impl<T> Mesh<T> {
    pub fn create_vertex_buffer(&mut self, device: &ash::Device, allocator: &vk_mem::Allocator) {
        if self.vertex_buffer.is_some() {
            panic!("vertex buffer already allocated");
        }

        let (buffer, buffer_size, allocation) = {
            let buffer_size = self.vertices.len() * std::mem::size_of::<T>();
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size as vk::DeviceSize)
                .usage(
                    vk::BufferUsageFlags::STORAGE_BUFFER
                        | vk::BufferUsageFlags::TRANSFER_DST
                        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                );
            let allocation_info = vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM
                    | vk_mem::AllocationCreateFlags::MAPPED,
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
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

        let allocated_buffer = AllocatedBuffer {
            buffer,
            device_address: Some(device_address),
            buffer_size,
            allocation,
        };

        self.vertex_buffer = Some(allocated_buffer);
    }

    pub fn create_index_buffer(&mut self, allocator: &vk_mem::Allocator) {
        if self.index_buffer.is_some() {
            panic!("index buffer already allocated");
        }

        let (buffer, buffer_size, allocation) = {
            let buffer_size = self.indices.len() * std::mem::size_of::<u32>();
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocation_info = vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM
                    | vk_mem::AllocationCreateFlags::MAPPED,
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
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

    pub fn destroy_buffers(&mut self, allocator: &vk_mem::Allocator) {
        match &mut self.vertex_buffer.take() {
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

const DEFAULT_VERTICES: [Vertex; 4] = [
    Vertex {
        position: glam::Vec3::new(0.0, 0.0, 0.0),
    },
    Vertex {
        position: glam::Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        position: glam::Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        position: glam::Vec3::new(1.0, 1.0, 0.0),
    },
];

const DEFAULT_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

pub fn load_default_mesh(
    device: &ash::Device,
    allocator: &mut vk_mem::Allocator,
    graphics_queue: vk::Queue,
    cmd: vk::CommandBuffer,
    command_pool: vk::CommandPool,
) -> Mesh<Vertex> {
    let mut mesh = Mesh {
        vertices: DEFAULT_VERTICES.to_vec(),
        indices: DEFAULT_INDICES.to_vec(),
        index_buffer: None,
        vertex_buffer: None,
    };

    {
        let data = vec_to_bytes(&mesh.vertices);

        let mut staging_buffer =
            crate::helpers::create_staging_buffer(data, data.len() as vk::DeviceSize, allocator);

        mesh.create_vertex_buffer(device, allocator);
        let vertex_buffer = mesh.vertex_buffer.as_ref().unwrap();

        copy_buffer(
            device,
            staging_buffer.buffer,
            vertex_buffer.buffer,
            vertex_buffer.buffer_size as vk::DeviceSize,
            command_pool,
            graphics_queue,
        );

        unsafe {
            allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation);
        }
    }

    {
        let data = vec_to_bytes(&mesh.indices);

        let mut staging_buffer =
            crate::helpers::create_staging_buffer(data, data.len() as vk::DeviceSize, allocator);

        mesh.create_index_buffer(allocator);
        let index_buffer = mesh.index_buffer.as_ref().unwrap();

        copy_buffer(
            device,
            staging_buffer.buffer,
            index_buffer.buffer,
            index_buffer.buffer_size as vk::DeviceSize,
            command_pool,
            graphics_queue,
        );

        unsafe {
            allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation);
        }
    }

    mesh
}
