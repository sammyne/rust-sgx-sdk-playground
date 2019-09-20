cmake_minimum_required(VERSION 3.10)

set(rustSGXPath /rust-sgx-sdk)

set(rtsLib sgx_trts${enclaveLibSuffix})
set(serviceLib sgx_tservice${enclaveLibSuffix})

set(cLib sgx_tstdc)
set(cxxLib sgx_tcxx)
set(cryptoLib sgx_tcrypto)
set(fsLib sgx_tprotected_fs)
set(kxLib sgx_tkey_exchange)

#set(addonLibPath ${CMAKE_SOURCE_DIR}/lib)
set(addonLibPath ${CMAKE_CURRENT_BINARY_DIR})
set(rustEnclaveLib "enclave_rust")

## rust sdk must come before official SGX
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}
    ${rustSGXPath}/common/inc
    ${rustSGXPath}/edl
    ${sgxPath}/include 
    ${sgxPath}/include/tlibc
    ${sgxPath}/include/stlport
    ${sgxPath}/include/epid
)

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -nostdinc -fvisibility=hidden -fpie -ffunction-sections \
	-fdata-sections -fstack-protector-strong")
set(CMAKE_CXX_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -nostdinc++ -fvisibility=hidden -fpie -ffunction-sections \
	-fdata-sections -fstack-protector-strong -std=c++11")

# stronger security: -Wl,-z,relro,-z,now,-z,noexecstack
# be careful about the no-ASCII "-" when copying from dev-ref
set(LDFLAGS "-Wl,--no-undefined \
	-nostdlib -nodefaultlibs -nostartfiles \
	-L${sgxLibPath} \
	-Wl,--whole-archive -l${rtsLib} -Wl,--no-whole-archive \
	-Wl,--start-group -l${cLib} -l${serviceLib} -l${cryptoLib} -l${cxxLib} \
    -L${addonLibPath} -lcompiler-rt-patch -l${rustEnclaveLib} \
    -Wl,--end-group \
    -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined -Wl,-pie,-eenclave_entry \
    -Wl,--export-dynamic -Wl,--defsym,__ImageBase=0 -Wl,--gc-sections \
    -Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/enclave.lds")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_t.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_t.h ${bridgeImpl})

set(srcs ${bridgeImpl})

add_custom_command(
    OUTPUT ${bridge}
    COMMAND ${sgxEdger8r} --trusted ${edl} 
       --search-path ${enclavePath} --search-path ${sgxPath}/include
       --search-path ${rustSGXPath}/edl
    COMMENT "[GEN] => ${bridge}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

# build the raw lib
add_library(${enclave} SHARED ${srcs})
target_link_libraries(${enclave} ${LDFLAGS})

add_custom_command(TARGET ${enclave} PRE_BUILD
    BYPRODUCTS ${CMAKE_SOURCE_DIR}/libcompiler-rt-patch.a
    COMMAND SGX_MODE=${SGX_MODE} make -C ${rustSGXPath}/compiler-rt 
    COMMAND cp ${rustSGXPath}/compiler-rt/libcompiler-rt-patch.a 
        ${CMAKE_CURRENT_BINARY_DIR}
    COMMENT "[PACK] compiler runtime for rust-sgx-sdk"
)

add_custom_command(TARGET ${enclave} PRE_BUILD
    COMMAND SGX_MODE=${SGX_MODE} cargo build --release
    COMMAND cp ${enclavePath}/target/release/libhelloworldsampleenclave.a 
        ${CMAKE_CURRENT_BINARY_DIR}/lib${rustEnclaveLib}.a
    COMMENT "[PACK] enclave"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

#    COMMAND cp ${enclaveLib} ${CMAKE_CURRENT_BINARY_DIR}
add_custom_command(TARGET ${enclave} POST_BUILD
    BYPRODUCTS ${enclaveLib} 
    COMMAND ${sgxSigner} sign -key ${key} -enclave lib${enclave}.so 
        -out ${enclaveLib} -config ${config}
    COMMENT "[SIGN] lib${enclave}.so => ${enclaveLib}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

## additional folder by Rust to remove
# must use absolute path
set_property(DIRECTORY APPEND PROPERTY ADDITIONAL_MAKE_CLEAN_FILES 
    "${CMAKE_CURRENT_SOURCE_DIR}/target"
)