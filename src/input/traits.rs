pub trait Pressable {
    type Pressed: Releasable;
    fn press(self) -> Self::Pressed;
}
pub trait Releasable {
    type Released: Pressable;
    fn release(self) -> Self::Released;
}

pub struct Down; // type ButtonPressed = Button<Pressed>; ?
pub struct Up; // Button<Released>; ?
