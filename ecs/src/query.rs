use std::marker::PhantomData;

use crate::{component::Component, entity::Entity, storage::ComponentsStorage};

pub struct Query<'a, Q>
where
    Q: Fetch<'a>,
{
    world: &'a ComponentsStorage,
    _marker: PhantomData<Q>,
}
impl<'a, Q> Query<'a, Q>
where
    Q: Fetch<'a>,
{
    pub fn new(world: &'a ComponentsStorage) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

impl<'a, Q> IntoIterator for Query<'a, Q>
where
    Q: Fetch<'a>,
{
    type Item = Q::Item;
    type IntoIter = Q::Iter;

    fn into_iter(self) -> Self::IntoIter {
        Q::fetch(self.world)
    }
}

// Fetch trait
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

// pub trait FetchMut<'a> {
//     type Item;
//     type Iter: Iterator<Item = Self::Item>;
//     fn fetch(components: &'a mut ComponentsStorage) -> Self::Iter;
// }

// impl<'a, T: Component> FetchMut<'a> for &mut T {
//     type Item = (&'a Entity, &'a mut T);
//     type Iter = std::collections::hash_map::IterMut<'a, Entity, T>;

//     fn fetch(components: &'a mut ComponentsStorage) -> Self::Iter {
//         components
//             .get_component_storage_mut::<T>()
//             .expect("Component not found")
//             .iter_mut()
//     }
// }

// impl<'a, A, B> Fetch<'a> for (&A, &B)
// where
//     A: Component,
//     B: Component,
// {
//     type Item = (Entity, (&'a A, &'a B));
//     type Iter = Zip2<'a, A, B>;

//     fn fetch(world: &'a ComponentsStorage) -> Self::Iter {
//         let storage_a = world.get_component_storage::<A>().expect("A not found");
//         let storage_b = world.get_component_storage::<B>().expect("B not found");

//         // closure to filter and map entities with both components
//         // let filter_map_fn = move |(entity, comp_a): (&Entity, &A)| {
//         //     if let Some(comp_b) = storage_b.get(entity) {
//         //         Some((entity, (comp_a, comp_b)))
//         //     } else {
//         //         None
//         //     }
//         // };
//         // storage_a.iter().filter_map(filter_map_fn)

//         Zip2 {
//             iter_a: storage_a.iter(),
//             storage_b,
//         }
//     }
// }

// pub struct Zip2<'a, A, B> {
//     iter_a: std::collections::hash_map::Iter<'a, Entity, A>,
//     storage_b: &'a std::collections::HashMap<Entity, B>,
// }

// impl<'a, A, B> Iterator for Zip2<'a, A, B> {
//     type Item = (Entity, (&'a A, &'a B));

//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some((entity, comp_a)) = self.iter_a.next() {
//             if let Some(comp_b) = self.storage_b.get(entity) {
//                 return Some((*entity, (comp_a, comp_b)));
//             }
//         }
//         None
//     }
// }
