#[cfg(feature = "tests")]
use rust_test_export::test_harness;

#[cfg(feature = "tests")]
fn main() {
    std::process::exit(test_harness::rust_test_main());
}

#[cfg(not(feature = "tests"))]
fn main() {
    eprintln!("Run with --features tests to execute tests");
}
