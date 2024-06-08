use ash::vk;

use crate::conf;

pub struct SurfaceSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceSupport {
    pub fn new(
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
    ) -> Self {
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };
        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()
        };

        Self {
            capabilities,
            present_modes,
            formats,
        }
    }

    pub fn is_physical_device_compatible(
        &self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let extensions: Vec<String> = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .unwrap()
                .iter()
                .map(|e| crate::utils::i8_to_str(&e.extension_name).unwrap())
                .collect()
        };

        if conf::DEVICE_EXTENSION_NAMES
            .iter()
            .all(|e| extensions.contains(&e.to_str().map(|s| s.to_string()).unwrap()))
        {
            if conf::DEVICE_EXTENSION_NAMES.contains(&ash::khr::swapchain::NAME)
                && self.present_modes.is_empty()
                || self.formats.is_empty()
            {
                return false;
            }
        }

        true
    }
}
