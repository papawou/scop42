use ecs::{component::Component, macros::Component};
use glam::Quat;

#[derive(Component, Debug)]
pub struct Rotation(pub Quat);
