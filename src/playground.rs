use my_macros::system_test_fn;

use crate::ecs::query::Query;

#[system_test_fn]
pub fn test(query: Query::<(i32, bool)>) {}

#[cfg(test)]
mod tests {
    use my_macros::system_test_fn;

    #[test]
    fn internal() {}
}
