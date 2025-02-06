use std::time::Instant;

use super::{
    button::{Button, Down, Up},
    traits::{Pressable, Releasable},
};

#[derive(Clone, Copy)]
pub struct Input<State> {
    pub at: Instant,
    pub button: Button<State>,
}

impl<State> Input<State> {
    pub fn refresh(&mut self) -> &mut Self {
        self.at = Instant::now();
        self
    }
}

impl<State> Default for Input<State> {
    fn default() -> Self {
        Self {
            at: Instant::now(),
            button: Button::<State>::default(),
        }
    }
}

impl Pressable for Input<Up> {
    type Pressed = Input<Down>;

    fn press(self) -> Self::Pressed {
        Default::default()
    }
}

impl Releasable for Input<Down> {
    type Released = Input<Up>;

    fn release(self) -> Self::Released {
        Default::default()
    }
}
