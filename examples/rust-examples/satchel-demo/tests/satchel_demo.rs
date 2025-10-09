use libtest_mimic::Arguments;
use std::env;
use test_runner;

pub fn discover_and_run() -> bool {
    // Check for `--list` before parsing libtest-mimic args, because nextest
    // may pass flags like `--format terse` that libtest-mimic doesn't know about.
    let args_raw: Vec<String> = env::args().collect();
    if args_raw.iter().any(|a| a == "--list") {
        // nextest manual harness contract: https://nexte.st/docs/design/custom-test-harnesses/
        // - MUST support `--list --format terse` and print only `name: kind` lines to stdout
        // - MUST support `--list --format terse --ignored` to print exactly the set of ignored tests
        let list_ignored = args_raw.iter().any(|a| a == "--ignored");

        let mut tests: Vec<&'static satchel::TestCase> = satchel::get_tests!().collect();
        // Sort to provide a stable, alphabetical order similar to nextest's listing
        tests.sort_by_key(|t| (t.module_path, t.name));

        // Emit text format expected by nextest fallback list parser
        for t in &tests {
            // Listing policy:
            // - Default (no --ignored): list only non-ignored tests
            // - With --ignored: list only ignored tests
            if list_ignored {
                if t.ignore.is_none() {
                    continue;
                }
            } else {
                if t.ignore.is_some() {
                    continue;
                }
            }

            let full_name = format!("{}::{}", t.module_path, t.name);
            // Do not append ignore reasons to names; keep names identical to runtime Trial names
            let suffix = match t.kind {
                satchel::TestKind::Unit => "test",
                // nextest expects the suffix to be ': benchmark'
                satchel::TestKind::Benchmark => "benchmark",
            };
            // Name must exactly match runtime trial name for filters to work
            println!("{}: {}", full_name, suffix);
        }
        return true;
    }

    // Normal run via libtest-mimic
    let args = Arguments::from_args();
    test_runner::run_tests(satchel::get_tests!(), args)
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
