cmake_minimum_required(VERSION 3.10)

project(hello-rust-sgx)

include(cmake/sgx.cmake)

add_subdirectory(enclave)
add_subdirectory(app)

#add_custom_target(makeApp ALL make -C ./app
#    COMMENT "[GEN] app"
#    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
#)

add_custom_target(doClean
    rm -rf bin/app bin/enclave.signed.so lib/*.a  app/target enclave/target
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)
