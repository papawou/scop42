pub type System = fn(&mut World, &mut Query);

pub trait System<'world, Q: FetchComponents<'world>> {
    fn run(&mut self, world: &mut World, query: &mut Query);
}
