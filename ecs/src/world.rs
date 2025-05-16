use std::{any::TypeId, collections::HashMap};

use crate::{
    component::Component, entity::Entity, query::Query, storage::ComponentStorage, system::SystemFn,
};

pub struct World {
    next_entity: usize,
    pub components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    systems: Vec<Box<dyn for<'a> SystemFn<'a>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: HashMap::new(),
            systems: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let e = Entity(self.next_entity);
        self.next_entity += 1;
        e
    }

    pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();

        self.components
            .entry(type_id)
            .or_insert_with(|| Box::new(HashMap::<Entity, T>::new()));

        let storage = self.get_component_storage_mut::<T>().unwrap();
        storage.insert(*entity, component);
    }

    pub fn remove_component<T: Component>(&mut self, entity: &Entity) {
        self.get_component_mut(entity).and_modify(|hash| {
            hash.remove(entity);
        });
    }

    pub fn get_component_storage<T: Component>(&self) -> Option<&HashMap<Entity, T>> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)?
            .as_any()
            .downcast_ref::<HashMap<Entity, T>>()
    }
    pub fn get_component_storage_mut<T: Component>(&mut self) -> Option<&mut HashMap<Entity, T>> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)?
            .as_any_mut()
            .downcast_mut::<HashMap<Entity, T>>()
    }

    pub fn get_component<T: Component>(&mut self, entity: &Entity) -> Option<&T> {
        self.get_component_storage::<T>()?.get(entity)
    }
    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        self.get_component_storage_mut::<T>()?.get_mut(entity)
    }

    pub fn add_system<T: Component>(&mut self, system: fn(query: Query<T>)) {
        self.systems.push(system);
    }

    pub fn run_system<T>(&self) {
        for system in self.systems.iter() {
            system(&self);
        }
    }
}
