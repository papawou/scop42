use std::time::Instant;

use super::traits::{Pressable, Releasable};

// # Button
pub struct Button<S> {
    marker_state: std::marker::PhantomData<S>,
}

// ## Button states
pub struct Down; // type ButtonPressed = Button<Pressed>; ?
pub struct Up; // Button<Released>; ?

// impl Button
impl Pressable for Button<Up> {
    type Pressed = Button<Down>;

    fn press(self) -> Self::Pressed {
        Self::Pressed {
            marker_state: std::marker::PhantomData,
        }
    }
}

impl Releasable for Button<Down> {
    type Released = Button<Up>;

    fn release(self) -> Self::Released {
        Self::Released {
            marker_state: std::marker::PhantomData,
            //..self
        }
    }
}
