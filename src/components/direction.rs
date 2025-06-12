use ecs::{component::Component, macros::Component};
use glam::Vec3;

#[derive(Component, Clone, Copy)]
pub struct Direction(pub Vec3);
