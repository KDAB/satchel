pub mod test_harness {
    use libtest_mimic::{Arguments, Trial};
    use linkme::distributed_slice;
    pub use satchel::{TestCase, TestFn};

    #[distributed_slice]
    pub static TESTS: [TestCase];

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_test_main() -> i32 {
        let args = Arguments::from_args();
        let tests: Vec<Trial> = TESTS
            .iter()
            .map(|case| {
                Trial::test(case.name, move || {
                    (case.test_fn)();
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod tests {
    use super::*;
    use satchel::test;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_adds_two_numbers() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn it_handles_zero() {
        let result = 0 + 0;
        assert_eq!(result, 0);
    }

    #[test]
    fn it_fails_intentionally() {
        assert_eq!(1, 2, "This is an intentional failure to check CTest output");
    }
}
