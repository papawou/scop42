mod frame_data;
mod queue_famillies;
mod surface_support;
mod swapchain;

use ash::vk::{self};
use frame_data::FrameData;
use queue_famillies::QueueFamilies;
use surface_support::SurfaceSupport;

use crate::conf;
use winit::{platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle};

pub trait Renderer {
    unsafe fn render(&self, engine: &Engine, framebuffer: vk::Framebuffer, cmd: vk::CommandBuffer);
}

pub struct Engine {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,

    //vmem
    pub allocator: Option<vk_mem::Allocator>,

    //swapchain
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: swapchain::Swapchain,

    //debug
    pub debug_utils_loader: ash::ext::debug_utils::Instance,
    pub debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    //device
    pub physical_device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    //surface
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,

    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub frames: [FrameData; conf::MAX_FRAMES_IN_FLIGHT],

    pub frame_count: usize,
}

impl Engine {
    pub fn new(entry: ash::Entry, window: &winit::window::Window) -> Self {
        //  window
        let hwnd = match window.window_handle().unwrap().as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => panic!("Unsupported platform!"),
        };
        let hinstance = unsafe { winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null()) };
        let window_info = vk::Win32SurfaceCreateInfoKHR::default()
            .hwnd(hwnd as vk::HWND)
            .hinstance(hinstance as vk::HINSTANCE);
        let window_physical_size = window.inner_size();

        let instance = create_instance(&entry);
        let (debug_utils_loader, debug_utils_messenger) = setup_debug_utils(&entry, &instance);

        //  surface
        let win_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
        let surface =
            unsafe { win_surface_loader.create_win32_surface(&window_info, None) }.unwrap();

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // device
        let physical_device = get_physical_device(&instance);

        let surface_support = SurfaceSupport::new(physical_device, surface, &surface_loader);
        if !surface_support.is_physical_device_compatible(&instance, physical_device) {
            panic!("physical_device invalid")
        }

        let (device, queue_families) =
            create_device(&instance, physical_device, &surface_loader, surface).unwrap();
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present, 0) };

        //swapchain
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = swapchain::Swapchain::new(
            &swapchain_loader,
            &device,
            (window_physical_size.width, window_physical_size.height),
            &surface_support,
            surface,
            &queue_families,
            None,
        );

        let frames = create_present_frames(&device, queue_families.graphics);

        //vmem allocator
        let allocator = create_allocator(&instance, &device, physical_device);

        let render_pass = create_default_render_pass(&device, swapchain.surface_format.format);

        let framebuffers = create_framebuffers(&device, &swapchain, render_pass);

        Self {
            entry,
            instance,
            device,

            allocator: Some(allocator),

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
        }
    }

    pub unsafe fn draw_frame(&mut self, renderer: &impl Renderer) -> bool {
        self.frame_count += 1;
        let FrameData {
            command_buffer: cmd,
            fence,
            present_semaphore,
            render_semaphore,
            ..
        } = self.frames[self.frame_count % conf::MAX_FRAMES_IN_FLIGHT];

        self.device
            .wait_for_fences(&[fence], true, u64::MAX)
            .unwrap();

        let swapchain_image_idx = match self.swapchain_loader.acquire_next_image(
            self.swapchain.chain,
            u64::MAX,
            present_semaphore,
            vk::Fence::null(),
        ) {
            Result::Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                return true
            }
            Err(e) => panic!("{}", e),
        };

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
        match self
            .swapchain_loader
            .queue_present(self.graphics_queue, &present_info)
        {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                return true
            }
            Err(e) => panic!("{}", e),
            _ => (),
        };

        false
    }

    pub unsafe fn destroy(&mut self) {
        for frame in &self.frames {
            self.device.destroy_semaphore(frame.present_semaphore, None);
            self.device.destroy_semaphore(frame.render_semaphore, None);
            self.device.destroy_fence(frame.fence, None);
            self.device.destroy_command_pool(frame.command_pool, None)
        }

        for &framebuffer in &self.framebuffers {
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.framebuffers.clear();

        self.device.destroy_render_pass(self.render_pass, None);

        self.allocator = None; //vmaDestroyAllocator(_allocator);

        self.swapchain.clean(&self.device, &self.swapchain_loader);

        self.device.destroy_device(None);

        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }

    pub unsafe fn handle_resize(&mut self, physical_size: (u32, u32)) -> bool {
        //swapchain
        let surface_support =
            SurfaceSupport::new(self.physical_device, self.surface, &self.surface_loader);
        let new_swapchain = swapchain::Swapchain::new(
            &self.swapchain_loader,
            &self.device,
            (physical_size.0, physical_size.1),
            &surface_support,
            self.surface,
            &self.queue_families,
            Some(self.swapchain.chain),
        );

        for &framebuffer in &self.framebuffers {
            self.device.destroy_framebuffer(framebuffer, None)
        }

        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain.clean(&self.device, &self.swapchain_loader);

        //init
        self.swapchain = new_swapchain;

        self.render_pass =
            create_default_render_pass(&self.device, self.swapchain.surface_format.format);
        self.framebuffers = create_framebuffers(&self.device, &self.swapchain, self.render_pass);

        return false;
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
        .unwrap();

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

    let features = vk::PhysicalDeviceFeatures::default();

    let device_extensions = conf::DEVICE_EXTENSION_NAMES
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(device_extensions.as_slice())
        .enabled_features(&features);

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

fn create_framebuffers(
    device: &ash::Device,
    swapchain: &swapchain::Swapchain,
    render_pass: vk::RenderPass,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = Vec::with_capacity(swapchain.image_views.len());

    //When rendering, the swapchain will give us the index of the image to render into, so we will use the framebuffer of the same index.
    for &image_view in &swapchain.image_views {
        let attachments = [image_view];
        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1);
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };
        framebuffers.push(framebuffer);
    }

    framebuffers
}

fn create_present_frames(
    device: &ash::Device,
    graphics_family: u32,
) -> [FrameData; conf::MAX_FRAMES_IN_FLIGHT] {
    let mut frames = Vec::with_capacity(conf::MAX_FRAMES_IN_FLIGHT);

    for _ in 0..conf::MAX_FRAMES_IN_FLIGHT {
        frames.push(FrameData::new(device, graphics_family));
    }
    frames.try_into().unwrap()
}

fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> vk_mem::Allocator {
    let create_info = vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);
    unsafe { vk_mem::Allocator::new(create_info).unwrap() }
}

fn create_default_render_pass(device: &ash::Device, format: vk::Format) -> vk::RenderPass {
    let color_attachments = [vk::AttachmentReference::default()
        .attachment(0) //index link to renderpass.attachments
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
    let subpasses = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)];

    let attachments = [vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];
    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses);
    unsafe { device.create_render_pass(&render_pass_info, None).unwrap() }
}
