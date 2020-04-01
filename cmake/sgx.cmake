cmake_minimum_required(VERSION 3.10)

# -ggdb: enable debugging with GDB
if(CMAKE_BUILD_TYPE STREQUAL "Debug" OR CMAKE_BUILD_TYPE STREQUAL "")
   set(sgxFlags "-m64 -ggdb -O0 -g")
   set(enclaveLibSuffix _sim)
   set(SGX_MODE SW)
elseif(CMAKE_BUILD_TYPE STREQUAL "Prerelease")
   set(sgxFlags "-g -O2")
   set(SGX_MODE HW)
elseif(CMAKE_BUILD_TYPE STREQUAL "Release")
   set(sgxFlags "-g -O2")
   set(SGX_MODE HW)
else()
   message(FATAL_ERROR "unknown build type: ${CMAKE_BUILD_TYPE}")
endif()

set(sgxPath /opt/intel/sgxsdk)
set(sgxLibPath ${sgxPath}/lib64)

# tools
set(sgxEdger8r ${sgxPath}/bin/x64/sgx_edger8r)
set(sgxSigner ${sgxPath}/bin/x64/sgx_sign)

# rust-sgx-sdk
set(rsgxPath ${PROJECT_SOURCE_DIR}/third_party/teaclave-sgx-sdk)

message(STATUS "SGX_MODE=${SGX_MODE}")
