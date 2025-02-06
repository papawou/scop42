use std::time::Instant;

use super::traits::{Pressable, Releasable};

#[derive(Clone, Copy)]
pub struct Down;
#[derive(Clone, Copy)]
pub struct Up;

#[derive(Clone, Copy)]
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
        Default::default()
    }
}

impl Releasable for Button<Down> {
    type Released = Button<Up>;

    fn release(self) -> Self::Released {
        Default::default()
    }
}
