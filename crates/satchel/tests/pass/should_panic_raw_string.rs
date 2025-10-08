// Ensures raw strings are accepted in `expected = r"..."` form.
use satchel::test;

fn main() {}

#[test]
#[should_panic(expected = r"raw msg")] // raw string literal
fn raw_string_expected() {
    panic!("some prefix raw msg some suffix");
}
