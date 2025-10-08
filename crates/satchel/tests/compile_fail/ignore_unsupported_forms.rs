mod common;
use satchel::test;

// Invalid form with parentheses and identifier
#[test]
#[ignore(bad)]
fn bad_ignore_form() {}

// Positional literal not supported
#[test]
#[ignore("reason")]
fn ignored_positional() {}

// Non-string literal reason
#[test]
#[ignore = 123]
fn ignored_non_string() {}

// Duplicate ignore attributes
#[test]
#[ignore]
#[ignore = "reason"]
fn duplicate_ignore() {}

fn main() {}