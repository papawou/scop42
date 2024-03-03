use anyhow::Ok;
use ash::vk;
use winit::platform::windows::WindowExtWindows;

fn main() -> anyhow::Result<()> {
    let entry = unsafe { ash::Entry::load()? };

    //window
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Hello window!")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
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

        let layer_names: Vec<std::ffi::CString> =
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();
        let extension_name_pointers: Vec<*const i8> = vec![
            ash::extensions::ext::DebugUtils::name().as_ptr(),
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::Win32Surface::name().as_ptr(),
        ];

        let instance_info = get_instance_info(
            &layer_name_pointers,
            &extension_name_pointers,
            &mut debug_info,
        )?;
        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        let mut data = AppData::default();

        let (debug_utils, utils_messenger) = create_debug(&entry, &instance, &debug_info);
        //  surface
        let win_surface_loader = ash::extensions::khr::Win32Surface::new(&entry, &instance);
        let surface = unsafe { win_surface_loader.create_win32_surface(&window_info, None) }?;
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

        data.debug_utils = Some(debug_utils);
        data.utils_messenger = utils_messenger;
        data.surface = surface;
        data.surface_loader = Some(surface_loader);

        let device = create_logical_device(
            &instance,
            &layer_name_pointers,
            &extension_name_pointers,
            &mut data,
        )?;

        Ok(Self {
            entry,
            instance,
            device,
            data,
        })
    }

    unsafe fn destroy(&mut self) {
        self.device.destroy_device(None);

        match &self.data.surface_loader {
            Some(p) => p.destroy_surface(self.data.surface, None),
            _ => (),
        }

        match &self.data.debug_utils {
            Some(p) => p.destroy_debug_utils_messenger(self.data.utils_messenger, None),
            _ => (),
        }

        self.instance.destroy_instance(None);
    }
}

#[derive(Default)]
struct AppData {
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
    utils_messenger: vk::DebugUtilsMessengerEXT,
    physical_device_queue_families: (u32, u32),
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,

    surface: vk::SurfaceKHR,
    surface_loader: Option<ash::extensions::khr::Surface>,
}

fn get_instance_info(
    layer_name_pointers: &Vec<*const i8>,
    extension_name_pointers: &Vec<*const i8>,
    debug_info: &mut vk::DebugUtilsMessengerCreateInfoEXT,
) -> anyhow::Result<vk::InstanceCreateInfo> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(std::ffi::CString::new("AppName")?.as_c_str())
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(std::ffi::CString::new("No Engine")?.as_c_str())
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::make_api_version(0, 1, 0, 0))
        .build();

    let instance_create_info: vk::InstanceCreateInfo = vk::InstanceCreateInfo::builder()
        .push_next(debug_info)
        .application_info(&application_info)
        .enabled_layer_names(layer_name_pointers)
        .enabled_extension_names(extension_name_pointers)
        .build();

    Ok(instance_create_info)
}

//DEVICES

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
                "NVIDIA GeForce RTX 4070 Ti" => Some(p),
                _ => None,
            }
        })
        .unwrap();

    Ok(phys_dev)
}

fn get_queue_families(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<(u32, u32)> {
    let queuefamilyproperties =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let (q_graphics_idx, q_transfer_idx) = queuefamilyproperties.iter().enumerate().fold(
        (None, None),
        |(mut acc_q_graphics_idx, mut acc_q_transfer_idx): (Option<usize>, Option<usize>),
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
                }
                if c.queue_flags.contains(vk::QueueFlags::TRANSFER)
                    && (!c.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        || acc_q_transfer_idx.is_none())
                {
                    acc_q_transfer_idx = Some(c_idx);
                }
            }
            (acc_q_graphics_idx, acc_q_transfer_idx)
        },
    );
    Ok((
        q_graphics_idx.unwrap().try_into()?,
        q_transfer_idx.unwrap().try_into()?,
    ))
}

fn create_logical_device(
    instance: &ash::Instance,
    layer_name_pointers: &Vec<*const i8>,
    extension_name_pointers: &Vec<*const i8>,
    data: &mut AppData,
) -> anyhow::Result<ash::Device> {
    let physical_device = get_physical_device(&instance)?;
    let physical_device_queue_families = get_queue_families(
        instance,
        physical_device,
        data.surface_loader.as_ref().unwrap(),
        data.surface,
    )?;

    let queue_priorities = [1.0];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(physical_device_queue_families.0)
            .queue_priorities(&queue_priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(physical_device_queue_families.1)
            .queue_priorities(&queue_priorities)
            .build(),
    ];

    let features = vk::PhysicalDeviceFeatures::builder();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(layer_name_pointers)
        .enabled_features(&features);
    //.enabled_extension_names(extension_name_pointers);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    data.physical_device_queue_families = physical_device_queue_families;
    data.graphics_queue = unsafe { device.get_device_queue(physical_device_queue_families.0, 0) };
    data.transfer_queue = unsafe { device.get_device_queue(physical_device_queue_families.1, 0) };

    Ok(device)
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
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
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
    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(debugcreateinfo, None) }.unwrap();
    (debug_utils, utils_messenger)
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
