use ecs::{component::Component, macros::Component};
use glam::Vec3;

#[derive(Component)]
pub struct Position(pub Vec3);
