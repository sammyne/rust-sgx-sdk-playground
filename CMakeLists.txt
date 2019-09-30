cmake_minimum_required(VERSION 3.10)

project(rust-sgx-sdk-playground)

include(cmake/sgx.cmake)

add_subdirectory(hello-world)
add_subdirectory(crypto)