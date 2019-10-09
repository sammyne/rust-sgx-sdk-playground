cmake_minimum_required(VERSION 3.10)

set(appBin ${CMAKE_CURRENT_BINARY_DIR}/cargo/release/app)

# redirect target directory into ${CMAKE_CURRENT_BINARY_DIR}/cargo to avoid mess up the source
# directory
add_custom_target(${app} ALL
    COMMAND SGX_SDK=${sgxPath} SGX_MODE=${SGX_MODE} 
        cargo build --release --target-dir=${CMAKE_CURRENT_BINARY_DIR}/cargo
    COMMENT "[CARGO] app => ${appBin}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(${PROJECT_NAME}-dev
    ${appBin} ${PROJECT_BINARY_DIR}/enclave/${enclaveLib} ${PROJECT_SOURCE_DIR}/pki/end.fullchain 
        ${PROJECT_SOURCE_DIR}/pki/end.rsa
    DEPENDS ${app} ${enclave}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)