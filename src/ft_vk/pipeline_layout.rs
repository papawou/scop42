use ash::vk;

pub struct PipelineLayout<T = ()> {
    pub layout: vk::PipelineLayout,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> PipelineLayout<T> {
    pub fn as_ref_vk(&self) -> &vk::PipelineLayout {
        &self.layout
    }

    pub fn as_vk(&self) -> vk::PipelineLayout {
        self.layout.clone()
    }
}
