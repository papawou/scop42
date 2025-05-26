use crate::{
    query::{Fetch, FetchMut, Query, QueryMut},
    storage::ComponentsStorage,
};
use std::marker::PhantomData;

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

pub struct FnSystem<Q, F>(pub F, pub PhantomData<Q>);

impl<F, Q> System for FnSystem<Q, F>
where
    F: IntoSystem<Q>,
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, components: &ComponentsStorage) {
        self.0.run(components);
    }
}

// /// Factory function to wrap a function or closure into a Boxed System
pub fn system<Q, F>(f: F) -> Box<dyn System>
where
    F: IntoSystem<Q> + 'static,
    for<'a> Q: Fetch<'a> + 'static,
{
    Box::new(FnSystem(f, PhantomData))
}

// MUT
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

pub struct FnSystemMut<Q, F>(pub F, pub PhantomData<Q>);

impl<Q, F> SystemMut for FnSystemMut<Q, F>
where
    F: IntoSystemMut<Q>,
    for<'a> Q: FetchMut<'a>,
{
    fn run(&mut self, world: &mut ComponentsStorage) {
        self.0.run(world);
    }
}

pub fn system_mut<Q, F>(f: F) -> Box<dyn SystemMut>
where
    F: IntoSystemMut<Q> + 'static,
    for<'a> Q: FetchMut<'a> + 'static,
{
    Box::new(FnSystemMut(f, PhantomData))
}
