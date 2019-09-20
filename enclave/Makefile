# Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
#  * Redistributions of source code must retain the above copyright
#    notice, this list of conditions and the following disclaimer.
#  * Redistributions in binary form must reproduce the above copyright
#    notice, this list of conditions and the following disclaimer in
#    the documentation and/or other materials provided with the
#    distribution.
#  * Neither the name of Baidu, Inc., nor the names of its
#    contributors may be used to endorse or promote products derived
#    from this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#
Rust_Enclave_Name := libenclave.a
Rust_Enclave_Files := $(wildcard src/*.rs)
Rust_Target_Path := $(CURDIR)/../../../xargo

include ../conf.mk

######## Enclave Settings ########

ifneq ($(SGX_MODE), HW)
  Trts_Library_Name := sgx_trts_sim
  Service_Library_Name := sgx_tservice_sim
else
  Trts_Library_Name := sgx_trts
  Service_Library_Name := sgx_tservice
endif
Crypto_Library_Name := sgx_tcrypto
KeyExchange_Library_Name := sgx_tkey_exchange
ProtectedFs_Library_Name := sgx_tprotected_fs

RustEnclave_C_Files := $(wildcard ./*.c)
RustEnclave_C_Objects := $(RustEnclave_C_Files:.c=.o)
RustEnclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH) -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I./ 

RustEnclave_Link_Libs := -L$(CUSTOM_LIBRARY_PATH) -lcompiler-rt-patch -lenclave
RustEnclave_Compile_Flags := $(SGX_COMMON_CFLAGS) -nostdinc -fvisibility=hidden -fpie -fstack-protector $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags := $(SGX_COMMON_CFLAGS) -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -l$(Service_Library_Name) -l$(Crypto_Library_Name) $(RustEnclave_Link_Libs) -Wl,--end-group \
	-Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
	-Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
	-Wl,--defsym,__ImageBase=0 \
	-Wl,--gc-sections \
	-Wl,--version-script=./Enclave.lds

RustEnclave_Name := ./enclave.so
Signed_RustEnclave_Name := enclave.signed.so

.PHONY: all
#all: $(App_Name) $(Signed_RustEnclave_Name)
all: $(Signed_RustEnclave_Name) 

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) ./Enclave.edl
	$(SGX_EDGER8R) --trusted ./Enclave.edl --search-path $(SGX_SDK)/include \
	 	--search-path ${rustSGXPath}/edl 
	@echo "GEN  =>  $(Enclave_EDL_Files)"

######## Enclave Objects ########

./Enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_Compile_Flags) -c ./Enclave_t.c -o $@
	@echo "CC   <=  $<"

$(RustEnclave_Name): ${Rust_Enclave_Name} compiler-rt ./Enclave_t.o
	@$(CXX) ./Enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK =>  $@"

$(Signed_RustEnclave_Name): $(RustEnclave_Name)
	mkdir -p bin
	@$(SGX_ENCLAVE_SIGNER) sign -key isv.pem -enclave $(RustEnclave_Name) -out $@ -config ./Enclave.config.xml
	@mv $(Signed_RustEnclave_Name) ../bin
	@echo "SIGN =>  $@"

.PHONY: compiler-rt
compiler-rt:
	$(MAKE) -C ${rustSGXPath}/compiler-rt/ 2> /dev/null
	cp ${rustSGXPath}/compiler-rt/libcompiler-rt-patch.a ../lib

.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(Signed_RustEnclave_Name) ./*_t.* app/*_u.* lib/*.a
	@cargo clean 

$(Rust_Enclave_Name): 
	cargo build --release
	cp ./target/release/libhelloworldsampleenclave.a ../lib/libenclave.a
#ifeq ($(XARGO_SGX), 1)
#	RUST_TARGET_PATH=$(Rust_Target_Path) xargo build --target x86_64-unknown-linux-sgx --release
#	cp ./target/x86_64-unknown-linux-sgx/release/libhelloworldsampleenclave.a ../lib/libenclave.a
#else
#	cargo build --release
#	cp ./target/release/libhelloworldsampleenclave.a ../lib/libenclave.a
#endif