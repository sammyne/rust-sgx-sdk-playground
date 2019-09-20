cmake_minimum_required(VERSION 3.10)

set(rustSGXPath /rust-sgx-sdk)
set(uAppLib Enclave_u)

# must
set(rtsLib sgx_urts${enclaveLibSuffix})

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -fPIC -Wno-attributes")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_u.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_u.h ${bridgeImpl})

include_directories(
    ${rustSGXPath}/edl
    ${sgxPath}/include 
    ${CMAKE_CURRENT_BINARY_DIR}
)

add_custom_command(
    OUTPUT ${bridge}
    COMMAND ${sgxEdger8r} --untrusted ${edl} 
       --search-path ${enclavePath} --search-path ${sgxPath}/include
       --search-path ${rustSGXPath}/edl
    COMMENT "[GEN] => ${bridge}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

add_library(${uAppLib} STATIC ${bridgeImpl})

add_custom_command(TARGET ${uAppLib} POST_BUILD
    COMMAND cp ${CMAKE_CURRENT_BINARY_DIR}/lib${uAppLib}.a ${CMAKE_SOURCE_DIR}/lib
)

add_custom_target(app ALL
    COMMAND SGX_SDK=${sgxPath} cargo build --release
    COMMAND cp ${CMAKE_CURRENT_SOURCE_DIR}/target/release/app ${CMAKE_SOURCE_DIR}/bin
    BYPRODUCTS app
    COMMENT "[GEN] cargo => app"
    DEPENDS ${uAppLib}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)