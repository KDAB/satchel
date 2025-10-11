# Satchel

## Overview

Satchel is a flexible test collection framework for Rust projects. It allows you to register tests and benchmarks using custom macros (`#[test]`, `#[bench]`), and provides a programmable interface for discovering and collecting tests outside of Rust’s default test runner.

Satchel is ideal for advanced integration scenarios, such as:
- Integrating Rust test execution into C++ projects via CMake/CTest,
- Running Rust tests from other languages or environments,
- Collecting and filtering test metadata for custom reporting.

Satchel uses distributed slices using the [linkme crate](https://crates.io/crates/linkme) to register test cases at compile time, and exposes APIs to enumerate and run tests programmatically. This makes it easy to build your own test harness, integrate with external tools, or embed Rust testing into larger polyglot projects.

Please note that platform support depends on the platforms supported by the [linkme crate](https://crates.io/crates/linkme).
See the linkme README for supported platforms.

## Project Structure

```plaintext
crates/
  satchel/                 # Core library for Rust test registration/discovery
  satchel-macro/           # Procedural macro for #[test] and #[bench]
examples/
  test-runner/             # Shared test runner crate for all examples
  ctest-integration/       # Example C++ project using CTest to run Rust tests
    somelib/               # Example Rust library with tests
    otherlib/              # Another Rust library with tests
  rust-examples/           # Pure Rust examples using custom test harnesses
    satchel-demo/          # Demonstrates satchel test registration/discovery
Cargo.toml                 # Cargo workspace manifest
```

## Rust-Only Examples

**satchel-demo/**
  Demonstrates how to use [`satchel`](crates/satchel/src/lib.rs) for automatic test registration and discovery in a pure Rust crate.
  Uses custom `#[test]` and `#[bench]` macros, distributed slices, and the shared test runner from `examples/test-runner`.
  Shows how to use `#[should_panic]` with expected panic messages and `#[ignore]` for tests that should be skipped by default.

### Supported Attribute Forms

Satchel mirrors many behaviors of Rust's built-in test attributes while remaining explicit about the supported forms:

`#[should_panic]` variants:

- `#[should_panic]` (accept any panic)
- `#[should_panic(expected = "substring")]` (panic message must contain substring)
- `#[should_panic = "substring"]` (shorthand for `expected =`)
- `#[should_panic("substring")]` (positional form)

`#[ignore]` variants:

- `#[ignore]` (skip test, no reason)
- `#[ignore = "reason"]` (skip test, track reason)

Unsupported forms produce a compile error emitted by the procedural macro (e.g. `#[ignore(foo)]`, `#[should_panic(bad = 1)]`).

## How It Works

**Test Registration:**
  Consumer crates (like `somelib`, `otherlib`, or `satchel-demo`) use the `#[test]` and `#[bench]` macros from Satchel to register test functions. These macros use [`linkme`](https://crates.io/crates/linkme) to collect test metadata into a distributed slice at compile time.

**Test Harness:**
  The test harness in `satchel` exposes a getter for all registered tests in the current crate.
  Consumer crates are responsible for providing a test harness and are free to choose any test harness they like.
  Our examples export a `*_tests_main` function that runs all tests using `libtest-mimic`.
  The example crates use the shared test runner from `examples/test-runner`, which provides a unified API for running tests and benchmarks.

**CTest Integration:**
  CMake builds the Rust libraries as `cdylib` and links them into the C++ test runner. The C++ main function calls the exported test entry points, and the results are reported to CTest.

## Adding Tests in a Consumer Crate

1. **Add Satchel as a Dependency:**

    ```toml
    [dependencies]
    satchel = { path = "../../../crates/satchel" }
    ```

2. **Write Tests Using the Macros:**

    ```rust
    use satchel::{test, bench};

    #[test]
    fn my_unit_test() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_panic_with_message() {
        panic!("integer overflow detected");
    }

    #[test]
    #[ignore]
    fn expensive_test() {
        assert_eq!(2 + 2, 4);
    }

    #[bench]
    fn my_benchmark() {
        for i in 0..1000 {
            let _ = i + 1;
        }
    }
    ```

3. **Export a Test Runner:**

  ```rust
  use libtest_mimic::Arguments;
  use test_runner;

  #[no_mangle]
  pub extern "C" fn some_tests_main() -> i32 {
    let tests = satchel::get_tests!().map(|t| *t).collect::<Vec<_>>();
    let args = Arguments::from_args();
    if test_runner::run_tests(tests, args) { 0 } else { 1 }
  }
  ```

4. **Implement `run_tests`:**

See [`examples/ctest-integration/somelib/src/lib.rs`](examples/ctest-integration/somelib/src/lib.rs) for a full example, including support for `#[should_panic]` and expected panic messages.

## Building and Running the Example

```bash
cd examples/ctest-integration
cmake --preset ctest-example
cmake --build build-ctest-example
cd build-ctest-example
ctest
```

## Running Rust-Only Examples

To run the pure Rust examples:

```bash
cargo test
```

Or to run a specific example crate:

```bash
cargo test --package satchel_demo --test satchel_demo -- tests --show-output
```

To run ignored tests:

```bash
cargo test --package satchel_demo --test satchel_demo -- --ignored
```

To run all tests including ignored ones:

```bash
cargo test --package satchel_demo --test satchel_demo -- --include-ignored
```

### Running Macro Compile Tests

```bash
cargo test -p satchel --test compile_fail -- --nocapture
```

```bash
cargo test -p satchel --test pass
```

If diagnostics intentionally changes:

```bash
TRYBUILD=overwrite cargo test -p satchel --test compile_fail
```

To diff without overwriting:

```bash
TRYBUILD=diff cargo test -p satchel --test compile_fail
```

## Using cargo nextest

Satchel supports nextest with a custom harness based on libtest-mimic. Our demo crate `satchel-demo` implements the minimal contract described in Nextest’s docs (see: https://nexte.st/docs/design/custom-test-harnesses/):

- The test binary supports `--list --format terse` and prints one `name: kind` per line to stdout.
- It also supports `--list --format terse --ignored` to print only ignored tests.
- Each test can be invoked via `<crate/module prefix::test-name> --exact`.

Naming and exact matching:

- The list output matches the runtime name exactly so `--exact` works without surprises.
- Ignore reasons, if any, are printed in the kind field (e.g. `test [ignored: reason] name ... ignored`), not in the name.

Listing behavior:

- By default, `--list --format terse` prints only non-ignored tests (matching nextest’s default list).
- With `--ignored`, the harness prints only ignored tests.

Examples:

```bash
# List all tests discovered by nextest for the demo crate
cargo nextest list -p satchel_demo

# List only ignored tests
cargo nextest list -p satchel_demo --run-ignored=only

# Run all ignored tests
cargo nextest run -p satchel_demo --run-ignored=only

# Run a single test by exact name
cargo nextest run -p satchel_demo -- satchel_demo::tests::test_multiply_positive --exact

# Run a single ignored test by exact name
cargo nextest run -p satchel_demo --run-ignored=only -- satchel_demo::tests::test_ignored_simple --exact

# Run a single test without capture (nextest flag goes before `--`)
cargo nextest run -p satchel_demo --no-capture -- satchel_demo::tests::test_multiply_positive --exact

# Run one test by substring (without --exact)
cargo nextest run -p satchel_demo -- tests::test_multiply_positive
```

Notes:

- If you change how runtime names are constructed in the runner, ensure the `--list` output names stay exactly in sync, or `--exact` will fail to match.

## Known Issues

**Tests may be optimized out in staticlib builds:**
When building a client crate with `crate-type = ["staticlib"]`, test functions registered via `#[linkme::distributed_slice]` may be optimized out by the compiler in certain build profiles (especially debug), because they are not explicitly referenced. This results in tests silently not running or being excluded from the final binary.

### Workarounds

**Use cdylib instead of staticlib:**
  When using `crate-type = ["cdylib"]`, you're telling Cargo to build a C-compatible dynamic library, which causes the Rust compiler and linker to preserve all `#[no_mangle]` and exported symbols (and also all statics with internal linkage), because it assumes they might be used externally (e.g., from C or via dlsym).

```toml
[lib]
crate-type = ["cdylib"]
```
