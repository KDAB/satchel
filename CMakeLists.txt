cmake_minimum_required(VERSION 3.15)
project(RustCTestIntegration C CXX)

enable_testing()

set(RUST_CRATE_DIR ${CMAKE_CURRENT_SOURCE_DIR})
set(RUST_BIN_NAME run_tests)
set(RUST_BIN_PATH ${RUST_CRATE_DIR}/target/debug/${RUST_BIN_NAME})

# Target to build Rust test binary
add_custom_target(
  build_rust_tests ALL
  COMMAND cargo build --features tests --bin ${RUST_BIN_NAME}
  WORKING_DIRECTORY ${RUST_CRATE_DIR}
  BYPRODUCTS ${RUST_BIN_PATH}
  COMMENT "Building Rust test binary"
  VERBATIM)

# Create a dummy target for the test itself
add_custom_target(
  rust_tests ALL
  COMMAND ${RUST_BIN_PATH}
  DEPENDS build_rust_tests
  COMMENT "Running Rust tests via CMake target")

# Register with CTest
add_test(NAME rust_tests COMMAND ${RUST_BIN_PATH})
