use std::{any::Any, collections::HashMap};

use crate::{component::Component, entity::Entity};

pub trait ComponentStorage: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> ComponentStorage for HashMap<Entity, T>
where
    T: Component + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
