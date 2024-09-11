use ash::vk;
use vk_mem::Alloc;

pub struct AllocatedImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub allocation: vk_mem::Allocation,
    pub extent: vk::Extent3D, //why imagecreateinfo is extent3d ?
    pub format: vk::Format,
}
