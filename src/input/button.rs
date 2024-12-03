use std::time::Instant;

use super::traits::{Pressable, Releasable};

pub struct Down;
pub struct Up;

pub struct Button<State> {
    pub marker_state: std::marker::PhantomData<State>,
}

impl<State> Default for Button<State> {
    fn default() -> Self {
        Self {
            marker_state: std::marker::PhantomData,
        }
    }
}

impl Pressable for Button<Up> {
    type Pressed = Button<Down>;

    fn press(self) -> Self::Pressed {
        Self::Pressed::default()
    }
}

impl Releasable for Button<Down> {
    type Released = Button<Up>;

    fn release(self) -> Self::Released {
        Self::Released::default()
    }
}
