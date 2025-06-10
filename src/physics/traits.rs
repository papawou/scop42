use std::time::Duration;

use glam::{Quat, Vec3};

pub trait Body {
    fn position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
    fn velocity(&self) -> Vec3;
    fn set_velocity(&mut self, velocity: Vec3);
    fn acceleration(&self) -> Vec3;
    fn set_acceleration(&mut self, acceleration: Vec3);
    fn rotation(&self) -> Quat;
    fn set_rotation(&mut self, rotation: Quat);

    fn integrate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();
        let new_velocity = self.velocity() + self.acceleration() * dt;
        self.set_velocity(new_velocity);
        let new_position = self.position() + new_velocity * dt;
        self.set_position(new_position);
    }
}
