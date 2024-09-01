use ash::vk;

use super::{queue_famillies::QueueFamilies, surface_support::SurfaceSupport};

pub struct Swapchain {
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    pub chain: vk::SwapchainKHR,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn new(
        swapchain_loader: &ash::khr::swapchain::Device,
        device: &ash::Device,
        physical_size: (u32, u32),
        surface_support: &SurfaceSupport,
        surface: vk::SurfaceKHR,
        queue_families: &QueueFamilies,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Self {
        let surface_format = choose_surface_format(&surface_support.formats);
        let extent = choose_extent(&surface_support.capabilities, physical_size);

        // Swapchain
        let swapchain = {
            let min_image_count = (surface_support.capabilities.min_image_count + 1).min(
                if surface_support.capabilities.max_image_count > 0 {
                    surface_support.capabilities.max_image_count
                } else {
                    u32::MAX
                },
            );
            let present_mode = choose_present_mode(&surface_support.present_modes);
            let queue_family_indices = [queue_families.graphics, queue_families.present];

            let mut create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(min_image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .pre_transform(surface_support.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE);

            create_info = if queue_families.graphics != queue_families.present {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&queue_family_indices)
            } else {
                create_info
            };

            create_info = if let Some(old_swap) = old_swapchain {
                create_info.old_swapchain(old_swap)
            } else {
                create_info
            };

            unsafe {
                swapchain_loader
                    .create_swapchain(&create_info, None)
                    .unwrap()
            }
        };

        // ImageView
        let images: Vec<vk::Image> =
            unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let image_views = images
            .iter()
            .map(|&e| {
                let image_view_info = vk::ImageViewCreateInfo::default()
                    .image(e)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .layer_count(1),
                    );
                unsafe { device.create_image_view(&image_view_info, None).unwrap() }
            })
            .collect::<Vec<vk::ImageView>>();

        Self {
            extent,
            surface_format,
            chain: swapchain,
            image_views,
        }
    }

    //self dropped because shoud not be used more
    pub fn destroy(self, device: &ash::Device, swapchain_loader: &ash::khr::swapchain::Device) {
        for &image_view in &self.image_views {
            unsafe { device.destroy_image_view(image_view, None) };
        }
        unsafe { swapchain_loader.destroy_swapchain(self.chain, None) };
    }

    pub fn get_framebuffers(
        &self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers = Vec::with_capacity(self.image_views.len());

        //When rendering, the swapchain will give us the index of the image to render into, so we will use the framebuffer of the same index.
        for &image_view in &self.image_views {
            let attachments = [image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let framebuffer =
                unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };
            framebuffers.push(framebuffer);
        }

        framebuffers
    }
}

fn choose_surface_format(formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .find(|p| {
            p.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                && p.format == vk::Format::B8G8R8A8_SRGB
        })
        .cloned()
        .unwrap_or_else(|| formats.get(0).cloned().unwrap())
}

fn choose_present_mode(presents: &Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    if presents.contains(&vk::PresentModeKHR::MAILBOX) {
        return vk::PresentModeKHR::MAILBOX;
    }

    vk::PresentModeKHR::FIFO
}

fn choose_extent(
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
