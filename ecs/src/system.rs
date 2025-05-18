use crate::{
    query::{Fetch, Query},
    storage::ComponentsStorage,
};

pub trait ErasedGeneric<'w> {
    fn run(&self, components: &'w mut ComponentsStorage);
}

pub trait System<'w> {
    fn run(&self, components: &'w mut ComponentsStorage);
}

impl<'w> ErasedGeneric<'w> for dyn System<'w> {
    fn run(&self, components: &'w mut ComponentsStorage) {
        self.run(components);
    }
}

impl<'w, Q> System<'w> for dyn Fn(Query<'w, Q>)
where
    Q: Fetch<'w>,
{
    fn run(&self, components: &'w mut ComponentsStorage) {
        let query: Query<'w, Q> = Query::new(components);
        (self)(query)
    }
}

impl<'w, Q> System<'w> for fn(Query<'w, Q>)
where
    Q: Fetch<'w>,
{
    fn run(&self, components: &'w mut ComponentsStorage) {
        let query = Query::<'w, Q>::new(components);
        (self)(query);
    }
}
