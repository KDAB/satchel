use libtest_mimic::Arguments;
use test_runner;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn other_tests_main() -> i32 {
    println!("Running tests in {}", std::module_path!());
    let tests = satchel::get_tests!();
    let args = Arguments::from_args();

    if test_runner::run_tests(tests, args) {
        0
    } else {
        1
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod tests {
    use super::*;
    use satchel::{bench, test};

    #[bench]
    fn benchmark_add() {
        for i in 0..1000 {
            add(i, i + 1);
        }
    }

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
    fn test_that_panics() {
        panic!("This test should panic");
    }

    #[test]
    fn it_fails_intentionally() {
        assert_eq!(1, 2, "This is an intentional failure to check CTest output");
    }
}
