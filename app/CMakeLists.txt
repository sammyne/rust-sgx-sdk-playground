cmake_minimum_required(VERSION 3.10)

set(rustSGXPath /rust-sgx-sdk)

# must
set(rtsLib sgx_urts${enclaveLibSuffix})

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -fPIC -Wno-attributes")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_u.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_u.h ${bridgeImpl})

set(uBridgeLib enclave_u)
set(uBridgeLibPath ${CMAKE_CURRENT_BINARY_DIR})

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

add_library(${uBridgeLib} STATIC ${bridgeImpl})

add_custom_target(app ALL
    COMMAND SGX_SDK=${sgxPath} SGX_MODE=${SGX_MODE} BRIDGE_LIB_PATH=${uBridgeLibPath} 
        cargo build --release
    COMMAND cp ${CMAKE_CURRENT_SOURCE_DIR}/target/release/app ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "[CARGO] app => ${CMAKE_CURRENT_BINARY_DIR}/app"
    DEPENDS ${uBridgeLib}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(dev
    ${CMAKE_CURRENT_BINARY_DIR}/app ${CMAKE_BINARY_DIR}/enclave/enclave.signed.so
    DEPENDS app enclave
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)

## add on clean by Rust
#get_directory_property(_oldToClean ADDITIONAL_MAKE_CLEAN_FILES)
#list(APPEND _oldToClean ${CMAKE_CURRENT_SOURCE_DIR}/target)
#set_directory_properties(PROPERTIES ADDITIONAL_MAKE_CLEAN_FILES "${_oldToClean}")

set_property(DIRECTORY APPEND PROPERTY ADDITIONAL_MAKE_CLEAN_FILES 
    "${CMAKE_CURRENT_SOURCE_DIR}/target"
)