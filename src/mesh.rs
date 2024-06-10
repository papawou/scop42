use ash::vk::{self};
use vk_mem::Alloc;

use crate::{vertex::Vertex, AllocatedBuffer};

pub struct Mesh<T> {
    pub vertices: Vec<T>,
    pub vertex_buffer: Option<AllocatedBuffer>,
}

impl<T> Mesh<T> {
    pub fn new(vertices: Vec<T>, vertex_buffer: Option<AllocatedBuffer>) -> Self {
        Self {
            vertices,
            vertex_buffer,
        }
    }

    pub fn load(&mut self, allocator: &vk_mem::Allocator) {
        if self.vertex_buffer.is_some() {
            return;
        }

        //create buffer
        let buffer_size = self.vertices.len() * std::mem::size_of::<Vertex>();
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::CpuToGpu,
            ..vk_mem::AllocationCreateInfo::default()
        };
        let (buffer, mut allocation) = unsafe {
            allocator
                .create_buffer(&buffer_info, &allocation_info)
                .unwrap()
        };

        //copy data to buffer
        let data_ptr = unsafe { allocator.map_memory(&mut allocation).unwrap() };
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.vertices.as_ptr() as *const u8,
                data_ptr,
                buffer_size,
            );
        }
        unsafe { allocator.unmap_memory(&mut allocation) };

        //store buffer
        self.vertex_buffer = Some(AllocatedBuffer { buffer, allocation });
    }

    pub fn unload(&mut self, allocator: &vk_mem::Allocator) {
        match &mut self.vertex_buffer.take() {
            Some(allocated_buffer) => unsafe {
                allocator.destroy_buffer(allocated_buffer.buffer, &mut allocated_buffer.allocation);
            },
            _ => {}
        }
    }
}

//DEFAULT
const VERTICES: [Vertex; 3] = [
    Vertex::new(
        glam::vec3(1.0, 1.0, 0.0),
        glam::vec3(1.0, 0.0, 0.0),
        glam::Vec3::ZERO,
    ),
    Vertex::new(
        glam::vec3(-1.0, 1.0, 0.0),
        glam::vec3(0.0, 1.0, 0.0),
        glam::Vec3::ZERO,
    ),
    Vertex::new(
        glam::vec3(0.0, -1.0, 0.0),
        glam::vec3(0.0, 0.0, 1.0),
        glam::Vec3::ZERO,
    ),
];

pub fn load_default_mesh(allocator: &vk_mem::Allocator) -> Mesh<Vertex> {
    let mut mesh = Mesh::new(VERTICES.to_vec(), None);
    mesh.load(&allocator);

    mesh
}
