use std::{rc::Rc, time::Duration};

use ecs::{component::Component, macros::Component, world::World, Entity};
use glam::Vec3;

use crate::{
    components::{position, rotation},
    physics::{
        compute_angular_velocity, compute_position, compute_rotation, compute_velocity,
        traits::IntegrateFn,
    },
};

#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec3,
    pub acceleration: Vec3,

    pub angular_velocity: Vec3,
    pub angular_acceleration: Vec3,

    pub integrate:
        Option<Box<dyn for<'a> Fn(&'a Entity, &'a mut World) -> Box<dyn IntegrateFn + 'a>>>,
}

impl PhysicsBody {
    pub fn integrate<'a>(
        &self,
        entity: &'a Entity,
        world: &'a mut World,
    ) -> Box<dyn IntegrateFn + 'a> {
        match &self.integrate {
            Some(func) => func(entity, world),
            None => integrate(entity, world),
        }
    }
}

pub fn integrate<'a>(entity: &'a Entity, world: &'a mut World) -> Box<dyn IntegrateFn + 'a> {
    Box::new(|dt: Duration| {
        let mut physics_body = unsafe {
            world
                .as_unsafe_mut()
                .components
                .get_component_mut::<PhysicsBody>(entity)
                .unwrap()
        };

        match unsafe {
            world
                .as_unsafe_mut()
                .components
                .get_component_mut::<position::Position>(entity)
        } {
            Some(position) => {
                physics_body.velocity =
                    compute_velocity(physics_body.velocity, physics_body.acceleration, dt);
                position.0 = compute_position(position.0, physics_body.velocity, dt);
            }
            None => {}
        };

        match unsafe {
            world
                .as_unsafe_mut()
                .components
                .get_component_mut::<rotation::Rotation>(entity)
        } {
            Some(rotation) => {
                physics_body.angular_velocity = compute_angular_velocity(
                    physics_body.angular_velocity,
                    physics_body.angular_acceleration,
                    dt,
                );
                rotation.0 = compute_rotation(rotation.0, physics_body.angular_velocity, dt);
            }
            None => {}
        };
    })
}
