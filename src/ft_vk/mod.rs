pub mod allocated_buffer;
pub mod allocated_image;
mod frame_data;
mod graphics_pipeline;
mod pipeline_layout;
mod shader_module;
use descriptor_allocator::DescriptorAllocator;
pub use graphics_pipeline::GraphicsPipelineInfoBuilder;
pub use pipeline_layout::PipelineLayout;
pub use shader_module::ShaderModule;
mod queue_famillies;
pub use queue_famillies::QueueFamilies;

pub mod descriptor_allocator;
pub mod descriptor_set_layout;
mod render_pass;
mod surface_support;
mod swapchain;

use ash::vk::{self};
use std::{error::Error, time::Instant};
use vk_mem::Alloc;
use winit::raw_window_handle::HasRawWindowHandle;

use frame_data::FrameData;
use surface_support::SurfaceSupport;
use swapchain::Swapchain;

use crate::conf;

pub trait Renderer {
    unsafe fn render(&self, engine: &Engine, framebuffer: vk::Framebuffer, cmd: vk::CommandBuffer);
}

pub struct Engine {
    pub entry: ash::Entry,
    pub instance: ash::Instance,

    pub start_instant: Instant,

    // Debug
    pub debug_utils_loader: ash::ext::debug_utils::Instance,
    pub debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    // Phyisical device
    pub physical_device: vk::PhysicalDevice,

    // Surface
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,

    // Device
    pub device: ash::Device,
    pub queue_families: QueueFamilies,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    // vkMem
    pub allocator: Option<vk_mem::Allocator>,

    // descriptor allocator
    pub descriptor_allocator: DescriptorAllocator,

    // Swapchain
    pub frames: Vec<FrameData>,

    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: swapchain::Swapchain,

    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub frame_count: usize,
}

impl Engine {
    pub fn new(entry: ash::Entry, window: &winit::window::Window) -> Self {
        // Instance
        let instance = create_instance(&entry);
        let (debug_utils_loader, debug_utils_messenger) = setup_debug_utils(&entry, &instance);

        // Physical device
        let physical_device = get_physical_device(&instance);

        // Window
        let window_physical_size = window.inner_size();

        #[cfg(target_os = "windows")]
        let surface = {
            let window_info = {
                use winit::raw_window_handle::HasWindowHandle;

                let hwnd = match window.window_handle().unwrap().as_raw() {
                    winit::raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd.get(),
                    _ => panic!("Unsupported platform!"),
                };

                let hinstance = {
                    let hmodule = unsafe {
                        windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap()
                    };

                    if hmodule.is_invalid() {
                        panic!("Unsupported windows hinstance")
                    }

                    hmodule.0
                };

                vk::Win32SurfaceCreateInfoKHR::default()
                    .hwnd(hwnd as vk::HWND)
                    .hinstance(hinstance as vk::HINSTANCE)
            };

            let win_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
            unsafe { win_surface_loader.create_win32_surface(&window_info, None) }.unwrap()
        };

        #[cfg(target_os = "linux")]
        let surface = {
            use winit::raw_window_handle::HasDisplayHandle;
            use winit::raw_window_handle::HasWindowHandle;

            let window_handle = window.window_handle().unwrap().as_raw();
            let display_handle = window.display_handle().unwrap().as_raw();
            match (window_handle, display_handle) {
                // Xlib
                (
                    winit::raw_window_handle::RawWindowHandle::Xlib(window_handle),
                    winit::raw_window_handle::RawDisplayHandle::Xlib(display_handle),
                ) => {
                    let window = window_handle.window;
                    let display = display_handle
                        .display
                        .map(|d| d.as_ptr())
                        .unwrap_or(std::ptr::null_mut());

                    let window_info = vk::XlibSurfaceCreateInfoKHR::default()
                        .window(window)
                        .dpy(display);
                    let surface_loader = ash::khr::xlib_surface::Instance::new(&entry, &instance);
                    unsafe { surface_loader.create_xlib_surface(&window_info, None) }.unwrap()
                }
                // Wayland
                (
                    winit::raw_window_handle::RawWindowHandle::Wayland(window_handle),
                    winit::raw_window_handle::RawDisplayHandle::Wayland(display_handle),
                ) => {
                    let surface = window_handle.surface.as_ptr();
                    let display = display_handle.display.as_ptr();

                    let window_info = vk::WaylandSurfaceCreateInfoKHR::default()
                        .surface(surface)
                        .display(display);
                    let surface_loader =
                        ash::khr::wayland_surface::Instance::new(&entry, &instance);
                    unsafe { surface_loader.create_wayland_surface(&window_info, None) }.unwrap()
                }
                _ => panic!("Unsupported platform!"),
            }
        };

        // Surface
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let surface_support = SurfaceSupport::new(physical_device, surface, &surface_loader);
        if !surface_support.is_physical_device_compatible(&instance, physical_device) {
            panic!("physical_device invalid")
        }

        // Device
        let (device, queue_families) =
            create_device(&instance, physical_device, &surface_loader, surface).unwrap();
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present, 0) };

        // Allocator
        let allocator = create_allocator(&instance, &device, physical_device);

        let descriptor_allocator = DescriptorAllocator::new(
            1,
            vec![vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)],
        );

        // Swapchain
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = Swapchain::new(
            &swapchain_loader,
            &device,
            &allocator,
            (window_physical_size.width, window_physical_size.height),
            &surface_support,
            surface,
            &queue_families,
            None,
        );

        let frames = create_present_frames(
            &device,
            queue_families.graphics,
            swapchain.min_image_count as usize,
        );
        let render_pass = render_pass::create_default(&device, swapchain.surface_format.format);
        let framebuffers = swapchain.get_framebuffers(&device, render_pass);

        Self {
            entry,
            instance,
            device,

            allocator: Some(allocator),
            descriptor_allocator,

            swapchain_loader,
            swapchain,

            debug_utils_loader,
            debug_utils_messenger,

            surface_loader,
            surface,

            physical_device,
            queue_families,
            graphics_queue,
            present_queue,

            render_pass,
            framebuffers,

            frames,
            frame_count: 0,

            start_instant: Instant::now(),
        }
    }

    pub unsafe fn draw_frame(&mut self, renderer: &impl Renderer) -> Result<(), vk::Result> {
        self.frame_count += 1;
        let FrameData {
            command_buffer: cmd,
            fence,
            present_semaphore,
            render_semaphore,
            ..
        } = self.frames[self.frame_count % self.swapchain.min_image_count as usize];

        self.device
            .wait_for_fences(&[fence], true, u64::MAX)
            .unwrap();

        let swapchain_image_idx = self
            .swapchain_loader
            .acquire_next_image(
                self.swapchain.chain,
                u64::MAX,
                present_semaphore,
                vk::Fence::null(),
            )?
            .0;

        let framebuffer = self.framebuffers[swapchain_image_idx as usize];

        self.device
            .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
            .unwrap();

        //RECORD
        let cmd_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device
            .begin_command_buffer(cmd, &cmd_begin_info)
            .unwrap();

        //RENDERER
        renderer.render(&self, framebuffer, cmd);

        self.device.end_command_buffer(cmd).unwrap();

        //SUBMIT
        self.device.reset_fences(&[fence]).unwrap();

        let command_buffers = [cmd];
        let present_semaphores = [present_semaphore];
        let render_semaphores = [render_semaphore];
        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&present_semaphores)
            .signal_semaphores(&render_semaphores);
        self.device
            .queue_submit(self.graphics_queue, &[submit_info], fence)
            .unwrap();

        //PRESENTATION
        let swapchains = [self.swapchain.chain];
        let image_indices = [swapchain_image_idx];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&render_semaphores)
            .image_indices(&image_indices);
        self.swapchain_loader
            .queue_present(self.graphics_queue, &present_info)?;

        Ok(())
    }

    pub unsafe fn destroy(mut self) {
        for frame in self.frames {
            frame.destroy(&self.device);
        }

        for &framebuffer in &self.framebuffers {
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.framebuffers.clear();

        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain.destroy(
            &self.device,
            self.allocator.as_ref().unwrap(),
            &self.swapchain_loader,
        );

        self.descriptor_allocator.destroy_pools(&self.device);
        self.allocator = None; //vmaDestroyAllocator(_allocator);

        self.device.destroy_device(None);

        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }

    pub unsafe fn handle_resize(&mut self, physical_size: (u32, u32)) {
        let new_swapchain = {
            //swapchain
            let surface_support =
                SurfaceSupport::new(self.physical_device, self.surface, &self.surface_loader);
            swapchain::Swapchain::new(
                &self.swapchain_loader,
                &self.device,
                &self.allocator.as_ref().unwrap(),
                (physical_size.0, physical_size.1),
                &surface_support,
                self.surface,
                &self.queue_families,
                Some(self.swapchain.chain),
            )
        };

        let old_swapchain = std::mem::replace(&mut self.swapchain, new_swapchain);

        //destroy old_swapchain
        {
            for &framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(framebuffer, None)
            }
            self.device.destroy_render_pass(self.render_pass, None);
            old_swapchain.destroy(
                &self.device,
                self.allocator.as_ref().unwrap(),
                &self.swapchain_loader,
            );
        }

        self.render_pass =
            render_pass::create_default(&self.device, self.swapchain.surface_format.format);
        self.framebuffers = self
            .swapchain
            .get_framebuffers(&self.device, self.render_pass);
    }
}

fn create_instance(entry: &ash::Entry) -> ash::Instance {
    let (_layer_names, layer_name_pointers) = conf::get_layer_names();

    let application_name = std::ffi::CString::new(conf::APPLICATION_NAME).unwrap();
    let engine_name = std::ffi::CString::new(conf::ENGINE_NAME).unwrap();
    let application_info = vk::ApplicationInfo::default()
        .application_name(&application_name)
        .application_version(conf::APPLICATION_VERSION)
        .engine_name(&engine_name)
        .engine_version(conf::ENGINE_VERSION)
        .api_version(conf::API_VERSION);

    let instance_create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&conf::EXTENSION_NAMES);

    unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
}

//DEVICES
fn get_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
    let phys_devs = unsafe { instance.enumerate_physical_devices().unwrap() };
    let phys_dev = phys_devs
        .into_iter()
        .find_map(|p| {
            let properties = unsafe { instance.get_physical_device_properties(p) };

            let name = unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }
                .to_str()
                .unwrap();
            match name {
                conf::PHYSICAL_DEVICE_NAME => Some(p),
                _ => None,
            }
        })
        .unwrap(); //onpanic: see conf::PHYSICAL_DEVICE_NAME

    phys_dev
}

//queue families
fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<(ash::Device, QueueFamilies)> {
    let physical_device_queue_families =
        QueueFamilies::new(instance, physical_device, surface_loader, surface);

    let queue_priorities = [1.0];

    let mut queue_infos = vec![vk::DeviceQueueCreateInfo::default()
        .queue_family_index(physical_device_queue_families.graphics)
        .queue_priorities(&queue_priorities)];

    if physical_device_queue_families.graphics != physical_device_queue_families.present {
        queue_infos.push(
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(physical_device_queue_families.present)
                .queue_priorities(&queue_priorities),
        );
    }

    let mut features2 = vk::PhysicalDeviceFeatures2::default();
    features2.features.shader_int64 = vk::TRUE;

    let mut buffer_device_address_features = vk::PhysicalDeviceBufferDeviceAddressFeatures {
        buffer_device_address: vk::TRUE,
        ..Default::default()
    };
    let device_extensions = conf::DEVICE_EXTENSION_NAMES
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(device_extensions.as_slice())
        .push_next(&mut buffer_device_address_features)
        .push_next(&mut features2);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    Ok((device, physical_device_queue_families))
}

//DEBUG
fn setup_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::ext::debug_utils::Instance::new(entry, instance);

    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            // vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));
    let debug_utils_messenger =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();

    (debug_utils_loader, debug_utils_messenger)
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

fn create_present_frames(
    device: &ash::Device,
    graphics_family: u32,
    count: usize,
) -> Vec<FrameData> {
    let mut frames = Vec::with_capacity(count);

    for _ in 0..count {
        frames.push(FrameData::new(device, graphics_family));
    }

    frames
}

fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> vk_mem::Allocator {
    let mut create_info = vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);
    create_info.flags = vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
    create_info.vulkan_api_version = vk::API_VERSION_1_3;
    unsafe { vk_mem::Allocator::new(create_info).unwrap() }
}
