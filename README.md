# Satchel - Rust and CTest Integration

## Overview

Satchel provides a framework for integrating Rust tests into a C++ project using `CTest`. It uses Rust's test harness combined with a custom `#[test]` macro to allow tests to be executed via CMake and CTest.

This project sets up a test harness in Rust, and provides a mechanism to call Rust test functions from C++ via a `libsatchel` library, utilizing `libtest_mimic` for running tests and generating test results.

## Requirements

-- CMake >= 3.14
-- Rust (with `cargo`)
-- A C++ compiler

## Project Structure

```plaintext
crates/
  satchel/                 # Core library for the Rust tests
  satchel-macro/           # Custom procedural macro for the #[test] attribute
examples/
  ctest-integration/       # Example C++ project using CTest to run Rust tests
Cargo.toml                 # Cargo workspace manifest
```

## Build and run the ctest-integration example

```bash
cd examples/ctest-integration
cmake --preset gcc && cd build_ctest_gcc && ninja && ./run_rust_tests
```

## Known Issues
Tests may be optimized out in staticlib builds
When building a client crate with `crate-type = ["staticlib"]`, test functions registered via `#[linkme::distributed_slice]` may be optimized out by the compiler in certain build profiles (especially debug), because they are not explicitly referenced. This results in tests silently not running or being excluded from the final binary.

### Workarounds:
Use cdylib instead of staticlib
When using `crate-type = ["cdylib"]`, you're telling Cargo to build a C-compatible dynamic library, which causes the Rust compiler and linker to preserve all `#[no_mangle]`
and exported symbols (and also all statics with internal linkage), because it assumes they might be used externally (e.g., from C or via dlsym).

```toml
[lib]
crate-type = ["cdylib"]
```
Alternatively, use RUSTFLAGS="-C link-dead-code" during compilation to prevent dead code elimination:

```sh
RUSTFLAGS="-C link-dead-code" cargo build
```
