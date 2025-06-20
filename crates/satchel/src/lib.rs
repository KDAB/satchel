pub use satchel_macro::test;

pub type TestFn = fn();

#[derive(Debug, Clone, Copy)]
pub struct TestCase {
    pub name: &'static str,
    pub test_fn: TestFn,
}
