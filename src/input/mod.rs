use std::{
    collections::{hash_map, HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

use button::{ButtonPressed, ButtonReleased};
pub mod button;

pub mod winit_impl;

enum Button {
    Pressed(ButtonPressed),
    Released(ButtonReleased),
}

// // Default manager with HashMap
pub struct Keys<K, V>(HashMap<K, V>);

impl Keys<Instant, Button> {
    fn press(&mut self, key: Instant) {
        self.0.insert(key, Button::Pressed);
    }
}

struct Input {
    at: Instant, // when the state trait change (action property?) (second business rule ? first will be Button: Pressable+Releasable)
    state: Button,
}

// pub trait StateAPI {
//     type Input;
//     type Output;

//     // query
//     fn is_press(&self, input: Self::Input) -> Self::Output;
//     fn is_release(&self, input: Self::Input) -> Self::Output;

//     // no override == dont refresh timer
//     // fn try_press(&mut self, input: Self::Input) -> Self::Output;
//     // fn try_release(&mut self, input: Self::Input) -> Self::Output;

//     // override == refresh timer
//     fn press(&mut self, input: Self::Input) -> Self::Output;
//     fn release(&mut self, input: Self::Input) -> Self::Output;
// }

// struct Key<T>(T); // type PressedKey<Pressed> = Key<Pressed>
// impl Pressable for Key<Released> {
//     type Pressed = Key<Pressed>;

//     fn press(self) -> Self {
//         Key(Pressed)
//     }
// }

// struct State<T> {
//     instant: Instant,
//     _marker: std::marker::PhantomData<T>,
// }

// impl State<Pressed> {
//     fn release(self) -> State<Released> {
//         State {
//             instant: Instant::now(),
//             _marker: std::marker::PhantomData,
//         }
//     }

//     fn is_press() -> bool {
//         true
//     }

//     fn is_release() -> bool {
//         false
//     }
// }

// impl State<Released> {
//     fn press(self) -> State<Pressed> {
//         State {
//             instant: Instant::now(),
//             _marker: std::marker::PhantomData,
//         }
//     }

//     fn is_press() -> bool {
//         false
//     }

//     fn is_release() -> bool {
//         true
//     }
// }

// pub enum State {
//     Pressed(Instant),
//     Released(Instant),
// }

// impl StateAPI for State {
//     type Input = ();
//     type Output<'a> = &'a mut Self;

//     fn press(&mut self, (_)) -> &mut Self {
//         *self = State::Pressed(Instant::now());
//         self
//     }

//     fn release(&mut self) -> &mut Self {
//         *self = State::Released(Instant::now());
//         self
//     }

//     fn is_press(&self, input: Self::Input) -> Self::Output {
//         todo!()
//     }

//     fn is_release(&self, input: Self::Input) -> Self::Output {
//         todo!()
//     }
//     // need - (trait Manager?) - (Code...) - Instant - when a key is pressed <- State is action of user
// }

// impl StateType  {
//     fn try_release(&mut self) -> &mut Self {
//         match self {
//             State::Pressed(_) => self.release(),
//             _ => self,
//         }
//     }

//     fn try_press(&mut self) -> &mut Self {
//         match self {
//             State::Released(_) => self.press(),
//             _ => self,
//         }
//     }
// }

// use state::{State, StateAPI};
// pub use winit_impl as winit;

// mod state;

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
