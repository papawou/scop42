use crate::{
    entity::Entity,
    resource::ResourceStorage,
    storage::ComponentsStorage,
    system::{System, SystemMut},
};

pub struct World {
    next_entity: usize,
    pub components: ComponentsStorage,
    pub resources: ResourceStorage,
    systems: Vec<Box<dyn System>>,
    systems_mut: Vec<Box<dyn SystemMut>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity: 0,
            components: ComponentsStorage::new(),
            systems: Vec::new(),
            systems_mut: Vec::new(),
            resources: ResourceStorage::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        let e = Entity(self.next_entity);
        self.next_entity += 1;
        e
    }

    pub fn run_systems(&mut self) {
        for system in &self.systems {
            system.run(&self.components, &self.resources);
        }

        for system in &mut self.systems_mut {
            system.run(&mut self.components, &mut self.resources);
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn add_system_mut(&mut self, system: Box<dyn SystemMut>) {
        self.systems_mut.push(system);
    }
}
