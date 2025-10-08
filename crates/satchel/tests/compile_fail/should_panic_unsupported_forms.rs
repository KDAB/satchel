mod common;
use satchel::test;

// Multiple positional arguments
#[test]
#[should_panic("a", "b")]
fn multi_positional_is_invalid() {}

// Mixed positional and named arguments (named first)
#[test]
#[should_panic(expected = "boom", "other")]
fn mixed_positional() {}

// Mixed positional and named arguments (positional first)
#[test]
#[should_panic("boom", expected = "boom")]
fn mixed_reverse() {}

// Unknown key in named arguments
#[test]
#[should_panic(bad = "format")]
fn bad_should_panic_form() {}

// Non-string value for named argument
#[test]
#[should_panic(message = 123)]
fn should_panic_non_string() {}

// Duplicate expected arguments
#[test]
#[should_panic(expected = "a", expected = "b")]
fn dup_expected() {}

// Duplicate should_panic attributes
#[test]
#[should_panic]
#[should_panic(expected = "boom")]
fn duplicate_should_panic() {}

fn main() {}