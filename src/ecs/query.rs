pub struct Query<T>(Vec<T>);

impl<T> Query<T> {
    pub fn new() -> Self {
        Self(vec![])
    }
}
