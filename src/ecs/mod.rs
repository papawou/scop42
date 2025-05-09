use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
};

pub mod entity;
pub use entity::Entity;
use query::Query;
pub mod query;
pub mod system;
pub struct World {
    next_entity: usize,
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any>>>,
    systems: Vec<dyn Fn(&Query<dyn Any>)>,
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

    pub fn add_component<T: Any>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();

        let mut entry = self.components.entry(type_id).or_insert_with(HashMap::new);

        entry.insert(*entity, Box::new(component));
    }

    pub fn remove_component<T: Any>(&mut self, entity: &Entity) {
        let type_id = TypeId::of::<T>();

        self.components.entry(type_id).and_modify(|hash| {
            hash.remove(entity);
        });
    }

    pub fn get_component<T: Any>(&mut self, entity: &Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.components.get_mut(&type_id).and_then(|hash| {
            hash.get(entity)
                .map(|component| component.as_ref().downcast_ref::<T>())
                .flatten()
        })
    }

    pub fn get_mut_component<T: Any>(&mut self, entity: &Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        self.components.get_mut(&type_id).and_then(|hash| {
            hash.get_mut(entity)
                .map(|component| component.as_mut().downcast_mut::<T>())
                .flatten()
        })
    }

    pub fn run_system<T>(f: fn(query: Query<T>)) {
        let query= Query::<T>::new();

        // does shitty things

        fn(query)
    }

    pub fn add_system<T: Any>(&mut self, system: fn(query: Query<T>)) {
        let query = Query::<T>::new();


        // extract T
        system(query);
    }
}
