use std::marker::PhantomData;

use glam::Vec3;

use super::{Entity, World};

// The `FetchComponents` trait is used to fetch components from the world.
// It is a generic trait that can be implemented for different types of queries.
pub trait FetchComponents<'a> {
    type Iter: Iterator<Item = (Entity, Self::Item)> + 'a;
    type Item;

    fn fetch(world: &'a World) -> Self::Iter;
}

// The `Query` struct is generic over the query type and the world.
// The `Query` struct has a method `iter` that returns an iterator over the components
// fetched from the world.
pub struct Query<'a, Q: FetchComponents<'a>> {
    world: &'a World,
    _marker: PhantomData<Q>,
}

impl<'a, Q> Query<'a, Q>
where
    Q: FetchComponents<'a>,
{
    pub fn iter(&self) -> Q::Iter {
        Q::fetch(self.world)
    }
}

// The `System` trait is implemented for any function that takes a `Query` as an argument.
// The `System` trait is generic over the query type and the world.
// The `System` trait has a method `run` that takes a mutable reference to the world
// and runs the system.
pub trait System<'a> {
    type Query: FetchComponents<'a>;
    fn run(&self, world: &'a World);
}

impl<'a, F, Q> System<'a> for F
where
    Q: FetchComponents<'a>,
    F: Fn(Query<'a, Q>) + 'static,
{
    type Query = Q;

    fn run(&self, world: &'a World) {
        let query = Query::<Q> {
            world,
            _marker: PhantomData,
        };
        (self)(query);
    }
}
