use std::time::Instant;

// # Traits - make sense to have codependent traits ?
pub trait Pressable {
    type Pressed: Releasable;
    fn press(self) -> Self::Pressed;
}
pub trait Releasable {
    type Released: Pressable;
    fn release(self) -> Self::Released;
}

// # Button
struct Button<S> {
    marker_state: std::marker::PhantomData<S>,
}

// ## Button states
struct Down; // type ButtonPressed = Button<Pressed>; ?
struct Up; // Button<Released>; ?

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

pub type ButtonPressed = Button<Down>;
pub type ButtonReleased = Button<Up>;
