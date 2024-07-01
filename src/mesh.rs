use ash::vk::{self};
use vk_mem::Alloc;

use crate::{
    engine::Engine,
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

        let (buffer, buffer_size, mut allocation) = {
            let buffer_size = self.vertices.len() * std::mem::size_of::<T>();
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

    pub fn create_staging_buffer(&self, allocator: &vk_mem::Allocator) -> AllocatedBuffer {
        let vertex_buffer = &self.vertex_buffer.as_ref().unwrap();
        let index_buffer = &self.index_buffer.as_ref().unwrap();

        let buffer_size = vertex_buffer.buffer_size + index_buffer.buffer_size;
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC);
        let allocation_info = vk_mem::AllocationCreateInfo {
            flags: vk_mem::AllocationCreateFlags::MAPPED,
            usage: vk_mem::MemoryUsage::CpuOnly,
            ..vk_mem::AllocationCreateInfo::default()
        };

        let (staging_buffer, mut allocation) = unsafe {
            allocator
                .create_buffer(&buffer_info, &allocation_info)
                .unwrap()
        };

        //  !warn
        //  The following code incorrectly maps and copies both vertex and index data to the same memory location,
        //  causing data corruption. The second copy operation overwrites the data copied by the first operation.
        //  This needs to be fixed by correctly managing the memory offsets for vertex and index data. (vk specs alignment)
        //  let alignment = device_properties.limits.non_coherent_atom_size as usize;
        //  let aligned_buffer_size = (vertex_buffer.buffer_size + alignment - 1) & !(alignment - 1);
        unsafe {
            let data_ptr = allocator.map_memory(&mut allocation).unwrap();
            std::ptr::copy_nonoverlapping(
                self.vertices.as_ptr() as *const u8,
                data_ptr,
                vertex_buffer.buffer_size as usize,
            );

            std::ptr::copy_nonoverlapping(
                self.indices.as_ptr() as *const u8,
                data_ptr.add(vertex_buffer.buffer_size),
                index_buffer.buffer_size as usize,
            );

            allocator.unmap_memory(&mut allocation);
        }

        AllocatedBuffer {
            allocation,
            buffer: staging_buffer,
            buffer_size,
            device_address: None,
        }
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
    Vertex {
        position: glam::Vec3::new(1.0, 1.0, 0.0),
        uv_x: 1.0,
        normal: glam::Vec3::new(0.0, 0.0, 1.0),
        uv_y: 1.0,
        color: glam::Vec3::new(1.0, 1.0, 0.0),
    },
];

const DEFAULT_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

pub fn load_default_mesh(
    engine: &Engine,
    device: &ash::Device,
    allocator: &vk_mem::Allocator,
    cmd: vk::CommandBuffer,
) -> Mesh<Vertex> {
    let mut mesh = Mesh {
        vertices: DEFAULT_VERTICES.to_vec(),
        indices: DEFAULT_INDICES.to_vec(),
        index_buffer: None,
        vertex_buffer: None,
    };

    mesh.create_vertex_buffer(device, allocator);
    mesh.create_index_buffer(allocator);
    let vertex_buffer = mesh.vertex_buffer.as_ref().unwrap();
    let index_buffer = mesh.vertex_buffer.as_ref().unwrap();

    let staging_buffer = mesh.create_staging_buffer(allocator);

    unsafe {
        device
            .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
            .unwrap();

        let cmd_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        device.begin_command_buffer(cmd, &cmd_begin_info).unwrap();
        let regions = [vk::BufferCopy::default()
            .size(index_buffer.buffer_size as u64)
            .src_offset(vertex_buffer.buffer_size as u64)];
        device.cmd_copy_buffer(cmd, staging_buffer.buffer, vertex_buffer.buffer, &regions);
        let regions = [vk::BufferCopy::default().size(vertex_buffer.buffer_size as u64)];
        device.cmd_copy_buffer(cmd, staging_buffer.buffer, index_buffer.buffer, &regions);
        device.end_command_buffer(cmd).unwrap();
        let submit_info = vk::SubmitInfo::default();
        device
            .queue_submit(engine.graphics_queue, &[submit_info], vk::Fence::null())
            .unwrap();

        device
            .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
            .unwrap();

        device.destroy_buffer(staging_buffer.buffer, None);
    }

    mesh
}
