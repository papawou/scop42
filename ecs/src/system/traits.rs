use crate::{
    query::{
        traits::{Fetch, FetchMut},
        Query, QueryMut,
    },
    storage::ComponentsStorage,
};

// System
pub trait System {
    fn run(&self, components: &ComponentsStorage);
}

pub trait IntoSystem<Q>
where
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, world: &ComponentsStorage);
}

impl<F, Q> IntoSystem<Q> for F
where
    F: for<'a> Fn(Query<'a, Q>),
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, world: &ComponentsStorage) {
        let query = Query::<Q>::new(world);
        (self)(query);
    }
}

// SystemMut
pub trait SystemMut {
    fn run(&mut self, components: &mut ComponentsStorage);
}

pub trait IntoSystemMut<Q>
where
    for<'a> Q: FetchMut<'a>,
{
    fn run(&mut self, world: &mut ComponentsStorage);
}

impl<F, Q> IntoSystemMut<Q> for F
where
    F: for<'a> FnMut(QueryMut<'a, Q>),
    for<'a> Q: FetchMut<'a>,
{
    fn run(&mut self, world: &mut ComponentsStorage) {
        let query = QueryMut::<Q>::new(world);
        (self)(query);
    }
}
