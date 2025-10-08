#[test]
fn should_panic_forms_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}
