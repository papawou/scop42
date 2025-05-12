use std::marker::PhantomData;

use super::World;

pub struct Query<'w, Q: QueryData> {
    world: &'w World,
    _marker: PhantomData<Q>,
}

impl<Q> IntoIterator for Query<'_, Q>
where
    Q: QueryData,
{
    type Item = Q::Item;
    type IntoIter = Q::Iter;

    fn into_iter(self) -> Self::IntoIter {
        Q::fetch(self.world)
    }
}

pub trait QueryData {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn fetch(world: &World) -> Self::Iter;
}

impl QueryData for i32 {
    type Item = i32;
    type Iter = std::slice::Iter<i32>;

    fn fetch(world: &World) -> Self::Iter {
        vec![].iter()
    }
}

pub trait System<T: QueryData> {
    fn run(&self, world: &World);
}

impl<F, Q> System<Q> for F
where
    F: Fn(Q::Iter),
    Q: QueryData,
{
    fn run(&self, world: &World) {
        let iter = Q::fetch(world);
        (self)(iter);
    }
}

fn a_system(query: Query<i32>) {
    for a_number in query {
        // Do something with the query
    }
}
