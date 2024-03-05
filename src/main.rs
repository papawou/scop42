mod conf;

use anyhow::Ok;
use ash::vk;
use conf::DEVICE_EXTENSION_NAMES;
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

    unsafe { app.destroy() };
    Ok(())
}

struct App {
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,

    data: AppData,
}

impl App {
    fn create(entry: ash::Entry, window: &winit::window::Window) -> anyhow::Result<Self> {
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
        let (logical_device, queue_families) =
            create_logical_device(&instance, physical_device, &surface_loader, surface)?;
        let graphics_queue = unsafe { logical_device.get_device_queue(queue_families.graphics, 0) };
        let present_queue = unsafe { logical_device.get_device_queue(queue_families.present, 0) };

        let _ = create_swapchain(
            &surface_loader,
            surface,
            physical_device,
            &instance,
            &logical_device,
            queue_families.graphics,
        );

        Ok(Self {
            entry,
            instance,
            device: logical_device,
            data: AppData {
                debug_utils_loader,
                debug_utils_messenger,
                queue_families,
                graphics_queue,
                present_queue,
                physical_device,
                surface,
                surface_loader,
            },
        })
    }

    unsafe fn destroy(&mut self) {
        self.device.destroy_device(None);

        self.data
            .surface_loader
            .destroy_surface(self.data.surface, None);
        self.data
            .debug_utils_loader
            .destroy_debug_utils_messenger(self.data.debug_utils_messenger, None);

        self.instance.destroy_instance(None);
    }
}

struct AppData {
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,

    physical_device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    //surface
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
    // // Swapchain
    // swapchain_format: vk::Format,
    // swapchain_extent: vk::Extent2D,
    // swapchain: vk::SwapchainKHR,
    // swapchain_images: Vec<vk::Image>,
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

fn check_physical_device(
    instance: &ash::Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> anyhow::Result<()> {
    let extensions: Vec<_> = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)?
            .iter()
            .map(|e| e.extension_name)
            .collect()
    };
    let bool = DEVICE_EXTENSION_NAMES
        .iter()
        .all(|&e| extensions.contains(e));
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

fn create_logical_device(
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

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&conf::DEVICE_EXTENSION_NAMES)
        .enabled_features(&features);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    Ok((device, physical_device_queue_families))
}

//swap chain
struct SurfaceSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceSupport {
    pub fn get_surface_support(
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

fn create_swapchain(
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    instance: &ash::Instance,
    logical_device: &ash::Device,
    queue_graphics_idx: u32,
) -> anyhow::Result<(ash::extensions::khr::Swapchain, vk::SwapchainKHR)> {
    let queue_families = [queue_graphics_idx];

    let surface_support =
        SurfaceSupport::get_surface_support(physical_device, surface, surface_loader)?;

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(
            3.max(surface_support.capabilities.min_image_count)
                .min(surface_support.capabilities.max_image_count),
        )
        .image_format(surface_support.formats.first().unwrap().format)
        .image_color_space(surface_support.formats.first().unwrap().color_space)
        .image_extent(surface_support.capabilities.current_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&queue_families)
        .pre_transform(surface_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO);

    let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, logical_device);
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };

    Ok((swapchain_loader, swapchain))
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
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | {
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            },
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
