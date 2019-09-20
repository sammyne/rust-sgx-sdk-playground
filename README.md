# Hello World to rust-sgx-sdk

## Build 

```bash
# only once
git submodule update --init 

# as many as you want
mkdir build 
cd build
# default is SIM mode
cmake ..
make -j
```

## Run

```bash
make dev
```

## Head Ups
- remove app/target and enclave/target for fresh build