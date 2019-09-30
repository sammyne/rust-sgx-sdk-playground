# Hello World to rust-sgx-sdk

## Docker playground
All examples can be built and run in simulation mode within docker containers provided by the 
official repository.

To speed up daily development, the [play.sh](./scripts/play.sh) can be used to bootstrap a ready
container for you, with a simple command

```bash
# this will mount ${PWD} into the /workspace directory within the container
./scripts/play.sh
```

## Preparation

1. Pull in the official [rust-sgx-sdk](https://github.com/baidu/rust-sgx-sdk) submodule
    ```bash
    git submodule update --init 
    ```

## Build 

```bash

# as many as you want
mkdir build 
cd build
# default is SIM mode
cmake ..
make -j
```

## Run

```bash
# the hello-world example
make hello-world-dev
```

## FYI
- In the hardware mode, please employ the specific rust toolchain tagged by `nightly-2019-08-01`
    ```bash
    rustup install nightly-2019-08-01
    rustup default nightly-2019-08-01
    ```
