use ash::vk;

pub struct ShaderModule;

impl ShaderModule {
    pub fn create_from_file(device: &ash::Device, filename: &str) -> vk::ShaderModule {
        let mut shader_file = std::fs::File::open(filename).unwrap();
        let shader_code = ash::util::read_spv(&mut shader_file).unwrap();

        let createinfo = ash::vk::ShaderModuleCreateInfo::default().code(&shader_code);
        unsafe { device.create_shader_module(&createinfo, None).unwrap() }
    }
}
