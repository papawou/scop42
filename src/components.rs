use ecs::{component::Component, entity::Entity, macros::Component};
use glam::Vec3;

#[derive(Component)]
pub struct Position(pub Vec3);

#[derive(Component)]
pub struct Camera {
    aspect_ratio: f32,
    target: Option<Entity>,
}

pub struct Mesh {
    
}