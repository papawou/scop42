#![feature(get_many_mut)]

use std::collections::{hash_set, HashMap, VecDeque};
use std::u32;

use ash::vk::{self};
use glam::Vec3;
use vk_mem::Alloc;

use crate::ft_vk::allocated_buffer::{self, AllocatedBuffer};
use crate::helpers::buffer::{copy_buffer, create_buffer, load_staging_buffer};
use crate::helpers::{arr_to_bytes, print_bytes_in_hex};
use crate::mesh_asset::MeshAsset;
use crate::obj_asset::obj_raw::face::Face;
use crate::obj_asset::{self, ObjAsset, ObjRaw};
use crate::{
    ft_vk::Engine,
    helpers::struct_to_bytes,
    vertex::{self, Vertex},
};

#[derive(Debug)]
pub struct Mesh<'a, T> {
    pub asset: &'a MeshAsset<T>,
    pub vertex_buffer: Option<AllocatedBuffer>,
    pub index_buffer: Option<AllocatedBuffer>,
}

impl<'a, T> Mesh<'a, T> {
    pub fn create_vertex_buffer(&mut self, device: &ash::Device, allocator: &vk_mem::Allocator) {
        if self.vertex_buffer.is_some() {
            panic!("vertex buffer already allocated");
        }

        let (buffer, buffer_size, allocation) = create_buffer(
            allocator,
            (self.asset.vertices.len() * std::mem::size_of::<T>()) as vk::DeviceSize,
        );

        // is driven by create_buffer allocation
        let device_address = {
            let device_address_info = vk::BufferDeviceAddressInfo::default().buffer(buffer);
            unsafe { device.get_buffer_device_address(&device_address_info) }
        };

        let allocated_buffer = AllocatedBuffer {
            allocation,
            buffer,
            buffer_size,
            device_address: Some(device_address),
        };

        self.vertex_buffer = Some(allocated_buffer);
    }

    pub fn create_index_buffer(&mut self, allocator: &vk_mem::Allocator) {
        if self.index_buffer.is_some() {
            panic!("index buffer already allocated");
        }

        let (buffer, buffer_size, allocation) = {
            let buffer_size =
                (self.asset.indices.len() * std::mem::size_of::<u32>()) as vk::DeviceSize;
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST);
            let allocation_info = vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM,
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

        self.index_buffer = Some(AllocatedBuffer {
            buffer,
            device_address: None,
            buffer_size,
            allocation,
        });
    }

    pub fn unload(&mut self, allocator: &vk_mem::Allocator) {
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

    pub fn load(
        &mut self,
        device: &ash::Device,
        allocator: &mut vk_mem::Allocator,
        graphics_queue: vk::Queue,
        cmd: vk::CommandBuffer,
        command_pool: vk::CommandPool,
    ) {
        // Vertex
        {
            let data = arr_to_bytes(&self.asset.vertices);
            let mut staging_buffer = load_staging_buffer(allocator, data);

            self.create_vertex_buffer(device, allocator);
            let vertex_buffer = self.vertex_buffer.as_ref().unwrap();

            copy_buffer(
                device,
                command_pool,
                graphics_queue,
                staging_buffer.buffer,
                vertex_buffer.buffer,
                vertex_buffer.buffer_size,
            );

            unsafe {
                allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation);
            }
        }

        // Index
        {
            let data = arr_to_bytes(&self.asset.indices);
            let mut staging_buffer = load_staging_buffer(allocator, data);

            self.create_index_buffer(allocator);
            let index_buffer = self.index_buffer.as_ref().unwrap();

            copy_buffer(
                device,
                command_pool,
                graphics_queue,
                staging_buffer.buffer,
                index_buffer.buffer,
                index_buffer.buffer_size,
            );

            unsafe {
                allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation);
            }
        }
    }
}
