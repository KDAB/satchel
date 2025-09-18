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
  satchel/                 # Core library for the Rust tests and test harness
  satchel-macro/           # Custom procedural macro for #[test] and #[bench]
examples/
  ctest-integration/       # Example C++ project using CTest to run Rust tests
    somelib/               # Example Rust library with tests
    otherlib/              # Another Rust library with tests
  rust-examples/           # Pure Rust examples using custom test harnesses
    satchel-demo/          # Demonstrates satchel test registration/discovery
Cargo.toml                 # Cargo workspace manifest
```

## Rust-Only Examples

- **satchel-demo/**  
  Demonstrates how to use [`satchel`](crates/satchel/src/lib.rs) for automatic test registration and discovery in a pure Rust crate.  
  Uses custom `#[test]` and `#[bench]` macros, distributed slices, and a programmable test runner.  
  Includes a helper function for test discovery and a single entry point for running all tests.

## How It Works

- **Test Registration:**  
  Consumer crates (like `somelib` or `otherlib`) use the `#[test]` and `#[bench]` macros from Satchel to register test functions. These macros use [`linkme`](https://crates.io/crates/linkme) to collect test metadata into a distributed slice at compile time.

- **Test Harness:**  
  The test harness in `satchel` exposes a getter for all registered tests in the current crate.
  Consumer crates are responsible for providing a test harness and are free to choose any test harness they like.
  Our examples export a `*_tests_main` function that runs all tests using `libtest-mimic`.

- **CTest Integration:**  
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

    #[bench]
    fn my_benchmark() {
        for i in 0..1000 {
            let _ = i + 1;
        }
    }
    ```

3. **Export a Test Runner:**
    ```rust
    #[no_mangle]
    pub extern "C" fn some_tests_main() -> i32 {
        let tests = satchel::get_tests!();
        let args = libtest_mimic::Arguments::from_args();
        if run_tests(tests, args) { 0 } else { 1 }
    }
    ```

4. **Implement `run_tests`:**
    See [`examples/ctest-integration/somelib/src/lib.rs`](examples/ctest-integration/somelib/src/lib.rs) for a full example.

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

## Known Issues

**Tests may be optimized out in staticlib builds:**  
When building a client crate with `crate-type = ["staticlib"]`, test functions registered via `#[linkme::distributed_slice]` may be optimized out by the compiler in certain build profiles (especially debug), because they are not explicitly referenced. This results in tests silently not running or being excluded from the final binary.

### Workarounds

- **Use cdylib instead of staticlib:**  
  When using `crate-type = ["cdylib"]`, you're telling Cargo to build a C-compatible dynamic library, which causes the Rust compiler and linker to preserve all `#[no_mangle]` and exported symbols (and also all statics with internal linkage), because it assumes they might be used externally (e.g., from C or via dlsym).

    ```toml
    [lib]
    crate-type = ["cdylib"]
    ```
