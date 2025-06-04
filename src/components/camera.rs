use ecs::{component::Component, entity::Entity, macros::Component};
use glam::Quat;

#[derive(Component)]
pub struct Camera {
    pub aspect_ratio: f32,
    pub look_at: Option<Entity>,
    pub fov: f32,
    pub far: f32,
    pub near: f32,
}
