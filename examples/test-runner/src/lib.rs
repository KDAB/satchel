use libtest_mimic::{Arguments, Failed, Trial};
use satchel::test_harness::TestCase;
use std::panic;

pub fn run_tests(tests: impl Iterator<Item = &'static TestCase>, args: Arguments) -> bool {
    let trials: Vec<Trial> = tests
        .into_iter()
        .map(|case| {
            let full_name = format!("{}::{}", case.module_path, case.name);
            let kind_str = format!("{:?}", case.kind);

            match case.kind {
                satchel::TestKind::Unit => {
                    let should_panic = case.should_panic;
                    Trial::test(full_name, move || {
                        let result = panic::catch_unwind(|| (case.test_fn)());
                        match (should_panic, result) {
                            (Some(panic_info), Err(e)) => {
                                // Extract the panic message
                                let panic_msg = if let Some(msg) = e.downcast_ref::<&str>() {
                                    *msg
                                } else if let Some(msg) = e.downcast_ref::<String>() {
                                    msg.as_str()
                                } else {
                                    return Err(Failed::from("Test panicked with a non-string message"));
                                };

                                // If an expected message was provided, check that it matches
                                if let Some(expected_msg) = panic_info.expected {
                                    if !panic_msg.contains(expected_msg) {
                                        return Err(Failed::from(format!(
                                            "Panic message did not contain expected string.\nExpected substring: {}\n      Found string: {}",
                                            expected_msg, panic_msg
                                        )));
                                    }
                                }
                                Ok(())
                            }
                            (Some(_), Ok(_)) => Err(Failed::from("Expected panic did not occur")),
                            (None, Ok(_)) => Ok(()), // normal pass
                            (None, Err(e)) => Err(Failed::from(format!("Unexpected panic: {:?}", e))),
                        }
                    })
                    .with_kind(kind_str)
                }
                satchel::TestKind::Benchmark => {
                    Trial::bench(full_name, move |test_mode| {
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
                    .with_kind(kind_str)
                }
            }
        })
        .collect();

    !libtest_mimic::run(&args, trials).has_failed()
}
