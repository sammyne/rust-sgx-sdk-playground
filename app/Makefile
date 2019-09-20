
include ../conf.mk

######## APP Settings ########

App_Rust_Flags := --release
App_SRC_Files := $(shell find ./ -type f -name '*.rs') $(shell find ./ -type f -name 'Cargo.toml')
App_Include_Paths := -I./ -I$(SGX_SDK)/include -I$(CUSTOM_EDL_PATH)
App_C_Flags := $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)

App_Rust_Path := ./target/release
App_Enclave_u_Object :=./libEnclave_u.a
App_Name := ../bin/app

all: ${App_Name}

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) ../enclave/enclave.edl
	$(SGX_EDGER8R) --untrusted ../enclave/enclave.edl --search-path $(SGX_SDK)/include \
		--search-path ${rustSGXPath}/edl
	@echo "[app] GEN  =>  $(Enclave_EDL_Files)"


######## App Objects ########

./Enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(App_C_Flags) -c ./Enclave_u.c -o $@
	@echo "CC   <=  $<"

$(App_Enclave_u_Object): ./Enclave_u.o
	$(AR) rcsD $@ $^
	cp $(App_Enclave_u_Object) ../lib

$(App_Name): $(App_Enclave_u_Object) $(App_SRC_Files)
	@SGX_SDK=$(SGX_SDK) cargo build $(App_Rust_Flags)
	@echo "Cargo  =>  $@"
	mkdir -p ../bin
	cp $(App_Rust_Path)/app ../bin

.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(Signed_RustEnclave_Name) ./*_t.* app/*_u.* lib/*.a
	@cargo clean 