cmake_minimum_required(VERSION 3.10)
project(ctest_integration)
set(CMAKE_CXX_STANDARD 17)

add_executable(run_rust_tests main.cpp)
target_link_directories(run_rust_tests PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../target/release)
target_link_libraries(run_rust_tests PRIVATE somelib)

enable_testing()
add_test(NAME RustTests COMMAND run_rust_tests)
