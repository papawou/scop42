use crate::{component::Component, entity::Entity, storage::ComponentsStorage};

// Fetch
pub trait Fetch<'a> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    fn fetch(world: &'a ComponentsStorage) -> Self::Iter;
}
impl<'a, T: Component> Fetch<'a> for &T {
    type Item = (&'a Entity, &'a T);
    type Iter = std::collections::hash_map::Iter<'a, Entity, T>;

    fn fetch(world: &'a ComponentsStorage) -> Self::Iter {
        world
            .get_component_storage::<T>()
            .expect("Component not found")
            .iter()
    }
}

// FetchMut
pub trait FetchMut<'a> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    fn fetch(components: &'a mut ComponentsStorage) -> Self::Iter;
}
impl<'a, T: Component> FetchMut<'a> for &mut T {
    type Item = (&'a Entity, &'a mut T);
    type Iter = std::collections::hash_map::IterMut<'a, Entity, T>;

    fn fetch(components: &'a mut ComponentsStorage) -> Self::Iter {
        components
            .get_component_storage_mut::<T>()
            .expect("Component not found")
            .iter_mut()
    }
}
