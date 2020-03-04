cmake_minimum_required(VERSION 3.10)

set(rtsLib sgx_trts${enclaveLibSuffix})
set(serviceLib sgx_tservice${enclaveLibSuffix})

set(cLib sgx_tstdc)
set(cxxLib sgx_tcxx)
set(cryptoLib sgx_tcrypto)

set(addonLibPath ${CMAKE_CURRENT_BINARY_DIR})
set(rustEnclaveLib "enclave_rust")

## rust sdk must come before official SGX
include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}
    ${rsgxPath}/common/inc
    ${rsgxPath}/edl
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
    -L${addonLibPath} -l${rustEnclaveLib} \
    -Wl,--end-group \
    -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined -Wl,-pie,-eenclave_entry \
    -Wl,--export-dynamic -Wl,--defsym,__ImageBase=0 -Wl,--gc-sections \
    -Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/enclave.lds")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/enclave_t.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/enclave_t.h ${bridgeImpl})

set(srcs ${bridgeImpl})

add_custom_command(
    OUTPUT ${bridge}
    COMMAND ${sgxEdger8r} --trusted ${edl} 
       --search-path ${enclavePath} --search-path ${sgxPath}/include
       --search-path ${rsgxPath}/edl
    COMMENT "[GEN] => ${bridge}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

# build the raw lib
add_library(${enclave} SHARED ${srcs})
target_link_libraries(${enclave} ${LDFLAGS})

add_custom_command(TARGET ${enclave} PRE_BUILD
    COMMAND SGX_MODE=${SGX_MODE} cargo build --release 
        --target-dir ${CMAKE_CURRENT_BINARY_DIR}/cargo
    COMMAND mv ${CMAKE_CURRENT_BINARY_DIR}/cargo/release/libenclave.a 
        ${CMAKE_CURRENT_BINARY_DIR}/lib${rustEnclaveLib}.a
    COMMENT "[PACK] enclave"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_custom_command(TARGET ${enclave} POST_BUILD
    BYPRODUCTS ${enclaveLib} 
    COMMAND ${sgxSigner} sign -key ${key} -enclave lib${enclave}.so 
        -out ${enclaveLib} -config ${config}
    COMMENT "[SIGN] lib${enclave}.so => ${enclaveLib}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

add_custom_target(format-${enclave}
    cargo fmt
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)
