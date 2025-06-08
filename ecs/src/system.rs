use crate::{resource::ResourceStorage, storage::ComponentsStorage};

pub trait System {
    fn run(&self, components: &ComponentsStorage, resources: &ResourceStorage);
}
impl<F> System for F
where
    F: Fn(&ComponentsStorage, &ResourceStorage),
{
    fn run(&self, components: &ComponentsStorage, resources: &ResourceStorage) {
        self(components, resources);
    }
}

pub trait SystemMut {
    fn run(&mut self, components: &mut ComponentsStorage, resources: &mut ResourceStorage);
}

impl<F> SystemMut for F
where
    F: FnMut(&mut ComponentsStorage, &mut ResourceStorage),
{
    fn run(&mut self, components: &mut ComponentsStorage, resources: &mut ResourceStorage) {
        self(components, resources)
    }
}

// utils
pub fn system<F>(f: F) -> Box<dyn System>
where
    F: System + 'static,
{
    Box::new(f)
}

pub fn system_mut<F>(f: F) -> Box<dyn SystemMut>
where
    F: SystemMut + 'static,
{
    Box::new(f)
}
