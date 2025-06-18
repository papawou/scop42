use std::{
    ops::{Add, Sub},
    time::Duration,
};

mod conf;
pub mod traits;

use ecs::{entity, storage::ComponentsStorage, world::World, Entity};
use glam::{Quat, Vec3};

use crate::{
    components::{physics_body, position, PhysicsBody, Position},
    physics::{self, traits::IntegrateFn},
};

pub struct Engine {
    pub frame_time_acc: Duration,
    pub last_update: std::time::Instant,
}

impl Engine {
    pub fn tick(&mut self, mut bodies: Vec<Box<dyn IntegrateFn>>) {
        let frame_time = self.last_update.elapsed().min(conf::MAX_FRAME_TIME);
        self.last_update = std::time::Instant::now();
        self.frame_time_acc = self.frame_time_acc.add(frame_time);

        while (self.frame_time_acc >= conf::PHYSICS_FPS) {
            for body in bodies.iter_mut() {
                body.integrate(conf::PHYSICS_FPS);
            }
            self.frame_time_acc = self.frame_time_acc.sub(conf::PHYSICS_FPS);
        }

        //todo!("Alpha rendering logic here");
        // let alpha = self.frame_time_acc.div_duration_f64(conf::PHYSICS_FPS);
    }
}

pub fn compute_velocity(velocity: Vec3, acceleration: Vec3, duration: Duration) -> Vec3 {
    velocity + acceleration * duration.as_secs_f32()
}

pub fn compute_position(position: Vec3, velocity: Vec3, duration: Duration) -> Vec3 {
    position + velocity * duration.as_secs_f32()
}

pub fn compute_angular_velocity(
    angular_velocity: Vec3,
    angular_acceleration: Vec3,
    duration: Duration,
) -> Vec3 {
    angular_velocity + angular_acceleration * duration.as_secs_f32()
}

pub fn compute_rotation(rotation: Quat, angular_velocity: Vec3, duration: Duration) -> Quat {
    (Quat::from_euler(
        glam::EulerRot::XYZ,
        angular_velocity.x * duration.as_secs_f32(),
        angular_velocity.y * duration.as_secs_f32(),
        angular_velocity.z * duration.as_secs_f32(),
    )) * rotation
}
