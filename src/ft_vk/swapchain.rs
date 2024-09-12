use std::any::Any;

use ash::vk;
use vk_mem::Alloc;

use super::{
    allocated_image::AllocatedImage, queue_famillies::QueueFamilies,
    surface_support::SurfaceSupport,
};

pub struct Swapchain {
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    pub chain: vk::SwapchainKHR,
    pub min_image_count: u32,

    // RENDERING
    pub image_views: Vec<vk::ImageView>,

    // DEPTH
    pub depth_images: Vec<AllocatedImage>,
}

impl Swapchain {
    pub fn new(
        swapchain_loader: &ash::khr::swapchain::Device,
        device: &ash::Device,
        allocator: &vk_mem::Allocator,
        physical_size: (u32, u32),
        surface_support: &SurfaceSupport,
        surface: vk::SurfaceKHR,
        queue_families: &QueueFamilies,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Self {
        let surface_format = choose_surface_format(&surface_support.formats);
        let extent = choose_extent(&surface_support.capabilities, physical_size);

        // Swapchain
        let present_mode = choose_present_mode(&surface_support.present_modes);
        let queue_family_indices = [queue_families.graphics, queue_families.present];

        // Determine optimal image count
        let min_image_count = {
            let mut desired_image_count = surface_support.capabilities.min_image_count + 1;
            // For `MAILBOX` present mode, we want at least 3 images for triple-buffering
            if present_mode == vk::PresentModeKHR::MAILBOX {
                desired_image_count = 3;
            }
            // Surface has a limit
            if surface_support.capabilities.max_image_count > 0 {
                desired_image_count = desired_image_count
                    .min(surface_support.capabilities.max_image_count) //minimum is max
                    .max(surface_support.capabilities.min_image_count); //maximum is min
            }
            desired_image_count
        };

        let swapchain = {
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

        // Images
        let images: Vec<vk::Image> =
            unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let mut image_views: Vec<vk::ImageView> = vec![];
        let mut depth_images: Vec<AllocatedImage> = vec![];

        for &image in &images {
            let image_view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .format(surface_format.format)
                .view_type(vk::ImageViewType::TYPE_2D)
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
            let image_view = unsafe { device.create_image_view(&image_view_info, None).unwrap() };

            let depth_image = create_depth_image(
                device,
                allocator,
                vk::Extent3D {
                    depth: 1,
                    ..extent.into()
                },
            );

            image_views.push(image_view);
            depth_images.push(depth_image);
        }

        Self {
            chain: swapchain,
            extent,
            surface_format,
            image_views,
            min_image_count,
            depth_images,
        }
    }

    //self dropped because shoud not be used more
    pub fn destroy(
        self,
        device: &ash::Device,
        allocator: &vk_mem::Allocator,
        swapchain_loader: &ash::khr::swapchain::Device,
    ) {
        for &image_view in &self.image_views {
            unsafe { device.destroy_image_view(image_view, None) };
        }
        for depth_image in self.depth_images {
            unsafe { device.destroy_image_view(depth_image.image_view, None) };

            let mut depth_image = depth_image;
            unsafe { allocator.destroy_image(depth_image.image, &mut depth_image.allocation) }
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
        for (&image_view, depth_image) in self.image_views.iter().zip(self.depth_images.iter()) {
            let attachments = [image_view, depth_image.image_view];
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

// MAILBOX ?? FIFO
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

fn create_depth_image(
    device: &ash::Device,
    allocator: &vk_mem::Allocator,
    extent: vk::Extent3D,
) -> AllocatedImage {
    let format = vk::Format::D32_SFLOAT;

    let (image, allocation) = {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .extent(extent)
            .format(format)
            .mip_levels(1)
            .array_layers(1);
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Default::default()
        };
        unsafe {
            allocator
                .create_image(&image_create_info, &allocation_create_info)
                .unwrap()
        }
    };

    let image_view = {
        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1),
            );
        unsafe {
            device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        }
    };

    AllocatedImage {
        image,
        image_view,
        extent,
        allocation,
        format,
    }
}
