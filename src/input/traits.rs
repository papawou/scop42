// Specs
pub trait Pressable {
    type Pressed: Releasable;
    fn press(self) -> Self::Pressed;
}

pub trait Releasable {
    type Released: Pressable;
    fn release(self) -> Self::Released;
}

// Code specs
#[derive(Default, Debug, Clone, Copy)]
pub struct Down;
#[derive(Default, Debug, Clone, Copy)]
pub struct Up;

impl Pressable for Up {
    type Pressed = Down;
    fn press(self) -> Self::Pressed {
        Self::Pressed {
            ..Default::default()
        }
    }
}

impl Releasable for Down {
    type Released = Up;
    fn release(self) -> Self::Released {
        Self::Released {
            ..Default::default()
        }
    }
}
