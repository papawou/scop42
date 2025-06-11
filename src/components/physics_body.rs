use std::{rc::Rc, time::Duration};

use ecs::{component::Component, macros::Component, world::World, Entity};
use glam::Vec3;

use crate::{
    components::position,
    physics::{compute_position, compute_velocity, traits::IntegrateFn},
};

#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec3,
    pub acceleration: Vec3,
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

fn integrate<'a>(entity: &'a Entity, world: &'a mut World) -> Box<dyn IntegrateFn + 'a> {
    Box::new(|dt: Duration| {
        let world = world as *mut World;
        let mut physics_body = unsafe {
            (*world)
                .components
                .get_component_mut::<PhysicsBody>(entity)
                .unwrap()
        };
        let mut position = unsafe {
            (*world)
                .components
                .get_component_mut::<position::Position>(entity)
                .unwrap()
        };
        physics_body.velocity =
            compute_velocity(physics_body.velocity, physics_body.acceleration, dt);
        position.0 = compute_position(position.0, physics_body.velocity, dt);
    })
}
