cmake_minimum_required(VERSION 3.10)

project(hello-rust-sgx)

include(cmake/sgx.cmake)

add_subdirectory(enclave)
add_subdirectory(app)