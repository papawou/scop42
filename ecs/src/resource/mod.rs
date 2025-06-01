use std::{any::TypeId, collections::HashMap};

pub mod traits;

use traits::Resource;

pub struct ResourceStorage(HashMap<TypeId, Box<dyn Resource>>);

impl ResourceStorage {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_component_storage<T: Resource>(&self) -> Option<&Storage<T>> {
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
