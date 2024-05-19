mod conf;
mod graphics_pipeline;
mod swapchain_scop;
mod utils;
mod vertex;

use anyhow::Ok;
use ash::vk::{self};
use conf::MAX_FRAMES_IN_FLIGHT;
use graphics_pipeline::{GraphicsPipeline, GraphicsPipelineBuilder};
use swapchain_scop::SwapchainScop;
use vertex::Vertex;
use winit::{platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle};

const ONE_SEC: u64 = u64::MAX;

const VERTICES: [Vertex; 4] = [
    Vertex::new(glam::vec2(-0.5, -0.5), glam::vec3(1.0, 0.0, 0.0)),
    Vertex::new(glam::vec2(0.5, -0.5), glam::vec3(0.0, 1.0, 0.0)),
    Vertex::new(glam::vec2(0.5, 0.5), glam::vec3(0.0, 0.0, 1.0)),
    Vertex::new(glam::vec2(-0.5, 0.5), glam::vec3(1.0, 1.0, 1.0)),
];

const INDEX_VERTICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

fn main() -> anyhow::Result<()> {
    //std::env::set_var("RUST_BACKTRACE", "1");
    let entry = unsafe { ash::Entry::load()? };

    //window
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("Hello window!")
        .with_inner_size(winit::dpi::LogicalSize::new(
            conf::WINDOW_WIDTH,
            conf::WINDOW_HEIGHT,
        ))
        .build(&event_loop)?;

    let mut app = App::create(entry, &window)?;

    let mut current_frame = 0;
    let mut framebuffer_resized = false;

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            winit::event::Event::AboutToWait => window.request_redraw(),
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::RedrawRequested,
                ..
            } => {
                if framebuffer_resized && app.handle_resize(&window) {
                    return;
                }
                framebuffer_resized = unsafe { app.draw_frame(current_frame) };
                current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(_),
                ..
            } => framebuffer_resized = true,
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            winit::event::Event::LoopExiting => {
                unsafe { app.device.device_wait_idle() }.unwrap();
                unsafe { app.destroy() };
            }
            _ => {}
        })
        .unwrap();

    Ok(())
}

struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,

    //swapchain
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: swapchain_scop::SwapchainScop,

    //debug
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    //device
    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    //surface
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    frames: [FrameData; MAX_FRAMES_IN_FLIGHT],
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
}

impl App {
    fn create(entry: ash::Entry, window: &winit::window::Window) -> anyhow::Result<Self> {
        //  window
        let hwnd = match window.window_handle()?.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => panic!("Unsupported platform!"),
        };
        let hinstance = unsafe { winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null()) };
        let window_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hwnd(hwnd as vk::HWND)
            .hinstance(hinstance as vk::HINSTANCE)
            .build();
        let window_physical_size = window.inner_size();

        let mut debug_info: vk::DebugUtilsMessengerCreateInfoEXT = create_debug_info();

        let instance = get_instance(&entry, &mut debug_info)?;

        let (debug_utils_loader, debug_utils_messenger) =
            create_debug(&entry, &instance, &debug_info);

        //  surface
        let win_surface_loader = ash::extensions::khr::Win32Surface::new(&entry, &instance);
        let surface = unsafe { win_surface_loader.create_win32_surface(&window_info, None) }?;

        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        // device
        let physical_device = get_physical_device(&instance)?;

        let surface_support = SurfaceSupport::get(physical_device, surface, &surface_loader)?;
        if !check_physical_device(&instance, physical_device, &surface_support)? {
            panic!("physical_device invalid")
        }

        let (device, queue_families) =
            create_device(&instance, physical_device, &surface_loader, surface)?;
        let graphics_queue = unsafe { device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queue_families.present, 0) };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);

        let swapchain = swapchain_scop::create_swapchain(
            &swapchain_loader,
            &device,
            (window_physical_size.width, window_physical_size.height),
            &surface_support,
            surface,
            &queue_families,
            None,
        )?;

        let render_pass = create_default_render_pass(&device, &swapchain);

        let framebuffers = create_framebuffers(&device, &swapchain, render_pass);

        let frames = create_framesdata(&device, queue_families.graphics);

        let pipeline_layout = create_pipeline_layout(&device);

        let graphics_pipeline =
            create_graphics_pipeline(&device, render_pass, pipeline_layout, &swapchain);

        Ok(Self {
            entry,
            instance,
            device,

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
            pipeline_layout,
            graphics_pipeline,
        })
    }

    unsafe fn draw_frame(&mut self, current_frame: usize) -> bool {
        let FrameData {
            command_buffer: cmd,
            fence,
            present_semaphore,
            render_semaphore,
            ..
        } = self.frames[current_frame];

        self.device
            .wait_for_fences(&[fence], true, ONE_SEC)
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
        let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        self.device
            .begin_command_buffer(cmd, &cmd_begin_info)
            .unwrap();

        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
            },
        };

        let renderpass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .framebuffer(framebuffer)
            .clear_values(&[clear_value])
            .build();
        self.device
            .cmd_begin_render_pass(cmd, &renderpass_info, vk::SubpassContents::INLINE);

        self.device
            .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);
        self.device.cmd_draw(cmd, 3, 1, 0, 0);

        self.device.cmd_end_render_pass(cmd);

        self.device.end_command_buffer(cmd).unwrap();

        //SUBMIT
        self.device.reset_fences(&[fence]).unwrap();

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[cmd])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&[present_semaphore])
            .signal_semaphores(&[render_semaphore])
            .build();
        self.device
            .queue_submit(self.graphics_queue, &[submit_info], fence)
            .unwrap();

        //PRESENTATION
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&[self.swapchain.chain])
            .wait_semaphores(&[render_semaphore])
            .image_indices(&[swapchain_image_idx])
            .build();
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

    unsafe fn destroy(&mut self) {
        self.device.destroy_pipeline(self.graphics_pipeline, None);
        self.device
            .destroy_pipeline_layout(self.pipeline_layout, None);

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

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        self.device.destroy_device(None);

        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }

    fn handle_resize(&mut self, window: &winit::window::Window) -> bool {
        unsafe { self.device.device_wait_idle().unwrap() };
        let physical_size = window.inner_size();
        if physical_size.width == 0 || physical_size.height == 0 {
            return true;
        }

        //swapchain
        let surface_support =
            SurfaceSupport::get(self.physical_device, self.surface, &self.surface_loader).unwrap();
        let new_swapchain = swapchain_scop::create_swapchain(
            &self.swapchain_loader,
            &self.device,
            (physical_size.width, physical_size.height),
            &surface_support,
            self.surface,
            &self.queue_families,
            Some(self.swapchain.chain),
        )
        .unwrap();

        //clean
        unsafe { self.device.destroy_pipeline(self.graphics_pipeline, None) }

        for &framebuffer in &self.framebuffers {
            unsafe { self.device.destroy_framebuffer(framebuffer, None) }
        }

        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
        }

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        //init
        self.swapchain = new_swapchain;

        self.render_pass = create_default_render_pass(&self.device, &self.swapchain);
        self.framebuffers = create_framebuffers(&self.device, &self.swapchain, self.render_pass);
        self.graphics_pipeline = create_graphics_pipeline(
            &self.device,
            self.render_pass,
            self.pipeline_layout,
            &self.swapchain,
        );

        return false;
    }
}

fn get_instance(
    entry: &ash::Entry,
    debug_info: &mut vk::DebugUtilsMessengerCreateInfoEXT,
) -> anyhow::Result<ash::Instance> {
    let (_layer_names, layer_name_pointers) = conf::get_layer_names();

    let application_info = vk::ApplicationInfo::builder()
        .application_name(std::ffi::CString::new(conf::APPLICATION_NAME)?.as_c_str())
        .application_version(conf::APPLICATION_VERSION)
        .engine_name(std::ffi::CString::new(conf::ENGINE_NAME)?.as_c_str())
        .engine_version(conf::ENGINE_VERSION)
        .api_version(conf::API_VERSION)
        .build();

    let instance_create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::builder()
        .push_next(debug_info)
        .application_info(&application_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&conf::EXTENSION_NAMES)
        .build();

    Ok(unsafe { entry.create_instance(&instance_create_info, None) }?)
}

//DEVICES
struct SurfaceSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceSupport {
    pub fn get(
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::extensions::khr::Surface,
    ) -> anyhow::Result<Self> {
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };

        Ok(Self {
            capabilities,
            present_modes,
            formats,
        })
    }
}

fn check_physical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_support: &SurfaceSupport,
) -> anyhow::Result<bool, anyhow::Error> {
    let extensions: Vec<String> = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)?
            .iter()
            .map(|e| utils::i8_to_str(&e.extension_name).unwrap())
            .collect()
    };

    if conf::DEVICE_EXTENSION_NAMES
        .iter()
        .all(|e| extensions.contains(&e.to_str().map(|s| s.to_string()).unwrap()))
    {
        if conf::DEVICE_EXTENSION_NAMES.contains(&ash::extensions::khr::Swapchain::name())
            && surface_support.present_modes.is_empty()
            || surface_support.formats.is_empty()
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn get_physical_device(instance: &ash::Instance) -> anyhow::Result<vk::PhysicalDevice> {
    let phys_devs = unsafe { instance.enumerate_physical_devices()? };

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

    Ok(phys_dev)
}

//queue families
struct QueueFamilies {
    graphics: u32,
    present: u32,
}

impl QueueFamilies {
    pub fn get(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<Self> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let (q_graphics_idx, q_present_idx) = queuefamilyproperties.iter().enumerate().fold(
            (None, None),
            |(mut acc_q_graphics_idx, mut acc_q_present_idx): (Option<usize>, Option<usize>),
             (c_idx, c)| {
                if c.queue_count > 0 {
                    if c.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && unsafe {
                            surface_loader.get_physical_device_surface_support(
                                physical_device,
                                c_idx.try_into().unwrap(),
                                surface,
                            )
                        }
                        .unwrap()
                    {
                        acc_q_graphics_idx = Some(c_idx);
                        acc_q_present_idx = Some(c_idx);
                    }
                    // if c.queue_flags.contains(vk::QueueFlags::TRANSFER)
                    //     && (!c.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    //         || acc_q_present_idx.is_none())
                    // {
                    //     acc_q_present_idx = Some(c_idx);
                    // }
                }
                (acc_q_graphics_idx, acc_q_present_idx)
            },
        );

        Ok(Self {
            graphics: q_graphics_idx.unwrap().try_into()?,
            present: q_present_idx.unwrap().try_into()?,
        })
    }
}

fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<(ash::Device, QueueFamilies)> {
    let physical_device_queue_families =
        QueueFamilies::get(instance, physical_device, surface_loader, surface)?;

    let queue_priorities = [1.0];

    let mut queue_infos = vec![vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(physical_device_queue_families.graphics)
        .queue_priorities(&queue_priorities)
        .build()];

    if physical_device_queue_families.graphics != physical_device_queue_families.present {
        queue_infos.push(
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(physical_device_queue_families.present)
                .queue_priorities(&queue_priorities)
                .build(),
        );
    }

    let features = vk::PhysicalDeviceFeatures::builder().build();

    let device_extensions = conf::DEVICE_EXTENSION_NAMES
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(device_extensions.as_slice())
        .enabled_features(&features);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    Ok((device, physical_device_queue_families))
}

fn create_debug(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debugcreateinfo: &vk::DebugUtilsMessengerCreateInfoEXT,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let debug_utils_messenger =
        unsafe { debug_utils_loader.create_debug_utils_messenger(debugcreateinfo, None) }.unwrap();
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

//vkGuide
#[derive(Debug)]
struct FrameData {
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,

    fence: vk::Fence,
    render_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
}

fn create_framesdata(
    device: &ash::Device,
    graphics_family: u32,
) -> [FrameData; MAX_FRAMES_IN_FLIGHT] {
    let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        let command_pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family)
            .build();
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap();

        let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();
        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_info) }.unwrap()[0];

        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        let semaphore_info = vk::SemaphoreCreateInfo::builder().build();
        let render_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };
        let present_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };

        frames.push(FrameData {
            command_pool,
            command_buffer,
            fence,
            render_semaphore,
            present_semaphore,
        });
    }
    frames.try_into().unwrap()
}

fn create_framebuffers(
    device: &ash::Device,
    swapchain: &SwapchainScop,
    render_pass: vk::RenderPass,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = Vec::with_capacity(swapchain.image_views.len());

    //When rendering, the swapchain will give us the index of the image to render into, so we will use the framebuffer of the same index.
    for &image_view in &swapchain.image_views {
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&[image_view])
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1)
            .build();
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };
        framebuffers.push(framebuffer);
    }

    framebuffers
}

//GRAPHICS
fn create_pipeline_layout(device: &ash::Device) -> vk::PipelineLayout {
    let info = vk::PipelineLayoutCreateInfo::builder().build();

    unsafe { device.create_pipeline_layout(&info, None).unwrap() }
}

fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    layout: vk::PipelineLayout,
    swapchain: &SwapchainScop,
) -> vk::Pipeline {
    //SHADERS
    let main_entry = std::ffi::CString::new("main").unwrap();

    let tri_vert_module = create_shader_module(device, "./shaders/tri.vert.spv").unwrap();
    let tri_vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(tri_vert_module)
        .name(main_entry.as_c_str())
        .build();
    let tri_frag_module = create_shader_module(device, "./shaders/tri.frag.spv").unwrap();
    let tri_frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(tri_frag_module)
        .name(main_entry.as_c_str())
        .build();

    let colored_tri_vert_module =
        create_shader_module(device, "./shaders/colored_tri.vert.spv").unwrap();
    let colored_tri_vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(colored_tri_vert_module)
        .name(main_entry.as_c_str())
        .build();
    let colored_tri_frag_module =
        create_shader_module(device, "./shaders/colored_tri.frag.spv").unwrap();
    let colored_tri_frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(colored_tri_frag_module)
        .name(main_entry.as_c_str())
        .build();

    //PIPELINE DEFAULTS
    let viewport = vk::Viewport::builder()
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .max_depth(1.0)
        .build();
    let viewports = [viewport];
    let scissor = vk::Rect2D::builder().extent(swapchain.extent).build();
    let scissors = [scissor];

    let color_blend_attachments_state = [vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)
        .build()];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachments_state);

    //TRI_COLORED PIPELINE
    let stages = [colored_tri_vert_stage, colored_tri_frag_stage];
    let mut template_pipeline_builder = GraphicsPipeline::builder();
    template_pipeline_builder.input_assembly_state = template_pipeline_builder
        .input_assembly_state
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    template_pipeline_builder.rasterization_state = template_pipeline_builder
        .rasterization_state
        .polygon_mode(vk::PolygonMode::FILL);
    template_pipeline_builder.viewport_state = template_pipeline_builder
        .viewport_state
        .viewports(&viewports)
        .scissors(&scissors);

    let template_pipeline_builded = template_pipeline_builder.build();
    let tri_colored_pipeline_info = template_pipeline_builded
        .create_pipeline_builder()
        .stages(&stages)
        .layout(layout)
        .render_pass(render_pass)
        .color_blend_state(&color_blend_state);

    //TRI PIPELINE
    let stages = [tri_vert_stage, tri_frag_stage];
    let mut template_pipeline_builder = GraphicsPipeline::builder();
    template_pipeline_builder.input_assembly_state = template_pipeline_builder
        .input_assembly_state
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    template_pipeline_builder.viewport_state = template_pipeline_builder
        .viewport_state
        .viewports(&viewports)
        .scissors(&scissors);
    template_pipeline_builder.rasterization_state = template_pipeline_builder
        .rasterization_state
        .polygon_mode(vk::PolygonMode::FILL);

    let template_pipeline_builded = template_pipeline_builder.build();
    let tri_pipeline_info = template_pipeline_builded
        .create_pipeline_builder()
        .stages(&stages)
        .layout(layout)
        .render_pass(render_pass)
        .color_blend_state(&color_blend_state);

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[tri_colored_pipeline_info.build(), tri_pipeline_info.build()],
                None,
            )
            .unwrap()
    };

    unsafe { device.destroy_shader_module(tri_frag_module, None) };
    unsafe { device.destroy_shader_module(tri_vert_module, None) };
    unsafe { device.destroy_shader_module(colored_tri_frag_module, None) };
    unsafe { device.destroy_shader_module(colored_tri_vert_module, None) };

    pipelines[0]
}

fn create_default_render_pass(device: &ash::Device, swapchain: &SwapchainScop) -> vk::RenderPass {
    let color_attachment_desc = vk::AttachmentDescription::builder()
        .format(swapchain.surface_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0) //index link to renderpass.attachments
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .build();

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&[color_attachment_desc])
        .subpasses(&[subpass])
        .build();
    unsafe { device.create_render_pass(&render_pass_info, None).unwrap() }
}

fn create_shader_module(
    device: &ash::Device,
    filename: &str,
) -> anyhow::Result<ash::vk::ShaderModule> {
    let mut shader_file = std::fs::File::open(filename).unwrap();
    let shader_code = ash::util::read_spv(&mut shader_file).unwrap();

    let createinfo = ash::vk::ShaderModuleCreateInfo::builder().code(&shader_code);
    Ok(unsafe { device.create_shader_module(&createinfo, None)? })
}

//INITIALIZERS

//DEBUG
fn create_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
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
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build()
}
