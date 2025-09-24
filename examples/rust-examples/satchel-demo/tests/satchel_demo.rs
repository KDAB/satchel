use libtest_mimic::Arguments;
use test_runner;

pub fn discover_and_run() -> bool {
    let tests = satchel::get_tests!();
    let args = Arguments::from_args();
    test_runner::run_tests(tests, args)
}

fn main() {
    let success = discover_and_run();
    std::process::exit(if success { 0 } else { 1 });
}

pub mod tests {
    use satchel::{bench, test};
    use satchel_demo::multiply;

    #[test]
    fn test_multiply_positive() {
        assert_eq!(multiply(2, 3), 6);
    }

    #[test]
    fn test_multiply_zero() {
        assert_eq!(multiply(0, 10), 0);
    }

    #[test]
    fn test_multiply_negative() {
        assert_eq!(multiply(-2, 3), -6);
    }

    #[test]
    #[should_panic]
    fn test_divide_by_zero_panics() {
        fn get_zero() -> i32 {
            0
        }
        let _result = 1 / get_zero();
    }

    // Using syntax: #[should_panic(expected = "...")]
    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn test_divide_by_zero_with_message() {
        fn get_zero() -> i32 {
            0
        }
        let _result = 1 / get_zero();
    }

    // Using syntax: #[should_panic("...")]
    #[test]
    #[should_panic("invalid multiplier: 42")]
    fn test_custom_panic_formatted() {
        let invalid_value = 42;
        panic!("invalid multiplier: {}", invalid_value);
    }

    // Using syntax: #[should_panic = "..."]
    #[test]
    #[should_panic = "multiplier"]
    fn test_custom_panic_partial_match() {
        panic!("Error: invalid multiplier in calculation");
    }

    #[bench]
    fn bench_multiply() {
        for i in 0..500 {
            let _ = multiply(i, i + 1);
        }
    }
}
