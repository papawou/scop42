use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

pub mod winit_impl;

pub use winit_impl as winit;

pub trait Manager<T> {
    fn is_press(&mut self, input: T) -> bool;
    fn is_release(&mut self, input: &T) -> bool;
    fn press(&mut self, input: &T) -> bool;
    fn release(&mut self, input: &T) -> bool;
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

// enum Test<T> {
//     Bot(bool),
//     Winit(WinitDeviceInput),
// }

// Default impl
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct DeviceInput<TDevice, TInput>(TDevice, TInput);
