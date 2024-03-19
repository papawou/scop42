use crate::{QueueFamilies, SurfaceSupport};
use ash::vk;

pub struct SwapchainScop {
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    pub chain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

pub fn create_swapchain(
    swapchain_loader: &ash::extensions::khr::Swapchain,
    device: &ash::Device,
    physical_size: (u32, u32),
    surface_support: &SurfaceSupport,
    surface: vk::SurfaceKHR,
    queue_families: &QueueFamilies,
) -> anyhow::Result<SwapchainScop> {
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
    swapchain_create_info = if queue_families.graphics != queue_families.present {
        swapchain_create_info
            .image_sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&swapchain_queue_families)
    } else {
        swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    };

    swapchain_create_info = swapchain_create_info
        .pre_transform(surface_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(swap_present_mode)
        .clipped(true);

    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
    let swapchain_images: Vec<vk::Image> =
        unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
    let swapchain_images_view = swapchain_images
        .iter()
        .map(|&e| {
            let image_view_info = vk::ImageViewCreateInfo::builder()
                .image(e)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swap_surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1)
                        .build(),
                );
            unsafe { device.create_image_view(&image_view_info, None).unwrap() }
        })
        .collect::<Vec<_>>();

    Ok(SwapchainScop {
        extent: swap_extent,
        surface_format: swap_surface_format,
        chain: swapchain,
        images: swapchain_images,
        image_views: swapchain_images_view,
    })
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
