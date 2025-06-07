use ecs::{component::Component, macros::Component};
use glam::Vec3;

#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}
