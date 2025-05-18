use crate::{
    query::{Fetch, Query},
    storage::ComponentsStorage,
};

pub trait ErasedGeneric {
    fn run(&self, components: &mut ComponentsStorage);
}

pub trait System {
    fn run(&self, components: &mut ComponentsStorage);
}

impl ErasedGeneric for dyn System {
    fn run(&self, components: &mut ComponentsStorage) {
        self.run(components);
    }
}

impl<'w, Q> System for dyn Fn(Query<'w, Q>)
where
    Q: Fetch<'w>,
{
    fn run(&self, components: &mut ComponentsStorage) {
        let query = Query::<'w>::new(components);
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
