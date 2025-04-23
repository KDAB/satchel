# Satchel - Rust and CTest Integration

## Overview

Satchel provides a framework for integrating Rust tests into a C++ project using `CTest`. It uses Rust's test harness combined with a custom `#[test]` macro to allow tests to be executed via CMake and CTest.

This project sets up a test harness in Rust, and provides a mechanism to call Rust test functions from C++ via a `libsatchel` library, utilizing `libtest_mimic` for running tests and generating test results.

## Project Structure

```plaintext
crates/
  satchel/                 # Core library for the Rust tests
  satchel-macro/           # Custom procedural macro for the #[test] attribute
examples/
  ctest-integration/       # Example C++ project using CTest to run Rust tests
Cargo.toml                 # Cargo workspace manifest
```
