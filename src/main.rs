use ash::vk::{self, QueueFamilyProperties};

fn get_phys_devs(
    instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties), Box<dyn std::error::Error>> {
    let phys_devs = unsafe { instance.enumerate_physical_devices()? };
    for p in phys_devs {
        let props = unsafe { instance.get_physical_device_properties(p) };
        let name = unsafe { std::ffi::CStr::from_ptr(props.device_name.as_ptr()) }.to_str()?;

        if name == "NVIDIA GeForce RTX 4070 Ti" {
            return Ok((p, props));
        }
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "device not found",
    )))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = unsafe { ash::Entry::load()? };

    //window
    let event_loop = winit::event_loop::EventLoop::new();
    let win = winit::window::Window::new(&event_loop)?;

    //app
    let enginename = std::ffi::CString::new("BlackMotor")?;
    let appname = std::ffi::CString::new("HoleFuel")?;
    let app_info = vk::ApplicationInfo::builder()
        .engine_name(&enginename)
        .engine_version(vk::make_api_version(0, 0, 42, 0))
        .application_name(&appname)
        .application_version(vk::make_api_version(0, 0, 42, 0))
        .api_version(vk::make_api_version(0, 1, 3, 0));

    //debug_info
    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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
        .pfn_user_callback(Some(vulkan_debug_utils_callback));

    //interface
    let layer_names: Vec<std::ffi::CString> =
        vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation")?];
    let layer_names_ptr: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let extension_names_ptr: Vec<*const i8> =
        vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    let instance_create_info: vk::InstanceCreateInfoBuilder<'_> = vk::InstanceCreateInfo::builder()
        .push_next(&mut debug_create_info)
        .application_info(&app_info)
        .enabled_layer_names(&layer_names_ptr)
        .enabled_extension_names(&extension_names_ptr);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None)? };

    //PHYSICAL_DEVICE
    let (physical_device, physical_device_properties) = get_phys_devs(&instance)?;
    let queuefamilyproperties =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    dbg!(&queuefamilyproperties);
    let (q_graphics_idx, q_transfer_idx) = {
        let (q_graphics_idx, q_transfer_idx) = queuefamilyproperties.iter().enumerate().fold(
            (None, None),
            |(mut acc_q_graphics_idx, mut acc_q_transfer_idx): (Option<usize>, Option<usize>),
             (c_idx, c)| {
                if c.queue_count > 0 {
                    if c.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
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
        (q_graphics_idx.unwrap(), q_transfer_idx.unwrap())
    };
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(q_graphics_idx.try_into()?)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(q_transfer_idx.try_into()?)
            .queue_priorities(&priorities)
            .build(),
    ];
    dbg!(queue_infos);
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layer_names_ptr);
    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue = unsafe { logical_device.get_device_queue(q_graphics_idx.try_into()?, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(q_transfer_idx.try_into()?, 0) };

    //Clean
    unsafe {
        logical_device.destroy_device(None);
        debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
        instance.destroy_instance(None)
    };

    Ok(())
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
