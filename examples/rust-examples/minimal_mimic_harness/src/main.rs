use libtest_mimic::{Arguments, Failed, Trial};

mod tests;

fn main() {
    let args = Arguments::from_args();
    let tests = vec![
        Trial::test("happy_test", runner(tests::happy_test)),
        Trial::test(
            "it_fails_intentionally",
            runner(tests::it_fails_intentionally),
        ),
    ];
    libtest_mimic::run(&args, tests).exit()
}

// A test, for `libtest_mimic`, is a function with no arguments that returns a `Result<(), Failed>`.
// We adapt the signature of our tests to this format: if a test doesn't panic, it's a success.
fn runner(test: fn() -> ()) -> impl FnOnce() -> Result<(), Failed> + Send + 'static {
    move || {
        test();
        Ok(())
    }
}
