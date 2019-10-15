cmake_minimum_required(VERSION 3.10)

# must
set(rtsLib sgx_urts${enclaveLibSuffix})

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -fPIC -Wno-attributes")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/enclave_u.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/enclave_u.h ${bridgeImpl})

set(uBridgeLib ${PROJECT_NAME}-enclave_u)
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

set(appBin ${CMAKE_CURRENT_BINARY_DIR}/cargo/release/app)

# redirect target directory into ${CMAKE_CURRENT_BINARY_DIR}/cargo to avoid mess up the source
# directory
add_custom_target(${app} ALL
    COMMAND SGX_SDK=${sgxPath} SGX_MODE=${SGX_MODE} BRIDGE_LIB_PATH=${uBridgeLibPath} 
        cargo build --release --target-dir=${CMAKE_CURRENT_BINARY_DIR}/cargo
    COMMENT "[CARGO] app => ${appBin}"
    DEPENDS ${uBridgeLib}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(run-${PROJECT_NAME}
    ${appBin} ${CMAKE_CURRENT_BINARY_DIR}/../enclave/${enclaveLib}
    DEPENDS ${app} ${enclave}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)

add_custom_target(${PROJECT_NAME}-check
    cargo check
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)