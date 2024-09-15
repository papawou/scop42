use ash::vk;

pub const PHYSICAL_DEVICE_NAME: &str = "NVIDIA GeForce RTX 2060";
//pub const PHYSICAL_DEVICE_NAME: &str = "NVIDIA GeForce RTX 4070 Ti";

pub const APPLICATION_NAME: &str = "AppName";
pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
pub const ENGINE_NAME: &str = "No Engine";
pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 = vk::API_VERSION_1_3;

pub const LAYER_NAMES: [&str; 2] = ["VK_LAYER_KHRONOS_validation", "VK_LAYER_LUNARG_monitor"];

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub fn get_layer_names() -> (Vec<std::ffi::CString>, Vec<*const i8>) {
    let layer_names: Vec<std::ffi::CString> = LAYER_NAMES
        .iter()
        .map(|&p| std::ffi::CString::new(p).unwrap())
        .collect();

    let layer_names_ptr = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    (layer_names, layer_names_ptr)
}

pub const EXTENSION_NAMES: [*const i8; 3] = [
    ash::ext::debug_utils::NAME.as_ptr(),
    ash::khr::surface::NAME.as_ptr(),
    ash::khr::win32_surface::NAME.as_ptr(),
];

pub const DEVICE_EXTENSION_NAMES: [&std::ffi::CStr; 2] = [
    ash::khr::swapchain::NAME,
    ash::khr::buffer_device_address::NAME,
];
