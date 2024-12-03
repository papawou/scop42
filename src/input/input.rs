use std::time::Instant;

use super::{
    button::{Button, Down, Up},
    traits::{Pressable, Releasable},
};

pub struct Input<State> {
    pub at: Instant,
    pub button: Button<State>,
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
        Self::Pressed::default()
    }
}

impl Releasable for Input<Down> {
    type Released = Input<Up>;

    fn release(self) -> Self::Released {
        Self::Released::default()
    }
}
