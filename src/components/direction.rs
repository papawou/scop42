use ecs::{component::Component, macros::Component};
use glam::Quat;

#[derive(Component, Clone, Copy)]
pub struct Direction(pub Quat);
