// examples/ctest-integration/src/lib.rs
use libtest_mimic::{Arguments, Failed, Trial};
use std::panic;

fn run_tests(tests: &[satchel::test_harness::TestCase], args: Arguments) -> bool {
    let trials: Vec<Trial> = tests
        .iter()
        .cloned()
        .map(|case| {
            let kind_str = format!("{:?}", case.kind);
            let full_name = format!("{}::{}", case.module_path, case.name);

            match case.kind {
                satchel::TestKind::Unit => Trial::test(full_name, move || {
                    panic::catch_unwind(|| (case.test_fn)())
                        .map_err(|e| Failed::from(format!("Test panicked: {:?}", e)))
                })
                .with_kind(kind_str),
                satchel::TestKind::Benchmark => Trial::bench(full_name, move |test_mode| {
                    let result = panic::catch_unwind(|| (case.test_fn)());
                    match (test_mode, result) {
                        (true, Ok(_)) => Ok(None),
                        (true, Err(e)) => Err(Failed::from(format!(
                            "Bench panicked in test_mode: {:?}",
                            e
                        ))),
                        (false, Err(e)) => Err(Failed::from(format!(
                            "Bench panicked in bench mode: {:?}",
                            e
                        ))),
                        (false, Ok(_)) => {
                            use std::time::Instant;
                            const N: u64 = 1000;
                            let mut times = Vec::with_capacity(N as usize);

                            for _ in 0..N {
                                let start = Instant::now();
                                (case.test_fn)();
                                let elapsed = start.elapsed().as_nanos() as f64;
                                times.push(elapsed);
                            }

                            let avg = times.iter().sum::<f64>() / N as f64;
                            let variance =
                                times.iter().map(|&x| (x - avg).powi(2)).sum::<f64>() / N as f64;
                            Ok(Some(libtest_mimic::Measurement {
                                avg: avg.round() as u64,
                                variance: variance.round() as u64,
                            }))
                        }
                    }
                })
                .with_kind(kind_str),
            }
        })
        .collect();

    println!("trials: {:?}", trials);
    !libtest_mimic::run(&args, trials).has_failed()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn some_tests_main() -> i32 {
    let tests_ref = satchel::get_tests!();
    let tests: Vec<satchel::test_harness::TestCase> = tests_ref.into_iter().cloned().collect();
    let args = Arguments::from_args();

    if run_tests(&tests, args) { 0 } else { 1 }
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
