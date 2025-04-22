# Rust CTest Integration

This project demonstrates how to build a Rust test suite as a native executable and run it either via CTest (CMake) or directly with Cargo.

## 🔧 Requirements

- CMake >= 3.16
- Rust (with `cargo`)
- A C++ compiler

## 🚀 Run Tests

You can run the Rust tests using either **Cargo** directly or **CTest** through CMake:

### Option 1: Run Tests with Cargo

```bash
cd rust_test_export
cargo run --bin run_tests --features tests
```

### Option 2: Run Tests with CTest

```bash
mkdir build
cd build
cmake ..
cmake --build .
ctest --output-on-failure
```

