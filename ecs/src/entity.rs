#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Entity(pub usize);

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum EntityTag {
    Camera,
    Custom(String),
}
