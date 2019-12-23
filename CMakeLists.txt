cmake_minimum_required(VERSION 3.10)

project(rust-sgx-sdk-playground)

include(cmake/sgx.cmake)

#add_subdirectory(crypto)
#add_subdirectory(hello-world)
#add_subdirectory(hello-world-without-cmake)
add_subdirectory(https-client)
#add_subdirectory(tls-server)
#add_subdirectory(ue-remote-attestation)