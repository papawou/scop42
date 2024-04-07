mod conf;
mod swapchain;
mod utils;
mod vertex;

use anyhow::Ok;
use ash::vk::{self, MemoryPropertyFlags};
use conf::MAX_FRAMES_IN_FLIGHT;
use swapchain::SwapchainScop;
use vertex::Vertex;
use winit::{platform::windows::WindowExtWindows, raw_window_handle::HasWindowHandle};

const VERTICES: [Vertex; 3] = [
    Vertex::new(glam::vec2(0.0, -0.5), glam::vec3(1.0, 1.0, 1.0)),
    Vertex::new(glam::vec2(0.5, 0.5), glam::vec3(0.0, 1.0, 0.0)),
    Vertex::new(glam::vec2(-0.5, 0.5), glam::vec3(0.0, 0.0, 1.0)),
];

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
    swapchain: swapchain::SwapchainScop,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_sempahores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>,

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

    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    graphics_pipelines: Vec<vk::Pipeline>,
    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl App {
    fn create(entry: ash::Entry, window: &winit::window::Window) -> anyhow::Result<Self> {
        //window specs
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

        let swapchain = swapchain::create_swapchain(
            &swapchain_loader,
            &device,
            (window_physical_size.width, window_physical_size.height),
            &surface_support,
            surface,
            &queue_families,
            None,
        )?;

        let (image_available_semaphores, render_finished_sempahores, inflight_fences) =
            create_sync_objs(&device);

        //global?
        let pipeline_layout = create_pipeline_layout(&device);
        let render_pass = create_render_pass(&device, &swapchain);
        let graphics_pipelines =
            create_graphics_pipeline(&device, &swapchain, pipeline_layout, render_pass);

        let framebuffers = create_framebuffers(&device, &swapchain, render_pass);

        let command_pool = create_command_pool(&device, queue_families.graphics);

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
            &device,
            unsafe { instance.get_physical_device_memory_properties(physical_device) },
            std::mem::size_of_val(&VERTICES) as vk::DeviceSize,
            command_pool,
            graphics_queue,
        );

        let command_buffers = create_command_buffers(&device, &command_pool);

        Ok(Self {
            entry,
            instance,
            device,

            swapchain_loader,
            swapchain,

            image_available_semaphores,
            render_finished_sempahores,
            inflight_fences,

            debug_utils_loader,
            debug_utils_messenger,

            surface_loader,
            surface,

            physical_device,
            queue_families,
            graphics_queue,
            present_queue,

            pipeline_layout,
            render_pass,
            graphics_pipelines,
            framebuffers,

            command_pool,
            vertex_buffer,
            vertex_buffer_memory,
            command_buffers,
        })
    }

    unsafe fn draw_frame(&mut self, current_frame: usize) -> bool {
        let inflight_fence = self.inflight_fences[current_frame];
        let image_avalaible_semaphore = self.image_available_semaphores[current_frame];
        let render_finished_semaphore = self.render_finished_sempahores[current_frame];
        let command_buffer = self.command_buffers[current_frame];

        self.device
            .wait_for_fences(&[inflight_fence], true, u64::MAX)
            .unwrap();

        let frame_idx = match self.swapchain_loader.acquire_next_image(
            self.swapchain.chain,
            u64::MAX,
            image_avalaible_semaphore,
            vk::Fence::null(),
        ) {
            Result::Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                return true
            }
            Err(e) => panic!("{}", e),
        };

        self.device
            .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
            .unwrap();

        record_command_buffer(
            &self.device,
            &command_buffer,
            &self.swapchain,
            &self.render_pass,
            &self.framebuffers[frame_idx as usize],
            &self.graphics_pipelines[0],
            &[self.vertex_buffer],
        );

        self.device.reset_fences(&[inflight_fence]).unwrap();
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .wait_semaphores(&[image_avalaible_semaphore])
            .signal_semaphores(&[render_finished_semaphore])
            .build();
        self.device
            .queue_submit(self.graphics_queue, &[submit_info], inflight_fence)
            .unwrap();

        //PRESENTATION
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&[self.swapchain.chain])
            .image_indices(&[frame_idx])
            .wait_semaphores(&[render_finished_semaphore])
            .build();
        let present_res = self
            .swapchain_loader
            .queue_present(self.present_queue, &present_info);

        match present_res {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                return true
            }
            Err(e) => panic!("{}", e),
            _ => (),
        };

        false
    }

    unsafe fn destroy(&mut self) {
        for &image_available_semaphore in self.image_available_semaphores.iter() {
            self.device
                .destroy_semaphore(image_available_semaphore, None)
        }
        self.image_available_semaphores.clear();

        for &render_finished_semaphore in self.render_finished_sempahores.iter() {
            self.device
                .destroy_semaphore(render_finished_semaphore, None)
        }
        self.render_finished_sempahores.clear();

        for &inflight_fence in self.inflight_fences.iter() {
            self.device.destroy_fence(inflight_fence, None)
        }
        self.inflight_fences.clear();

        for &framebuffer in self.framebuffers.iter() {
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.framebuffers.clear();

        for &pipeline in self.graphics_pipelines.iter() {
            self.device.destroy_pipeline(pipeline, None);
        }
        self.graphics_pipelines.clear();

        self.device.destroy_render_pass(self.render_pass, None);

        self.device
            .destroy_pipeline_layout(self.pipeline_layout, None);

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        self.device.destroy_buffer(self.vertex_buffer, None);
        self.device.free_memory(self.vertex_buffer_memory, None);

        self.device.destroy_command_pool(self.command_pool, None);

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
        let new_swapchain = swapchain::create_swapchain(
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
        for &framebuffer in self.framebuffers.iter() {
            unsafe { self.device.destroy_framebuffer(framebuffer, None) };
        }
        self.framebuffers.clear();

        for &pipeline in self.graphics_pipelines.iter() {
            unsafe { self.device.destroy_pipeline(pipeline, None) };
        }
        self.graphics_pipelines.clear();

        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
        }

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        //init
        self.swapchain = new_swapchain;
        self.render_pass = create_render_pass(&self.device, &self.swapchain);

        self.graphics_pipelines = create_graphics_pipeline(
            &self.device,
            &self.swapchain,
            self.pipeline_layout,
            self.render_pass,
        );

        self.framebuffers = create_framebuffers(&self.device, &self.swapchain, self.render_pass);

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

//GRAPHICS
fn create_graphics_pipeline(
    device: &ash::Device,
    swapchain: &swapchain::SwapchainScop, //use fields?
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> Vec<vk::Pipeline> {
    let main_entry = std::ffi::CString::new("main").unwrap();
    let mut vert_shader_file = std::fs::File::open("./shaders/vert.spv").unwrap();
    let vert_shader_code = ash::util::read_spv(&mut vert_shader_file).unwrap();
    let vert_shader_module = create_shader_module(device, &vert_shader_code).unwrap();
    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(main_entry.as_c_str())
        .build();

    let mut frag_shader_file = std::fs::File::open("./shaders/frag.spv").unwrap();
    let frag_shader_code = ash::util::read_spv(&mut frag_shader_file).unwrap();
    let frag_shader_module = create_shader_module(device, &frag_shader_code).unwrap();
    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(main_entry.as_c_str())
        .build();
    let shaders_stage_createinfo = vec![frag_shader_stage_info, vert_shader_stage_info];

    //pipeline_dynamic_createinfo
    let pipeline_dynamics = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let pipeline_dynamic_createinfo = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&pipeline_dynamics)
        .build();

    let binding_description = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_createinfo = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_description)
        .vertex_attribute_descriptions(&attribute_descriptions)
        .build();

    let assembly_input_createinfo = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();

    let viewport_createinfo = get_pipeline_viewport_createinfo(swapchain.extent);

    let rasterization_createinfo = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .build();

    let multisample_createinfo = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .build();

    let color_blend_attachments = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(true)
        //color
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        //alpha
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build();
    let color_blend_createinfo = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&[color_blend_attachments])
        .build();

    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shaders_stage_createinfo)
        .vertex_input_state(&vertex_input_createinfo)
        .input_assembly_state(&assembly_input_createinfo)
        .viewport_state(&viewport_createinfo)
        .rasterization_state(&rasterization_createinfo)
        .multisample_state(&multisample_createinfo)
        .color_blend_state(&color_blend_createinfo)
        .dynamic_state(&pipeline_dynamic_createinfo)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

    let pipeline = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
    }
    .unwrap();

    unsafe { device.destroy_shader_module(vert_shader_module, None) };
    unsafe { device.destroy_shader_module(frag_shader_module, None) };

    pipeline
}

fn create_pipeline_layout(device: &ash::Device) -> vk::PipelineLayout {
    let pipeline_layout_createinfo = vk::PipelineLayoutCreateInfo::builder().build();

    unsafe { device.create_pipeline_layout(&pipeline_layout_createinfo, None) }.unwrap()
}

fn get_pipeline_viewport_createinfo(extent: vk::Extent2D) -> vk::PipelineViewportStateCreateInfo {
    let viewport = vk::Viewport::builder()
        .width(extent.width as f32)
        .height(extent.height as f32)
        .max_depth(1.0)
        .build();

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(extent)
        .build();

    vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&[viewport])
        .scissors(&[scissor])
        .build()
}

fn create_shader_module(
    device: &ash::Device,
    code: &[u32],
) -> anyhow::Result<ash::vk::ShaderModule> {
    let createinfo = ash::vk::ShaderModuleCreateInfo::builder().code(code);
    Ok(unsafe { device.create_shader_module(&createinfo, None)? })
}

//FRAMEBUFFERS
fn create_framebuffers(
    device: &ash::Device,
    swapchain: &SwapchainScop, //use fields? no too tide
    render_pass: vk::RenderPass,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![];

    for &p in &swapchain.image_views {
        let p_imageview = [p];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&p_imageview)
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1);

        framebuffers.push(unsafe { device.create_framebuffer(&framebuffer_info, None) }.unwrap());
    }

    framebuffers
}

fn create_render_pass(
    device: &ash::Device,
    swapchain: &swapchain::SwapchainScop, //use field?
) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription::builder()
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
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let subpass_dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::NONE)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .build();

    let subpass_description = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .build();

    let render_pass_createinfo = vk::RenderPassCreateInfo::builder()
        .attachments(&[color_attachment])
        .subpasses(&[subpass_description])
        .dependencies(&[subpass_dependency])
        .build();

    unsafe { device.create_render_pass(&render_pass_createinfo, None) }.unwrap()
}

//COMMAND POOL
fn create_command_pool(device: &ash::Device, graphics_family: u32) -> vk::CommandPool {
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(graphics_family);

    unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap()
}

fn create_command_buffers(
    device: &ash::Device,
    &command_pool: &vk::CommandPool,
) -> Vec<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
        .build();

    unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap()
}

fn record_command_buffer(
    device: &ash::Device,
    &command_buffer: &vk::CommandBuffer,
    swapchain: &SwapchainScop,
    &render_pass: &vk::RenderPass,
    &framebuffer: &vk::Framebuffer,
    &graphics_pipeline: &vk::Pipeline,
    vertex_buffers: &[vk::Buffer],
) {
    let begin_info = vk::CommandBufferBeginInfo::builder().build();
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();

    let clear_values = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };
    let render_pass_info = vk::RenderPassBeginInfo::builder()
        .render_pass(render_pass)
        .framebuffer(framebuffer)
        .render_area(vk::Rect2D {
            extent: swapchain.extent,
            ..Default::default()
        })
        .clear_values(&[clear_values])
        .build();
    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        )
    };

    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            graphics_pipeline,
        )
    };

    let viewport = vk::Viewport::builder()
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build();
    unsafe { device.cmd_set_viewport(command_buffer, 0, &[viewport]) };

    let scissor = vk::Rect2D::builder().extent(swapchain.extent).build();
    unsafe { device.cmd_set_scissor(command_buffer, 0, &[scissor]) }

    let offsets = [0];
    unsafe { device.cmd_bind_vertex_buffers(command_buffer, 0, vertex_buffers, &offsets) };

    unsafe { device.cmd_draw(command_buffer, VERTICES.len() as u32, 1, 0, 0) };
    unsafe { device.cmd_end_render_pass(command_buffer) };
    unsafe { device.end_command_buffer(command_buffer) }.unwrap();
}

//draw commands
fn create_sync_objs(
    device: &ash::Device,
) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
    let mut image_available_semaphores = vec![];
    let mut render_finished_semaphores = vec![];
    let mut inflight_fences = vec![];

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        let image_available_semaphore =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }.unwrap();
        image_available_semaphores.push(image_available_semaphore);

        let render_finished_semaphore =
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }.unwrap();
        render_finished_semaphores.push(render_finished_semaphore);

        let inflight_fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let inflight_fence = unsafe { device.create_fence(&inflight_fence_info, None) }.unwrap();
        inflight_fences.push(inflight_fence);
    }
    (
        image_available_semaphores,
        render_finished_semaphores,
        inflight_fences,
    )
}

//Memory
fn get_memory_type_index(
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    memory_requirements: &vk::MemoryRequirements,
    memory_properties: vk::MemoryPropertyFlags,
) -> usize {
    device_memory_properties
        .memory_types
        .iter()
        .enumerate()
        .find(|(i, value)| {
            ((memory_requirements.memory_type_bits) & (1 << i)) != 0
                && value.property_flags & memory_properties == memory_properties
        })
        .map(|(i, _)| i)
        .unwrap()
}

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
            //vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build()
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

//BUFFERS
fn create_buffer(
    device: &ash::Device,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    buffer_size: vk::DeviceSize,
    buffer_usage: vk::BufferUsageFlags,
    memory_properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer = {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();
        unsafe { device.create_buffer(&buffer_info, None) }.unwrap()
    };

    let buffer_memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = get_memory_type_index(
        &device_memory_properties,
        &buffer_memory_requirements,
        memory_properties,
    );

    let buffer_memory = {
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .memory_type_index(memory_type_index as u32)
            .allocation_size(buffer_memory_requirements.size)
            .build();
        unsafe { device.allocate_memory(&allocate_info, None) }.unwrap()
    };
    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0) }.unwrap();

    (buffer, buffer_memory)
}

fn create_vertex_buffer(
    device: &ash::Device,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    buffer_size: vk::DeviceSize,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, vk::DeviceMemory) {
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        &device,
        device_memory_properties,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let staging_data_ptr = unsafe {
        device.map_memory(
            staging_buffer_memory,
            0,
            buffer_size,
            vk::MemoryMapFlags::empty(),
        )
    }
    .unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(
            VERTICES.as_ptr(),
            staging_data_ptr as *mut Vertex,
            VERTICES.len(),
        );
    }
    unsafe { device.unmap_memory(staging_buffer_memory) };

    //staging copied

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        &device,
        device_memory_properties,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    copy_buffer(
        device,
        staging_buffer,
        vertex_buffer,
        buffer_size,
        command_pool,
        queue,
    );

    unsafe { device.destroy_buffer(staging_buffer, None) };
    unsafe { device.free_memory(staging_buffer_memory, None) };

    (vertex_buffer, vertex_buffer_memory)
}

fn copy_buffer(
    device: &ash::Device,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) {
    let allocation_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1)
        .build();

    let command_buffer = unsafe { device.allocate_command_buffers(&allocation_info) }.unwrap()[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();
    let copy_region = vk::BufferCopy::builder()
        .src_offset(0)
        .dst_offset(0)
        .size(size)
        .build();
    unsafe { device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]) };
    unsafe { device.end_command_buffer(command_buffer) }.unwrap();

    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&[command_buffer])
        .build();

    unsafe { device.queue_submit(queue, &[submit_info], vk::Fence::null()) }.unwrap();
    unsafe { device.queue_wait_idle(queue) }.unwrap();

    unsafe { device.free_command_buffers(command_pool, &[command_buffer]) }
}
