cmake_minimum_required(VERSION 3.15)

ExternalProject_Add(teaclave-sgx-sdk
    GIT_REPOSITORY https://gitee.com/sammyne/incubator-teaclave-sgx-sdk
    GIT_TAG v1.1.2
    GIT_PROGRESS true
    SOURCE_DIR ${PROJECT_SOURCE_DIR}/third_party/teaclave-sgx-sdk
    UPDATE_DISCONNECTED true
    CONFIGURE_COMMAND echo "skip configure for teaclave-sgx-sdk"
    BUILD_COMMAND echo "skip build for teaclave-sgx-sdk"
    INSTALL_COMMAND echo "skip install for teaclave-sgx-sdk"
)
