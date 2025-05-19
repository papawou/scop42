use crate::{
    query::{Fetch, Query},
    storage::ComponentsStorage,
};

pub trait System {
    fn run(&self, components: &mut ComponentsStorage);
}

impl<Q> System for dyn Fn(Query<'_, Q>)
where
    Q: Fetch,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::new(components);
        (self)(query)
    }
}

impl<Q> System for fn(Query<'_, Q>)
where
    Q: Fetch,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::new(components);
        (self)(query);
    }
}
