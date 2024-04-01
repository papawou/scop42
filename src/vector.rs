#[derive(Debug, Default, Copy, Clone)]
pub struct Vec2<T>
where
    T: Copy,
{
    pub x: T,
    pub y: T,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}
