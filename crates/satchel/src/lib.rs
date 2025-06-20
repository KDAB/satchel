pub use satchel_macro::{bench, test};

pub type TestFn = fn();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestKind {
    Unit,
    Benchmark,
}

#[derive(Debug, Clone, Copy)]
pub struct TestCase {
    pub name: &'static str,
    pub module_path: &'static str,
    pub kind: TestKind,
    pub test_fn: TestFn,
}

pub mod test_harness {
    pub use crate::{TestCase, TestFn, TestKind};
    use linkme::distributed_slice;

    #[distributed_slice]
    pub static TESTS: [TestCase];

    /// Returns only the tests whose module_path starts with the given prefix.
    pub fn get_tests_for_crate(crate_prefix: &str) -> Vec<&'static TestCase> {
        TESTS
            .iter()
            .filter(|case| case.module_path.starts_with(crate_prefix))
            .collect()
    }
}
