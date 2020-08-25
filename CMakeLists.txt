cmake_minimum_required(VERSION 3.10)

project(rust-sgx-sdk-playground)

include(ExternalProject)

#include(cmake/git.cmake)
include(cmake/sgx.cmake)

#add_subdirectory(crypto)
add_subdirectory(dns)
#add_subdirectory(hello-world)
#add_subdirectory(hello-world-debug)
#add_subdirectory(hello-world-i-dont-like)
#add_subdirectory(https-client)
#add_subdirectory(massive-stdio)
#add_subdirectory(mini-ecdh)
#add_subdirectory(prost-based-protobuf)
#add_subdirectory(tls-server)
#add_subdirectory(ue-remote-attestation)
