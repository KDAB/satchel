#![no_std]

pub use satchel_macro::{bench, test};

pub type TestFn = fn();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestKind {
    Unit,
    Benchmark,
}

#[derive(Debug, Clone)]
pub struct ShouldPanic {
    pub expected: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct Ignore {
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: &'static str,
    pub module_path: &'static str,
    pub kind: TestKind,
    pub test_fn: TestFn,
    pub should_panic: Option<ShouldPanic>,
    pub ignore: Option<Ignore>,
}

pub mod test_harness {
    pub use crate::{Ignore, ShouldPanic, TestCase};
    use linkme::distributed_slice;

    #[doc(hidden)]
    #[distributed_slice]
    pub static TESTS: [TestCase];
}

#[doc(hidden)]
pub fn extract_crate_name(module_path: &str) -> &str {
    module_path
        .split("::")
        .next()
        .expect("Split never returns a empty iterator")
}

#[doc(hidden)]
pub fn get_tests_for_crate(crate_prefix: &str) -> impl Iterator<Item = &'static TestCase> {
    let crate_name = extract_crate_name(crate_prefix);
    test_harness::TESTS
        .iter()
        .filter(move |case| case.module_path.starts_with(crate_name))
}

#[macro_export]
macro_rules! get_tests {
    () => {
        ::satchel::get_tests_for_crate(::core::module_path!())
    };
}

#[cfg(test)]
mod tests {
    use super::extract_crate_name;

    #[test]
    fn handles_empty_string() {
        assert_eq!(extract_crate_name(""), "");
    }

    #[test]
    fn extracts_simple_crate_name() {
        assert_eq!(extract_crate_name("mycrate"), "mycrate");
    }

    #[test]
    fn extracts_crate_name_with_module() {
        assert_eq!(extract_crate_name("mycrate::foo"), "mycrate");
        assert_eq!(extract_crate_name("mycrate::foo::bar"), "mycrate");
    }
}
