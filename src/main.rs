mod conf;
mod swapchain;
mod utils;

use anyhow::Ok;
use ash::vk;
use utils::read_file;
use winit::platform::windows::WindowExtWindows;

fn main() -> anyhow::Result<()> {
    let entry = unsafe { ash::Entry::load()? };

    //window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Hello window!")
        .with_inner_size(winit::dpi::LogicalSize::new(
            conf::WINDOW_WIDTH,
            conf::WINDOW_HEIGHT,
        ))
        .build(&event_loop)?;

    let mut app = App::create(entry, &window)?;

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::CloseRequested,
            ..
        } => {
            unsafe { app.destroy() };
            *control_flow = winit::event_loop::ControlFlow::Exit;
        }
        winit::event::Event::MainEventsCleared => {} //request_redraw
        winit::event::Event::RedrawRequested(_) => {} //render
        _ => {}
    });
}

struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,

    //swapchain
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: swapchain::SwapchainScop,

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
}

impl App {
    fn create(entry: ash::Entry, window: &winit::window::Window) -> anyhow::Result<Self> {
        let window_physical_size = window.inner_size();
        let window_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(window.hinstance())
            .hwnd(window.hwnd());

        let mut debug_info: vk::DebugUtilsMessengerCreateInfoEXT = create_debug_info();

        let (_layer_names, layer_name_pointers) = conf::get_layer_names();

        let instance_info = get_instance_info(&layer_name_pointers, &mut debug_info)?;
        let instance = unsafe { entry.create_instance(&instance_info, None)? };

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
        )?;

        let pipeline_layout = create_graphics_pipeline(&device, &swapchain);

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

            pipeline_layout,
        })
    }

    unsafe fn destroy(&mut self) {
        self.device
            .destroy_pipeline_layout(self.pipeline_layout, None);

        for &image_view in self.swapchain.image_views.iter() {
            self.device.destroy_image_view(image_view, None);
        }

        self.swapchain_loader
            .destroy_swapchain(self.swapchain.chain, None);

        self.device.destroy_device(None);

        self.surface_loader.destroy_surface(self.surface, None);
        self.debug_utils_loader
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }
}

fn get_instance_info(
    layer_name_pointers: &Vec<*const i8>,
    debug_info: &mut vk::DebugUtilsMessengerCreateInfoEXT,
) -> anyhow::Result<vk::InstanceCreateInfo> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(std::ffi::CString::new(conf::APPLICATION_NAME)?.as_c_str())
        .application_version(conf::APPLICATION_VERSION)
        .engine_name(std::ffi::CString::new(conf::ENGINE_NAME)?.as_c_str())
        .engine_version(conf::ENGINE_VERSION)
        .api_version(conf::API_VERSION)
        .build();

    //let (_, layer_names_ptr) = conf::get_layer_names();

    let instance_create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::builder()
        .push_next(debug_info)
        .application_info(&application_info)
        .enabled_layer_names(layer_name_pointers)
        .enabled_extension_names(&conf::EXTENSION_NAMES)
        .build();

    Ok(instance_create_info)
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

//SHADERS
type ShaderCode = Vec<u8>;
fn create_graphics_pipeline(
    device: &ash::Device,
    swapchain: &swapchain::SwapchainScop,
) -> vk::PipelineLayout {
    let shaders_stage_createinfo = get_shaders_stage_createinfo(device);

    //pipeline_dynamic_createinfo
    let pipeline_dynamics = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let pipeline_dynamic_createinfo = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&pipeline_dynamics)
        .build();

    let vertex_input_createinfo = vk::PipelineVertexInputStateCreateInfo::builder().build();
    let assembly_input_createinfo = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build();

    let viewport_createinfo = get_pipeline_viewport_createinfo(swapchain);

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

    let color_blend_createinfo = get_color_blend_info();

    let pipeline_layout_createinfo = vk::PipelineLayoutCreateInfo::builder();
    //todo call destroy pipeline_layout
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pipeline_layout_createinfo, None) }.unwrap();

    pipeline_layout
}

fn get_color_blend_info() -> vk::PipelineColorBlendStateCreateInfo {
    let color_blend_attachments = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
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

    vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&[color_blend_attachments])
        .build()
}

fn get_shaders_stage_createinfo(device: &ash::Device) -> Vec<vk::PipelineShaderStageCreateInfo> {
    let vert_shader_code: ShaderCode = read_file("./shaders/vert.spv").unwrap();
    let vert_shader_module =
        create_shader_module(device, &utils::from_u8_to_u32(&vert_shader_code)).unwrap();
    let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(std::ffi::CString::new("main").unwrap().as_c_str())
        .build();

    let frag_shader_code: ShaderCode = read_file("./shaders/frag.spv").unwrap();
    let frag_shader_module =
        create_shader_module(device, &utils::from_u8_to_u32(&frag_shader_code)).unwrap();
    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(std::ffi::CString::new("main").unwrap().as_c_str())
        .build();

    vec![frag_shader_stage_info, vert_shader_stage_info]
}

fn get_pipeline_viewport_createinfo(
    swapchain: &swapchain::SwapchainScop,
) -> vk::PipelineViewportStateCreateInfo {
    let viewport = vk::Viewport::builder()
        .width(swapchain.extent.width as f32)
        .height(swapchain.extent.height as f32)
        .max_depth(1.0)
        .build();

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain.extent)
        .build();

    vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .viewports(&[viewport])
        .scissor_count(1)
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
