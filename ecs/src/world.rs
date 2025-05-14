use std::{any::TypeId, collections::HashMap};

use crate::{component::Component, entity::Entity, query::Query, system::SystemFn};

pub struct World {
    next_entity: usize,
    pub components: HashMap<TypeId, HashMap<Entity, Box<dyn Component>>>,
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

        let entry = self.components.entry(type_id).or_insert_with(HashMap::new);

        entry.insert(*entity, Box::new(component));
    }

    pub fn remove_component<T: Component>(&mut self, entity: &Entity) {
        let type_id = TypeId::of::<T>();

        self.components.entry(type_id).and_modify(|hash| {
            hash.remove(entity);
        });
    }

    pub fn get_component<T: Component>(&mut self, entity: &Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.components.get_mut(&type_id).and_then(|hash| {
            hash.get(entity)
                .map(|component| component.as_ref().as_any().downcast_ref::<T>())
                .flatten()
        })
    }

    pub fn get_component_storage<T: Component>(
        &self,
    ) -> Option<&HashMap<Entity, Box<dyn Component>>> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)
    }

    pub fn get_component_storage_mut<T: Component>(
        &mut self,
    ) -> Option<&mut HashMap<Entity, Box<dyn Component>>> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        self.components.get_mut(&type_id).and_then(|hash| {
            hash.get_mut(entity)
                .map(|component| component.as_mut().as_any_mut().downcast_mut::<T>())
                .flatten()
        })
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
