use std::marker::PhantomData;

use traits::{IntoSystem, IntoSystemMut, System, SystemMut};

use crate::{
    query::traits::{Fetch, FetchMut},
    storage::ComponentsStorage,
};

pub mod traits;

// FnSystem
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

// FnSystemMut
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

// utils
pub fn system<Q, F>(f: F) -> Box<dyn System>
where
    F: IntoSystem<Q> + 'static,
    for<'a> Q: Fetch<'a> + 'static,
{
    Box::new(FnSystem(f, PhantomData))
}

pub fn system_mut<Q, F>(f: F) -> Box<dyn SystemMut>
where
    F: IntoSystemMut<Q> + 'static,
    for<'a> Q: FetchMut<'a> + 'static,
{
    Box::new(FnSystemMut(f, PhantomData))
}
