
all: makeApp makeEnclave 
.PHONY: makeApp makeEnclave clean

export SGX_MODE=SW

makeApp:
	make -C ./app/

makeEnclave:
	make -C ./enclave/

dev: makeApp makeEnclave
	cd ./bin && ./app

clean:
	@make -C app/ clean
	@make -C enclave/ clean
	@rm -f lib/*.a bin/*.so bin/app