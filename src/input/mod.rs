use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    time::Instant,
};

mod winit;

// Input(key, state, timestamp)

// use std::{
//     collections::{HashSet, VecDeque},
//     hash::Hash,
// };
// pub struct Input<TInput>  {
//     pressed: HashSet<TInput>,
//     just_pressed: HashSet<TInput>,
//     just_released: HashSet<TInput>,

//     // events: VecDeque<TEvent>,
// }

// impl<TInput> Input<TInput> {
//     pub fn new() -> Self {
//         Self {
//             pressed: HashSet::new(),
//             just_pressed: HashSet::new(),
//             just_released: HashSet::new(),
//         }
//     }

//     pub fn add<T>(&mut self, key_event: T)
//     where
//         TEvent: From<T>,
//     {
//         self.events.push_back(key_event.into());
//     }

//     pub fn press<T>(&mut self, key_event: T)
//     where
//         TInput: From<T>,
//     {
//         self.pressed()
//     }

//     pub fn is_pressed<T>(&self, key: T)
//     where
//         TInput: From<T> + Eq + Hash,
//     {
//         self.pressed.(&key.into()).is_some();
//     }
// }

trait InputHandler<T> {
    fn press(&mut self, input: T);
    fn release(&mut self, input: T);
    fn hold(&mut self, input: T);
    fn is_press(&mut self, input: &T) -> bool;
    fn is_release(&mut self, input: &T) -> bool; // returns !is_press
}

pub enum State {
    Pressed(Instant),
    Released(Instant),
}

pub struct InputManager<T, E> {
    keys: HashMap<T, State>,
    queue: VecDeque<E>,
}

impl<T, E> InputManager<T, E> {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            queue: VecDeque::new(),
        }
    }
}

impl<T, E> InputManager<T, E> {
    pub fn enqueue(&mut self, event: E) {
        self.queue.push_back(event);
    }

    pub fn poll(&mut self) -> Option<E> {
        self.queue.pop_front()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl<T, E> InputHandler<T> for InputManager<T, E>
where
    T: Eq + Hash,
{
    fn press(&mut self, input: T) {
        self.keys.insert(input, State::Pressed(Instant::now()));
    }

    fn release(&mut self, input: T) {
        self.keys.insert(input, State::Released(Instant::now()));
    }

    // how long is press to be considered as hold ?
    fn hold(&mut self, input: T) {
        todo!()
    }

    fn is_press(&mut self, input: &T) -> bool {
        match self.keys.get(input) {
            Some(State::Pressed(_)) => true,
            _ => false,
        }
    }

    fn is_release(&mut self, input: &T) -> bool {
        !self.is_press(input)
    }
}
