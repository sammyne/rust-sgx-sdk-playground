cmake_minimum_required(VERSION 3.10)

set(appBin ${CMAKE_CURRENT_BINARY_DIR}/cargo/release/app)

# redirect target directory into ${CMAKE_CURRENT_BINARY_DIR}/cargo to avoid mess up the source
# directory
add_custom_target(${app} ALL
    COMMAND SGX_SDK=${sgxPath} SGX_MODE=${SGX_MODE} RUST_SGX_SDK=${rsgxPath}
        cargo build --release --target-dir=${CMAKE_CURRENT_BINARY_DIR}/cargo
    COMMENT "[CARGO] app => ${appBin}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(format-${app}
    cargo fmt
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_target(run-${PROJECT_NAME}
    ${appBin} ${PROJECT_BINARY_DIR}/enclave/${enclaveLib}
    DEPENDS ${app} ${enclave}
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
)
