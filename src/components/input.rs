use std::rc::Rc;

use ecs::{component::Component, macros::Component, world::World, Entity};

#[derive(Component, Clone)]
pub struct Input(pub Rc<dyn Fn(&Entity, &mut World)>);

impl Input {
    pub fn apply(&self, entity: &Entity, world: &mut World) {
        (self.0)(entity, world)
    }
}
