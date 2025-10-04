use libtest_mimic::Arguments;

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
    fn it_fails_intentionally() {
        assert_eq!(multiply(-1, -1), -2, "This is an intentional failure!");
    }

    #[bench]
    fn bench_multiply() {
        for i in 0..500 {
            let _ = multiply(i, i + 1);
        }
    }
}
