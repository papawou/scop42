use ecs::{component::Component, macros::Component};
use glam::Quat;

#[derive(Component, Clone, Copy)]
pub struct Rotation(pub Quat);
