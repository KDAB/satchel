use libtest_mimic::{Arguments, Failed, Trial};
use satchel::test_harness::TestCase;
use std::panic;

pub fn run_tests(tests: impl Iterator<Item = &'static TestCase>, args: Arguments) -> bool {
    let trials: Vec<Trial> = tests.map(create_trial_for_case).collect();
    !libtest_mimic::run(&args, trials).has_failed()
}

fn format_test_name(case: &TestCase) -> String {
    let base_name = format!("{}::{}", case.module_path, case.name);

    match &case.ignore {
        Some(ignore) => {
            if let Some(reason) = ignore.reason {
                format!("{} (ignored: {})", base_name, reason)
            } else {
                base_name
            }
        }
        None => base_name,
    }
}

fn apply_ignore_flag(trial: Trial, case: &TestCase) -> Trial {
    if case.ignore.is_some() {
        trial.with_ignored_flag(true)
    } else {
        trial
    }
}

fn handle_unit_test(
    result: std::thread::Result<()>,
    should_panic: Option<satchel::ShouldPanic>,
) -> Result<(), Failed> {
    match (should_panic, result) {
        (Some(panic), Err(e)) => handle_expected_panic(e, panic),
        (Some(_), Ok(_)) => Err(Failed::from("Expected panic did not occur")),
        (None, Ok(_)) => Ok(()),
        (None, Err(e)) => Err(Failed::from(format!("Unexpected panic: {:?}", e))),
    }
}

fn handle_expected_panic(
    e: Box<dyn std::any::Any + Send>,
    panic: satchel::ShouldPanic,
) -> Result<(), Failed> {
    let panic_msg = if let Some(msg) = e.downcast_ref::<&str>() {
        *msg
    } else if let Some(msg) = e.downcast_ref::<String>() {
        msg.as_str()
    } else {
        return Err(Failed::from("Test panicked with a non-string message"));
    };

    if let Some(expected_msg) = panic.expected {
        if !panic_msg.contains(expected_msg) {
            return Err(Failed::from(format!(
                "Panic message did not contain expected string.\nExpected substring: {}\n      Found string: {}",
                expected_msg, panic_msg
            )));
        }
    }
    Ok(())
}

fn run_benchmark(test_fn: fn()) -> Result<Option<libtest_mimic::Measurement>, Failed> {
    use std::time::Instant;
    const N: u64 = 1000;
    let mut times = Vec::with_capacity(N as usize);

    for _ in 0..N {
        let start = Instant::now();
        test_fn();
        let elapsed = start.elapsed().as_nanos() as f64;
        times.push(elapsed);
    }

    let avg = times.iter().sum::<f64>() / N as f64;
    let variance = times.iter().map(|&x| (x - avg).powi(2)).sum::<f64>() / N as f64;
    Ok(Some(libtest_mimic::Measurement {
        avg: avg.round() as u64,
        variance: variance.round() as u64,
    }))
}

fn create_trial_for_case(case: &'static TestCase) -> Trial {
    let full_name = format_test_name(case);
    let kind_str = format!("{:?}", case.kind);

    match case.kind {
        satchel::TestKind::Unit => {
            let should_panic = case.should_panic.clone();
            let trial = Trial::test(full_name, move || {
                let result = panic::catch_unwind(|| (case.test_fn)());
                handle_unit_test(result, should_panic)
            })
            .with_kind(kind_str);
            apply_ignore_flag(trial, case)
        }
        satchel::TestKind::Benchmark => {
            let trial = Trial::bench(full_name, move |test_mode| {
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
                    (false, Ok(_)) => run_benchmark(case.test_fn),
                }
            })
            .with_kind(kind_str);
            apply_ignore_flag(trial, case)
        }
    }
}
