// Valid #[should_panic] forms to ensure they continue compiling.
use satchel::test;

// Provide an empty main so rustc treats this as a bin crate and does not error E0601.
fn main() {}

#[test]
#[should_panic]
fn bare_should_panic() { panic!("boom"); }

#[test]
#[should_panic(expected = "partial message")] // named form
fn named_expected() { panic!("some partial message here"); }

#[test]
#[should_panic = "substring shorthand"] // = shorthand form
fn eq_shorthand() { panic!("substring shorthand and more"); }

#[test]
#[should_panic("positional literal")] // positional form
fn positional_form() { panic!("xxx positional literal yyy"); }
