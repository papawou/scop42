use ecs::{resource::ResourceStorage, storage::ComponentsStorage};

pub fn a_system(components: &ComponentsStorage, resources: &ResourceStorage) {}

pub fn a_mut_system(components: &mut ComponentsStorage, resources: &mut ResourceStorage) {}
