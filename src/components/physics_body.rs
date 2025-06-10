use std::rc::Rc;

use ecs::{component::Component, macros::Component, world::World, Entity};
use glam::Vec3;

use crate::physics::traits::IntegrateFn;

#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub integrate:
        Option<Rc<dyn for<'a> Fn(&'a Entity, &'a mut World) -> Box<dyn IntegrateFn + 'a>>>,
}

impl PhysicsBody {
    pub fn integrate<'a>(
        &self,
        entity: &'a Entity,
        world: &'a mut World,
    ) -> Option<Box<dyn IntegrateFn + 'a>> {
        match &self.integrate {
            Some(func) => Some(func(entity, world)),
            None => None,
        }
    }
}
