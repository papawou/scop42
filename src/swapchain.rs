use crate::{QueueFamilies, SurfaceSupport};
use ash::vk;

pub fn create_swapchain(
    physical_size: (u32, u32),
    surface_support: &SurfaceSupport,
    surface: vk::SurfaceKHR,
    instance: &ash::Instance,
    logical_device: &ash::Device,
    queue_families: &QueueFamilies,
) -> anyhow::Result<(ash::extensions::khr::Swapchain, vk::SwapchainKHR)> {
    let swap_surface_format = choose_swap_surface_format(&surface_support.formats);
    let swap_present_mode = choose_swap_present_mode(&surface_support.present_modes);
    let swap_extent = choose_swap_extent(
        &surface_support.capabilities,
        (physical_size.0, physical_size.1),
    );

    let image_count = (surface_support.capabilities.min_image_count + 1).min(
        if surface_support.capabilities.max_image_count > 0 {
            surface_support.capabilities.max_image_count
        } else {
            u32::MAX
        },
    );

    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(swap_surface_format.format)
        .image_color_space(swap_surface_format.color_space)
        .image_extent(swap_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let swapchain_queue_families = [queue_families.graphics, queue_families.present];
    if queue_families.graphics != queue_families.present {
        swapchain_create_info = swapchain_create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&swapchain_queue_families)
    } else {
        swapchain_create_info = swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    }

    swapchain_create_info = swapchain_create_info
        .pre_transform(surface_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(swap_present_mode)
        .clipped(true);

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

fn choose_swap_present_mode(presents: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    if presents.contains(&vk::PresentModeKHR::MAILBOX) {
        return vk::PresentModeKHR::MAILBOX;
    }

    vk::PresentModeKHR::FIFO
}

fn choose_swap_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    (width, height): (u32, u32),
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    vk::Extent2D {
        width: width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}
