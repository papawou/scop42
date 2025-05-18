use std::marker::PhantomData;

use crate::{component::Component, entity::Entity, storage::ComponentsStorage};

pub struct Query<'w, Q>
where
    Q: Fetch<'w>,
{
    world: &'w ComponentsStorage,
    _marker: PhantomData<Q>,
}
impl<'w, Q> Query<'w, Q>
where
    Q: Fetch<'w>,
{
    pub fn new(world: &'w ComponentsStorage) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

impl<'w, Q> IntoIterator for Query<'w, Q>
where
    Q: Fetch<'w>,
{
    type Item = Q::Item;
    type IntoIter = Q::Iter;

    fn into_iter(self) -> Self::IntoIter {
        Q::fetch(self.world)
    }
}

// Fetch trait
pub trait Fetch<'w> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    fn fetch(world: &'w ComponentsStorage) -> Self::Iter;
}

impl<'w, T: Component> Fetch<'w> for &'w T {
    type Item = (&'w Entity, &'w T);
    type Iter = std::collections::hash_map::Iter<'w, Entity, T>;

    fn fetch(world: &'w ComponentsStorage) -> Self::Iter {
        world
            .get_component_storage::<T>()
            .expect("Component not found")
            .iter()
    }
}

pub trait FetchMut<'w> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    fn fetch(components: &'w mut ComponentsStorage) -> Self::Iter;
}

impl<'w, T: Component> FetchMut<'w> for &'w mut T {
    type Item = (&'w Entity, &'w mut T);
    type Iter = std::collections::hash_map::IterMut<'w, Entity, T>;

    fn fetch(components: &'w mut ComponentsStorage) -> Self::Iter {
        components
            .get_component_storage_mut::<T>()
            .expect("Component not found")
            .iter_mut()
    }
}

impl<'w, A, B> Fetch<'w> for (&'w A, &'w B)
where
    A: Component,
    B: Component,
{
    type Item = (Entity, (&'w A, &'w B));
    type Iter = Zip2<'w, A, B>;

    fn fetch(world: &'w ComponentsStorage) -> Self::Iter {
        let storage_a = world.get_component_storage::<A>().expect("A not found");
        let storage_b = world.get_component_storage::<B>().expect("B not found");

        // closure to filter and map entities with both components
        // let filter_map_fn = move |(entity, comp_a): (&Entity, &A)| {
        //     if let Some(comp_b) = storage_b.get(entity) {
        //         Some((entity, (comp_a, comp_b)))
        //     } else {
        //         None
        //     }
        // };
        // storage_a.iter().filter_map(filter_map_fn)

        Zip2 {
            iter_a: storage_a.iter(),
            storage_b,
        }
    }
}

pub struct Zip2<'w, A, B> {
    iter_a: std::collections::hash_map::Iter<'w, Entity, A>,
    storage_b: &'w std::collections::HashMap<Entity, B>,
}

impl<'w, A, B> Iterator for Zip2<'w, A, B> {
    type Item = (Entity, (&'w A, &'w B));

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((entity, comp_a)) = self.iter_a.next() {
            if let Some(comp_b) = self.storage_b.get(entity) {
                return Some((*entity, (comp_a, comp_b)));
            }
        }
        None
    }
}
