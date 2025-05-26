use traits::ComponentStorage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{component::Component, entity::Entity};
///
pub mod traits;

///

pub type Storage<T> = HashMap<Entity, T>; // T: Component

pub struct ComponentsStorage {
    pub components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl ComponentsStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn get_component_storage<T: Component>(&self) -> Option<&Storage<T>> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.as_any().downcast_ref()
    }
    pub fn get_component_storage_mut<T: Component>(&mut self) -> Option<&mut Storage<T>> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .as_any_mut()
            .downcast_mut()
    }

    pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();

        self.components
            .entry(type_id)
            .or_insert_with(|| Box::new(Storage::<T>::new()));

        let storage = self.get_component_storage_mut::<T>().unwrap();
        storage.insert(*entity, component);
    }

    pub fn get_component<T: Component>(&mut self, entity: &Entity) -> Option<&T> {
        self.get_component_storage::<T>()?.get(entity)
    }
    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        self.get_component_storage_mut::<T>()?.get_mut(entity)
    }
}
impl<T> ComponentStorage for Storage<T>
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
