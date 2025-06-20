cmake_minimum_required(VERSION 3.20)
project(ctest_integration)
set(CMAKE_CXX_STANDARD 17)

find_package(Corrosion 0.5.1 QUIET)
if(NOT Corrosion_FOUND)
    include(FetchContent)
    FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.5.1
    )

    FetchContent_MakeAvailable(Corrosion)
endif()

corrosion_import_crate(
    MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    CRATES somelib
    )

add_executable(run_rust_tests main.cpp)
target_link_libraries(run_rust_tests PRIVATE somelib)

enable_testing()
add_test(NAME RustTests COMMAND run_rust_tests)
