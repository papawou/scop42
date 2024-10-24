use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

pub mod winit_impl;

pub use winit_impl as winit;

pub trait Manager<T> {
    // query
    fn is_press(&mut self, input: T) -> Option<Instant>;
    fn is_release(&mut self, input: T) -> Option<Instant>;

    // returns prev value ?
    fn try_press(&mut self, input: T) -> Option<State>;
    fn try_release(&mut self, input: T) -> Option<State>;

    // returns prev value ?
    fn press(&mut self, input: T) -> Option<State>; // override
    fn release(&mut self, input: T) -> Option<State>; // override
}

pub enum State {
    Pressed(Instant),
    Released(Instant),
}

// Default manager with HashMap
pub struct InputManager<T> {
    keys: HashMap<T, State>,
}

impl<T> InputManager<T> {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
}

// Default impl
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct DeviceInput<TDevice, TInput>(TDevice, TInput);
