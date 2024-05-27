mod conf;
mod graphics_pipeline;
mod mesh;
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
    Vertex::new(
        glam::vec2(-0.5, -0.5),
        glam::vec3(1.0, 0.0, 0.0),
        glam::Vec3::ZERO,
    ),
    Vertex::new(
        glam::vec2(0.5, -0.5),
        glam::vec3(0.0, 1.0, 0.0),
        glam::Vec3::ZERO,
    ),
    Vertex::new(
        glam::vec2(0.5, 0.5),
        glam::vec3(0.0, 0.0, 1.0),
        glam::Vec3::ZERO,
    ),
    Vertex::new(
        glam::vec2(-0.5, 0.5),
        glam::vec3(1.0, 1.0, 1.0),
        glam::Vec3::ZERO,
    ),
];
const INDEX_VERTICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

enum GraphicsPipelineType {
    Tri,
    TriColored,
    Mesh,
}

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

    let mut selected_pipeline = GraphicsPipelineType::Tri;

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            winit::event::Event::AboutToWait => window.request_redraw(),
            winit::event::Event::LoopExiting => {
                unsafe { app.device.device_wait_idle() }.unwrap();
                unsafe { app.destroy() };
            }
            //WINDOW EVENTS
            winit::event::Event::WindowEvent { event, .. } => match event {
                //WINDOW MANAGENMENT
                winit::event::WindowEvent::RedrawRequested => {
                    if framebuffer_resized && unsafe { app.handle_resize(&window) } {
                        return;
                    }
                    framebuffer_resized =
                        unsafe { app.draw_frame(current_frame, &selected_pipeline) };
                    current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                }
                winit::event::WindowEvent::Resized(_) => framebuffer_resized = true,
                winit::event::WindowEvent::CloseRequested => elwt.exit(),
                //CONTROLS
                winit::event::WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key:
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW),
                            state: winit::event::ElementState::Pressed,
                            repeat: false,
                            ..
                        },
                    ..
                } => {
                    selected_pipeline = match selected_pipeline {
                        GraphicsPipelineType::Tri => GraphicsPipelineType::TriColored,
                        GraphicsPipelineType::TriColored => GraphicsPipelineType::Mesh,
                        GraphicsPipelineType::Mesh => GraphicsPipelineType::Tri,
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();

    Ok(())
}

struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,

    //vmem
    allocator: Option<vk_mem::Allocator>,

    mesh: mesh::Mesh,

    //swapchain
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: swapchain_scop::SwapchainScop,

    //debug
    debug_utils_loader: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    //device
    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    //surface
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    frames: [FrameData; MAX_FRAMES_IN_FLIGHT],
    pipeline_layout: vk::PipelineLayout,
    tri_pipeline: vk::Pipeline,
    tri_colored_pipeline: vk::Pipeline,
    mesh_pipeline: vk::Pipeline,
}

impl App {
    fn create(entry: ash::Entry, window: &winit::window::Window) -> anyhow::Result<Self> {
        //  window
        let hwnd = match window.window_handle()?.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd.get(),
            _ => panic!("Unsupported platform!"),
        };
        let hinstance = unsafe { winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null()) };
        let window_info = vk::Win32SurfaceCreateInfoKHR::default()
            .hwnd(hwnd as vk::HWND)
            .hinstance(hinstance as vk::HINSTANCE);
        let window_physical_size = window.inner_size();

        let instance = create_instance(&entry)?;
        let (debug_utils_loader, debug_utils_messenger) = setup_debug_utils(&entry, &instance);

        //  surface
        let win_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
        let surface = unsafe { win_surface_loader.create_win32_surface(&window_info, None) }?;

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

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

        //swapchain
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = swapchain_scop::create_swapchain(
            &swapchain_loader,
            &device,
            (window_physical_size.width, window_physical_size.height),
            &surface_support,
            surface,
            &queue_families,
            None,
        )?;

        let frames = create_framesdata(&device, queue_families.graphics);

        //vmem allocator
        let allocator = create_allocator(&instance, &device, physical_device);

        //graphics
        let mut mesh = mesh::Mesh::new(VERTICES.to_vec(), None);
        mesh.load(&allocator);

        let render_pass = create_default_render_pass(&device, &swapchain);
        let pipeline_layout = create_pipeline_layout(&device);
        let framebuffers = create_framebuffers(&device, &swapchain, render_pass);

        let (tri_pipeline, tri_colored_pipeline, mesh_pipeline) =
            create_graphics_pipeline(&device, render_pass, pipeline_layout, &swapchain);

        Ok(Self {
            entry,
            instance,
            device,

            allocator: Some(allocator),
            mesh,

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
            tri_pipeline,
            tri_colored_pipeline,
            mesh_pipeline,
        })
    }

    unsafe fn draw_frame(&mut self, current_frame: usize, pipeline: &GraphicsPipelineType) -> bool {
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
        let cmd_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device
            .begin_command_buffer(cmd, &cmd_begin_info)
            .unwrap();

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
            },
        }];

        let renderpass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .framebuffer(framebuffer)
            .clear_values(&clear_values);
        self.device
            .cmd_begin_render_pass(cmd, &renderpass_info, vk::SubpassContents::INLINE);

        match pipeline {
            GraphicsPipelineType::Tri => {
                self.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.tri_pipeline,
                );
                self.device.cmd_draw(cmd, 3, 1, 0, 0);
            }
            GraphicsPipelineType::TriColored => {
                self.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.tri_colored_pipeline,
                );
                self.device.cmd_draw(cmd, 3, 1, 0, 0);
            }
            GraphicsPipelineType::Mesh => {
                self.device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.mesh_pipeline,
                );

                let vertex_buffers = [self.mesh.vertex_buffer.as_ref().unwrap().buffer];
                let offsets = [0];
                self.device
                    .cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);
                self.device
                    .cmd_draw(cmd, self.mesh.vertices.len() as u32, 1, 0, 0)
            }
        }

        self.device.cmd_end_render_pass(cmd);

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

    unsafe fn destroy(&mut self) {
        self.device.destroy_pipeline(self.mesh_pipeline, None);
        self.device
            .destroy_pipeline(self.tri_colored_pipeline, None);
        self.device.destroy_pipeline(self.tri_pipeline, None);

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

        if let Some(allocator) = &self.allocator {
            self.mesh.unload(allocator);
            self.allocator = None; //free vkmem::Allocator
        }

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        self.device.destroy_device(None);

        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }

    unsafe fn handle_resize(&mut self, window: &winit::window::Window) -> bool {
        self.device.device_wait_idle().unwrap();
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
        self.device.destroy_pipeline(self.mesh_pipeline, None);
        self.device
            .destroy_pipeline(self.tri_colored_pipeline, None);
        self.device.destroy_pipeline(self.tri_pipeline, None);

        for &framebuffer in &self.framebuffers {
            self.device.destroy_framebuffer(framebuffer, None)
        }

        self.device.destroy_render_pass(self.render_pass, None);

        self.swapchain
            .clean_swapchain(&self.device, &self.swapchain_loader);

        //init
        self.swapchain = new_swapchain;

        self.render_pass = create_default_render_pass(&self.device, &self.swapchain);
        self.framebuffers = create_framebuffers(&self.device, &self.swapchain, self.render_pass);
        let (tri_pipeline, tri_colored_pipeline, mesh_pipeline) = create_graphics_pipeline(
            &self.device,
            self.render_pass,
            self.pipeline_layout,
            &self.swapchain,
        );

        self.tri_pipeline = tri_pipeline;
        self.tri_colored_pipeline = tri_colored_pipeline;
        self.mesh_pipeline = mesh_pipeline;

        return false;
    }
}

fn create_instance(entry: &ash::Entry) -> anyhow::Result<ash::Instance> {
    let (_layer_names, layer_name_pointers) = conf::get_layer_names();

    let application_name = std::ffi::CString::new(conf::APPLICATION_NAME)?;
    let engine_name = std::ffi::CString::new(conf::ENGINE_NAME)?;
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
        surface_loader: &ash::khr::surface::Instance,
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
        if conf::DEVICE_EXTENSION_NAMES.contains(&ash::khr::swapchain::NAME)
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
        surface_loader: &ash::khr::surface::Instance,
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
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<(ash::Device, QueueFamilies)> {
    let physical_device_queue_families =
        QueueFamilies::get(instance, physical_device, surface_loader, surface)?;

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
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_family);
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap();

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer =
            unsafe { device.allocate_command_buffers(&command_buffer_info) }.unwrap()[0];

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        let semaphore_info = vk::SemaphoreCreateInfo::default();
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

//GRAPHICS
fn create_pipeline_layout(device: &ash::Device) -> vk::PipelineLayout {
    let info = vk::PipelineLayoutCreateInfo::default();

    unsafe { device.create_pipeline_layout(&info, None).unwrap() }
}

fn create_graphics_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    layout: vk::PipelineLayout,
    swapchain: &SwapchainScop,
) -> (vk::Pipeline, vk::Pipeline, vk::Pipeline) {
    //SHADERS
    let main_entry = std::ffi::CString::new("main").unwrap();

    let tri_vert_module = create_shader_module(device, "./shaders/tri.vert.spv").unwrap();
    let tri_vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(tri_vert_module)
        .name(main_entry.as_c_str());
    let tri_frag_module = create_shader_module(device, "./shaders/tri.frag.spv").unwrap();
    let tri_frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(tri_frag_module)
        .name(main_entry.as_c_str());

    let colored_tri_vert_module =
        create_shader_module(device, "./shaders/colored_tri.vert.spv").unwrap();
    let colored_tri_vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(colored_tri_vert_module)
        .name(main_entry.as_c_str());
    let colored_tri_frag_module =
        create_shader_module(device, "./shaders/colored_tri.frag.spv").unwrap();
    let colored_tri_frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(colored_tri_frag_module)
        .name(main_entry.as_c_str());

    let mesh_vert_module = create_shader_module(device, "./shaders/mesh.vert.spv").unwrap();
    let mesh_vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(mesh_vert_module)
        .name(main_entry.as_c_str());

    //PIPELINE DEFAULTS
    let viewport = vk::Viewport::default()
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .max_depth(1.0);
    let viewports = [viewport];
    let scissor = vk::Rect2D::default().extent(swapchain.extent);
    let scissors = [scissor];

    let color_blend_attachments_state = [vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
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

    //MESH PIPELINE
    let stages = [mesh_vert_stage, tri_frag_stage];
    let bindings = Vertex::bindings();
    let attributes = Vertex::attributes();
    let mut template_pipeline_builder = GraphicsPipeline::builder().vertex_input_state(
        vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attributes),
    );
    template_pipeline_builder.input_assembly_state = template_pipeline_builder
        .input_assembly_state
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    template_pipeline_builder.viewport_state = template_pipeline_builder
        .viewport_state
        .viewports(&viewports)
        .scissors(&scissors);

    let template_pipeline_builded = template_pipeline_builder.build();
    let mesh_pipeline_info = template_pipeline_builded
        .create_pipeline_builder()
        .stages(&stages)
        .layout(layout)
        .render_pass(render_pass)
        .color_blend_state(&color_blend_state);

    let pipelines = unsafe {
        device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[
                    tri_colored_pipeline_info,
                    tri_pipeline_info,
                    mesh_pipeline_info,
                ],
                None,
            )
            .unwrap()
    };

    unsafe { device.destroy_shader_module(mesh_vert_module, None) }
    unsafe { device.destroy_shader_module(tri_frag_module, None) };
    unsafe { device.destroy_shader_module(tri_vert_module, None) };
    unsafe { device.destroy_shader_module(colored_tri_frag_module, None) };
    unsafe { device.destroy_shader_module(colored_tri_vert_module, None) };

    (pipelines[1], pipelines[0], pipelines[2])
}

fn create_default_render_pass(device: &ash::Device, swapchain: &SwapchainScop) -> vk::RenderPass {
    let color_attachments = [vk::AttachmentReference::default()
        .attachment(0) //index link to renderpass.attachments
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
    let subpasses = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachments)];

    let attachments = [vk::AttachmentDescription::default()
        .format(swapchain.surface_format.format)
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

fn create_shader_module(
    device: &ash::Device,
    filename: &str,
) -> anyhow::Result<ash::vk::ShaderModule> {
    let mut shader_file = std::fs::File::open(filename).unwrap();
    let shader_code = ash::util::read_spv(&mut shader_file).unwrap();

    let createinfo = ash::vk::ShaderModuleCreateInfo::default().code(&shader_code);
    Ok(unsafe { device.create_shader_module(&createinfo, None)? })
}

//vkMem
struct AllocatedBuffer {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
}

fn create_allocator(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
) -> vk_mem::Allocator {
    let create_info = vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);
    unsafe { vk_mem::Allocator::new(create_info).unwrap() }
}

//INITIALIZERS

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
