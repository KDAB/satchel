//! Test registration and discovery primitives for custom harnesses built on Satchel.
//!
//! This crate collects metadata for functions annotated with [`macro@test`] and
//! [`macro@bench`], storing them in a linkme-powered distributed slice. Harnesses invoke
//! [`get_tests!`] to enumerate the [`TestCase`] entries for the current crate, inspect
//! [`TestCase::case_attributes`] for custom markers, and run or filter tests and benchmarks as
//! needed. See the project README for end-to-end examples, including CTest integration and
//! `libtest-mimic` runners.
#![no_std]

/// The main macro of the satchel crate used to register unit tests with the Satchel harness.
/// It works like the standard Rust [test macro](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html#unit-testing), 
/// but can be used with custom test harnesses.
///
/// Satchel supports the standard `#[ignore]` and `#[should_panic="..."]` attributes, 
/// as well as custom arguments that can be freely interpreted by the test harness via `#[test(...)]`
///
/// # Examples
/// ```no_run
/// use satchel::test;
///
/// #[test]
/// fn adds_numbers() {
///     assert_eq!(2 + 2, 4);
/// }
/// ```
pub use satchel_macro::test;

/// This is a variant of the [`macro@test`] macro that will result in [TestKind::Benchmark].
/// See the documentation on [`macro@test`] for details.
///
/// # Examples
/// ```no_run
/// use satchel::bench;
///
/// #[bench]
/// fn spin_loop() {
///     let mut sum = 0u64;
///     for i in 0..100 {
///         sum = sum.wrapping_add(i);
///     }
///     assert!(sum > 0);
/// }
/// ```
pub use satchel_macro::bench;

/// Function pointer for bare test entry points.
pub type TestFn = fn();

/// Classification of a registered case.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestKind {
    /// Standard unit-style test.
    Unit,
    /// Benchmark case executed by the harness.
    Benchmark,
}

/// Metadata describing an expected panic.
#[derive(Debug, Clone)]
pub struct ShouldPanic {
    /// Optional substring that must appear in the panic payload.
    pub expected: Option<&'static str>,
}

/// Metadata describing whether a case should be skipped by default.
#[derive(Debug, Clone)]
pub struct Ignore {
    /// Optional reason string carried alongside the skip flag.
    pub reason: Option<&'static str>,
}

/// Static description of a registered test or benchmark.
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Name of the function as it appears in the source crate.
    pub name: &'static str,
    /// Fully qualified module path for the test function.
    pub module_path: &'static str,
    /// Kind of case (unit test or benchmark).
    pub kind: TestKind,
    /// Entry point invoked by the harness.
    pub test_fn: TestFn,
    /// Panic expectations attached via `#[should_panic]`.
    pub should_panic: Option<ShouldPanic>,
    /// Optional ignore flag populated from `#[ignore]`.
    pub ignore: Option<Ignore>,
    /// Additional markers supplied through `#[test(...)]` or `#[bench(...)]`.
    pub case_attributes: &'static [&'static str],
}

/// Distributed slice exposing registered cases to harness implementations.
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
#[doc = "Returns an iterator over the [`TestCase`] entries belonging to the current crate."]
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
