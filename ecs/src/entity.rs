#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Entity {
    Camera,
    Origin,
    Custom(String),
    Id(usize),
}
