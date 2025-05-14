use std::marker::PhantomData;

use crate::{entity::Entity, world::World};

pub struct Query<'w, Q>
where
    Q: Fetch<'w>,
{
    world: &'w World,
    _marker: PhantomData<Q>,
}
impl<'w, Q> Query<'w, Q>
where
    Q: Fetch<'w>,
{
    pub fn new(world: &'w World) -> Self {
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
    fn fetch(world: &'w World) -> Self::Iter;
}
impl<'w, T> Fetch<'w> for &'w T {
    type Item = (&'w Entity, &'w T);
    type Iter = std::slice::Iter<'w, Self::Item>;
    fn fetch(world: &'w World) -> Self::Iter {
        todo!()
    }
}
impl<'w, T> Fetch<'w> for &'w mut T {
    type Item = (&'w Entity, &'w T);
    type Iter = std::slice::IterMut<'w, Self::Item>;
    fn fetch(world: &'w World) -> Self::Iter {
        todo!()
    }
}

impl<'w, A, B> Fetch<'w> for (A, B)
where
    A: Fetch<'w>,
    B: Fetch<'w>,
{
    type Item = (A::Item, B::Item);
    type Iter = std::slice::Iter<'w, Self::Item>;
    fn fetch(world: &'w World) -> Self::Iter {
        let a_iter = A::fetch(world);
        let b_iter = B::fetch(world);
    }
}
