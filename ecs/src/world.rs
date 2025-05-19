use crate::{entity::Entity, storage::ComponentsStorage, system::System};

pub struct World {
    next_entity: usize,
    pub components: ComponentsStorage,
    systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: ComponentsStorage::new(),
            systems: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let e = Entity(self.next_entity);
        self.next_entity += 1;
        e
    }

    pub fn run_systems(&mut self) {
        for system in &self.systems {
            system.run(&mut self.components);
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }
}
