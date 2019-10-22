# TLS server within enclave

## Run

> This example requires hardwares supporting SGX

### Server 
```bash
# in the project root, i.e., rust-sgx-sdk-playground

## start a docker container with rust-sgx-sdk configured
./scripts/play.sh

mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j

## start the server
make run-ue-remote-attestation
```

### Client
```bash
# in your host machine with golang>=1.13 installed

cd client-go
go run .
```