use crate::{
    query::{Fetch, Query},
    storage::ComponentsStorage,
};

pub trait System {
    fn run(&self, components: &mut ComponentsStorage);
}

impl<Q> System for dyn Fn(Query<'_, Q>)
where
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::new(components);
        (self)(query)
    }
}

impl<Q> System for fn(Query<'_, Q>)
where
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::new(components);
        (self)(query);
    }
}

pub struct FnSystem<F>(pub F);

impl<F> System for FnSystem<F>
where
    F: IntoSystem,
{
    fn run(&self, components: &mut ComponentsStorage) {
        self.0.run(components);
    }
}

pub trait IntoSystem<Q>
where
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, components: &mut ComponentsStorage);
}

impl<F, Q> IntoSystem<Q> for F
where
    F: for<'a> Fn(Query<'a, Q>),
    for<'a> Q: Fetch<'a>,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::<Q>::new(components);
        (self)(query);
    }
}
