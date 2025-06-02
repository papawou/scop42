use std::marker::PhantomData;
use traits::{Fetch, FetchMut};

use crate::storage::ComponentsStorage;

pub mod traits;

// Query
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

// QueryMut
pub struct QueryMut<'a, Q>
where
    Q: FetchMut<'a>,
{
    world: &'a mut ComponentsStorage,
    _marker: PhantomData<Q>,
}
impl<'a, Q> QueryMut<'a, Q>
where
    Q: FetchMut<'a>,
{
    pub fn new(world: &'a mut ComponentsStorage) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}
impl<'a, Q> IntoIterator for QueryMut<'a, Q>
where
    Q: FetchMut<'a>,
{
    type Item = Q::Item;
    type IntoIter = Q::Iter;

    fn into_iter(self) -> Self::IntoIter {
        Q::fetch(self.world)
    }
}
