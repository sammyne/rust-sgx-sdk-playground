# TLS server within enclave

## Run
### Server 
```bash
# in the project root, i.e., rust-sgx-sdk-playground

## start a docker container with rust-sgx-sdk configured
./scripts/play.sh

mkdir build
cd build
cmake ..
make -j

## start the server
make tls-server-dev
```

### Client
```bash
# in your host machine with golang>=1.13 installed

cd tls-server/client
go run main.go
```