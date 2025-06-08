use traits::ComponentStorage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{component::Component, entity::Entity};

pub mod traits;

pub type Storage<T> = HashMap<Entity, T>;
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

pub struct ComponentsStorage(HashMap<TypeId, Box<dyn ComponentStorage>>);
impl ComponentsStorage {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    // Internal storage
    pub fn get_component_storage<T: Component>(&self) -> Option<&Storage<T>> {
        let type_id = TypeId::of::<T>();
        self.0.get(&type_id)?.as_ref().as_any().downcast_ref()
    }
    pub fn get_component_storage_mut<T: Component>(&mut self) -> Option<&mut Storage<T>> {
        let type_id = TypeId::of::<T>();
        self.0
            .get_mut(&type_id)?
            .as_mut()
            .as_any_mut()
            .downcast_mut()
    }

    // Component function
    pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();

        self.0
            .entry(type_id)
            .or_insert_with(|| Box::new(Storage::<T>::new()));

        let storage = self.get_component_storage_mut::<T>().unwrap();
        storage.insert(entity.clone(), component);
    }
    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        self.get_component_storage::<T>()?.get(entity)
    }
    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        self.get_component_storage_mut::<T>()?.get_mut(entity)
    }
    pub fn remove_component<T: Component>(&mut self, entity: &Entity) -> Option<T> {
        self.get_component_storage_mut::<T>()?.remove(entity)
    }
}
