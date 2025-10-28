use libtest_mimic::{Arguments, Failed, Trial};
use satchel::test_harness::TestCase;
use std::cell::Cell;
use std::panic;

thread_local! {
    static ACTIVE_CASE_ATTRIBUTES: Cell<&'static [&'static str]> = Cell::new(&[]);
}

struct CaseAttributesGuard {
    previous: &'static [&'static str],
}

impl Drop for CaseAttributesGuard {
    fn drop(&mut self) {
        ACTIVE_CASE_ATTRIBUTES.with(|cell| cell.set(self.previous));
    }
}

fn push_case_attributes(attributes: &'static [&'static str]) -> CaseAttributesGuard {
    ACTIVE_CASE_ATTRIBUTES.with(|cell| {
        let previous = cell.replace(attributes);
        CaseAttributesGuard { previous }
    })
}

fn run_with_case_attributes<F, R>(attributes: &'static [&'static str], f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = push_case_attributes(attributes);
    let result = f();
    drop(guard);
    result
}

fn invoke_test_fn(test_fn: fn(), attributes: &'static [&'static str]) -> std::thread::Result<()> {
    run_with_case_attributes(attributes, || panic::catch_unwind(|| (test_fn)()))
}

pub fn current_case_attributes() -> &'static [&'static str] {
    ACTIVE_CASE_ATTRIBUTES.with(|cell| cell.get())
}

#[cfg(test)]
mod tests {
    use super::{current_case_attributes, run_with_case_attributes};

    #[test]
    fn case_attributes_reset_to_previous() {
        assert!(current_case_attributes().is_empty());
        run_with_case_attributes(&["one", "--two=2"], || {
            assert_eq!(current_case_attributes(), &["one", "--two=2"]);
        });
        assert!(current_case_attributes().is_empty());
    }
}

pub fn run_tests(tests: impl Iterator<Item = &'static TestCase>, args: Arguments) -> bool {
    let trials: Vec<Trial> = tests.map(create_trial_for_case).collect();
    !libtest_mimic::run(&args, trials).has_failed()
}

fn format_test_name(case: &TestCase) -> String {
    let base_name = format!("{}::{}", case.module_path, case.name);
    case.ignore
        .as_ref()
        .and_then(|info| info.reason)
        .map(|reason| format!("{} (ignored: {})", base_name, reason))
        .unwrap_or(base_name)
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

fn run_benchmark(
    test_fn: fn(),
    case_attributes: &'static [&'static str],
) -> Result<Option<libtest_mimic::Measurement>, Failed> {
    use std::time::Instant;
    const N: u64 = 1000;
    let mut times = Vec::with_capacity(N as usize);

    run_with_case_attributes(case_attributes, || {
        for _ in 0..N {
            let start = Instant::now();
            test_fn();
            let elapsed = start.elapsed().as_nanos() as f64;
            times.push(elapsed);
        }
    });

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
            let case_attributes = case.case_attributes;
            let retry_on_failure = should_panic.is_none()
                && case_attributes
                    .iter()
                    .any(|attr| *attr == "retry_on_failure");
            let test_fn = case.test_fn;
            let trial = Trial::test(full_name, move || {
                let mut result = invoke_test_fn(test_fn, case_attributes);
                if retry_on_failure && result.is_err() {
                    result = invoke_test_fn(test_fn, case_attributes);
                }
                handle_unit_test(result, should_panic.clone())
            })
            .with_kind(kind_str);
            apply_ignore_flag(trial, case)
        }
        satchel::TestKind::Benchmark => {
            let case_attributes = case.case_attributes;
            let test_fn = case.test_fn;
            let trial = Trial::bench(full_name, move |test_mode| {
                let result = invoke_test_fn(test_fn, case_attributes);
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
                    (false, Ok(_)) => run_benchmark(test_fn, case_attributes),
                }
            })
            .with_kind(kind_str);
            apply_ignore_flag(trial, case)
        }
    }
}
