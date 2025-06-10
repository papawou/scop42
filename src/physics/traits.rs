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
}

pub trait IntegrateFn {
    fn integrate(&mut self, dt: Duration);
}

impl IntegrateFn for dyn Body {
    fn integrate(&mut self, dt: Duration) {
        let mut body = self;
        let dt = dt.as_secs_f32();
        let velocity = body.velocity();
        let acceleration = body.acceleration();
        let new_velocity = velocity + acceleration * dt;
        body.set_velocity(new_velocity);
        body.set_position(body.position() + new_velocity * dt);
    }
}
