use ecs::{component::Component, entity::Entity, macros::Component};
use glam::Quat;

#[derive(Component)]
pub struct Camera {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub far: f32,
    pub near: f32,
    pub mode: Mode,
}

pub enum Mode {
    Free,
    Follow {
        target: Entity,
        yaw: f32,
        pitch: f32,
    },
}

impl Mode {
    input_con
}
