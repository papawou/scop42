use crate::SurfaceSupport;
use ash::vk;

pub fn create_swapchain(
    surface_support: &SurfaceSupport,
    surface: vk::SurfaceKHR,
    instance: &ash::Instance,
    logical_device: &ash::Device,
    queue_graphics_idx: u32,
) -> anyhow::Result<(ash::extensions::khr::Swapchain, vk::SwapchainKHR)> {
    let queue_families = [queue_graphics_idx];

    let swap_surface_format = choose_swap_surface_format(&surface_support.formats);

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(
            3.max(surface_support.capabilities.min_image_count)
                .min(surface_support.capabilities.max_image_count),
        )
        .image_format(swap_surface_format.format)
        .image_color_space(swap_surface_format.color_space)
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

fn choose_swap_surface_format(formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .find(|p| {
            p.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && p.format == vk::Format::B8G8R8A8_SRGB
        })
        .cloned()
        .unwrap_or_else(|| formats.get(0).cloned().unwrap())
}

fn choose_swap_present_mode(prensts: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
	
}
