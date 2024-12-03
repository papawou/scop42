pub trait [[Releasable]] {
    fn release() -> [[Released]] {
        [[Released]]
    }
}

impl [[Releasable]] for [[Pressed]] {}