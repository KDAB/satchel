// Re-hosted compile-fail tests for proc-macro diagnostics in the main `satchel` crate.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/ignore_unsupported_forms.rs");
    t.compile_fail("tests/compile_fail/should_panic_unsupported_forms.rs");
}
