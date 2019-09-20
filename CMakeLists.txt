cmake_minimum_required(VERSION 3.10)

project(hello-rust-sgx)

include(cmake/sgx.cmake)

set(rustSGXPath /rust-sgx-sdk)

set(rtsLib sgx_trts${enclaveLibSuffix})
set(serviceLib sgx_tservice${enclaveLibSuffix})

set(cLib sgx_tstdc)
set(cxxLib sgx_tcxx)
set(cryptoLib sgx_tcrypto)

set(addonLibPath ${CMAKE_CURRENT_SOURCE_DIR}/lib)

include_directories(
    ${CMAKE_CURRENT_BINARY_DIR}
    ${sgxPath}/include 
    ${sgxPath}/include/tlibc
    ${sgxPath}/include/stlport
    ${sgxPath}/include/epid
    ${rustSGXPath}/common/inc
    ${rustSGXPath}/edl
)

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -nostdinc -fvisibility=hidden -fpie -ffunction-sections \
	-fdata-sections -fstack-protector-strong")
set(CMAKE_CXX_FLAGS "${CMAKE_C_FLAGS} ${sgxFlags} \
    -nostdinc++ -fvisibility=hidden -fpie -ffunction-sections \
	-fdata-sections -fstack-protector-strong -std=c++11")

# stronger security: -Wl,-z,relro,-z,now,-z,noexecstack
# be careful about the no-ASCII "-" when copying from dev-ref
set(LDFLAGS "-Wl,-pie,-eenclave_entry \
	-nostdlib -nodefaultlibs -nostartfiles -Wl,--no-undefined \
	-L${sgxLibPath} \
	-Wl,--whole-archive -l${rtsLib} -Wl,--no-whole-archive \
	-Wl,--start-group -l${cLib} -l${serviceLib} -l${cryptoLib} -l${cxxLib} \
    -L${addonLibPath} -lcompiler-rt-patch -lenclave \
    -Wl,--end-group \
	-Wl,-Bstatic -Wl,-Bsymbolic -Wl,--defsym=__ImageBase=0 -Wl,--export-dynamic
    -Wl,--defsym,__ImageBase=0 \
	-Wl,--gc-sections \
	-Wl,--version-script=enclave/enclave.lds")

set(bridgeImpl ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_t.c) 
set(bridge ${CMAKE_CURRENT_BINARY_DIR}/${enclave}_t.h ${bridgeImpl})

set(srcs ${bridgeImpl})

# build the raw lib
add_library(${enclave} SHARED ${srcs})
target_link_libraries(${enclave} ${LDFLAGS})

add_custom_command(
    OUTPUT ${bridge}
    COMMAND ${sgxEdger8r} --trusted ${edl} 
       --search-path ${enclavePath} --search-path ${sgxPath}/include
       --search-path ${rustSGXPath}/edl
    COMMENT "[GEN] => ${bridge}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})

add_custom_command(TARGET ${enclave} PRE_BUILD
    BYPRODUCTS libcompiler-rt-patch.a
    COMMAND make -C ${rustSGXPath}/compiler-rt 
    COMMAND cp ${rustSGXPath}/compiler-rt/libcompiler-rt-patch.a ${CMAKE_CURRENT_SOURCE_DIR}/lib
    COMMENT "[PACK] compiler runtime for rust-sgx-sdk"
)

add_custom_command(TARGET ${enclave} PRE_BUILD
    BYPRODUCTS libenclave.a
    COMMAND make -C ${enclavePath}
    COMMAND cp ${enclavePath}/libenclave.a ${CMAKE_CURRENT_SOURCE_DIR}/lib
    COMMENT "[PACK] enclave"
)

add_custom_command(TARGET ${enclave} POST_BUILD
    BYPRODUCTS ${enclaveLib} 
    COMMAND ${sgxSigner} sign -key ${key} -enclave lib${enclave}.so 
        -out ${enclaveLib} -config ${config}
    COMMENT "[SIGN] lib${enclave}.so => ${enclaveLib}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
