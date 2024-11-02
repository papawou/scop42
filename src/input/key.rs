use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};
pub mod button;
mod traits;

pub mod winit_impl;

// use state::{State, StateAPI};
// pub use winit_impl as winit;

// mod state;

// // Default manager with HashMap
// pub struct InputManager<T> {
//     keys: HashMap<T, State>,
// }

// impl<T> InputManager<T> {
//     pub fn new() -> Self {
//         Self {
//             keys: HashMap::new(),
//         }
//     }
// }

// // Default impl
// #[derive(Hash, Eq, PartialEq, Debug, Clone)]
// pub struct DeviceInput<TDevice, TInput>(TDevice, TInput);

// impl<TDevice, TInput> StateAPI for InputManager<DeviceInput<TDevice, TInput>>
// where
//     TDevice: Hash + Eq + PartialEq + Clone,
//     TInput: Hash + Eq + PartialEq + Clone,
// {
//     fn is_press(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         self.keys.get(&input).and_then(|state| match state {
//             State::Pressed(instant) => Some(instant.clone()),
//             _ => None,
//         })
//     }

//     fn is_release(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         self.keys.get(&input).and_then(|state| match state {
//             State::Released(instant) => Some(instant.clone()),
//             _ => None,
//         })
//     }

//     fn try_press(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         self.keys
//             .entry(input.clone())
//             .or_insert(State::Pressed(Instant::now()))
//             .try_press()
//     }

//     fn try_release(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         match self.keys.entry(input.clone()) {
//             hash_map::Entry::Occupied(o) => match o.get() {
//                 State::Released(instant) => Some(State::Released(*instant)),
//                 _ => self.release(input.clone()),
//             },
//             hash_map::Entry::Vacant(v) => self.release(input.clone()),
//         }
//     }

//     fn press(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         self.keys
//             .entry(input)
//             .or_insert(input, State::Pressed(Instant::now()))
//             .press()
//     }

//     fn release(&mut self, input: DeviceInput<TDevice, TInput>) -> State {
//         self.keys.insert(input, State::Released(Instant::now()))
//     }
// }
