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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    fn attributes_slice() -> &'static [&'static str] {
        test_runner::current_case_attributes()
    }

    fn threads_attribute_or(default: usize) -> usize {
        attributes_slice()
            .iter()
            .find_map(|attr| attr.strip_prefix("--threads=")?.parse::<usize>().ok())
            .unwrap_or(default)
    }

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

    // Test with ignore reason using #[ignore = "..."]
    #[test]
    #[ignore = "not yet implemented"]
    fn test_ignored_simple() {
        assert_eq!(multiply(2, 2), 4);
    }

    // Test with plain #[ignore] (no reason)
    #[test]
    #[ignore]
    fn test_ignored_failing() {
        assert_eq!(multiply(2, 2), 5, "This test fails but is ignored");
    }

    // Combining #[ignore] with #[should_panic]
    #[test]
    #[ignore]
    #[should_panic]
    fn test_ignored_with_panic() {
        panic!("This panic is ignored");
    }

    // Another test with ignore reason
    #[test]
    #[ignore = "performance test - takes too long"]
    fn test_ignored_performance() {
        for i in 0..1000000 {
            assert!(multiply(i, 1) == i);
        }
    }

    #[test("--threads=9")]
    fn test_attributes_control_parallelism() {
        let thread_count = threads_attribute_or(1);
        assert_eq!(thread_count, 9);

        let completions = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..thread_count)
            .map(|i| {
                let completions = Arc::clone(&completions);
                std::thread::spawn(move || {
                    assert_eq!(multiply(i as i32, 2), (i * 2) as i32);
                    completions.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("worker thread panicked");
        }

        assert_eq!(completions.load(Ordering::SeqCst), thread_count);
    }

    #[test(retry_on_failure)]
    fn test_retry_on_failure_runs_twice() {
        static FIRST_TRY: AtomicBool = AtomicBool::new(true);
        if FIRST_TRY
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            panic!("First try!");
        }
    }

    #[bench]
    fn bench_multiply() {
        for i in 0..500 {
            let _ = multiply(i, i + 1);
        }
    }

    #[bench]
    #[ignore]
    fn bench_multiply_ignored() {
        for i in 0..1000 {
            let _ = multiply(i, i * 2);
        }
    }
}
