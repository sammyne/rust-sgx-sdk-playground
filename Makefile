
all: makeApp makeEnclave 
.PHONY: makeApp makeEnclave clean

makeApp:
	make -C ./app/

makeEnclave:
	make -C ./enclave/

clean:
	@make -C app/ clean
	@make -C enclave/ clean
	@rm -f lib/*.a bin/*.so bin/app