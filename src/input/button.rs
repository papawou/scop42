use std::time::Instant;

use super::traits::{Down, Pressable, Releasable, Up};

// # Button
#[derive(Default)]
pub struct Button<State> {
    pub marker_state: std::marker::PhantomData<State>,
}

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
        }
    }
}
