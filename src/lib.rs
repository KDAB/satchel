#[cfg(feature = "tests")]
pub mod test_harness {
    pub(crate) type TestFn = fn();
    pub(crate) type TestData = (&'static str, TestFn);

    use libtest_mimic::{Arguments, Trial};
    use linkme::distributed_slice;

    #[distributed_slice]
    pub(crate) static TESTS: [TestData];

    #[no_mangle]
    pub extern "C" fn rust_test_main() -> i32 {
        let args = Arguments::from_args();
        let tests: Vec<Trial> = TESTS
            .iter()
            .map(|(name, test_fn)| {
                Trial::test(*name, move || {
                    test_fn();
                    Ok(())
                })
            })
            .collect();

        if libtest_mimic::run(&args, tests).has_failed() {
            1
        } else {
            0
        }
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::test_harness::{TestData, TESTS};
    use linkme::distributed_slice;

    #[distributed_slice(TESTS)]
    static TEST_A: TestData = ("test_a", test_a);

    fn test_a() {
        println!("Running test_a");
        assert_eq!(1 + 1, 2);
    }

    #[distributed_slice(TESTS)]
    static TEST_B: TestData = ("test_b", test_b);

    fn test_b() {
        println!("Running test_b");
        assert_eq!(2 * 2, 4);
    }
}
