use ash::vk;

pub trait IntoOwned {
    type Owned;
    fn into_owned(&self) -> Self::Owned;
}
