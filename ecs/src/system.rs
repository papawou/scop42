use std::marker::PhantomData;

use crate::world::World;

use super::query::{Fetch, Query};

pub trait SystemFn<'w> {
    fn run(&mut self, world: &'w mut World);
}

pub struct QuerySystem<F, Q> {
    f: F,
    _marker: PhantomData<Q>,
}
impl<F, Q> QuerySystem<F, Q> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

impl<'w, F, Q> SystemFn<'w> for QuerySystem<F, Q>
where
    F: Fn(Query<Q>),
    Q: for<'a> Fetch<'a>,
{
    fn run(&mut self, world: &'w mut World) {
        let query = Query::<'w, Q>::new(world);
        (self.f)(query);
    }
}
