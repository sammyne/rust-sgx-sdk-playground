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

1. Pull in the official [incubator-teaclave-sgx-sdk](https://github.com/incubator-teaclave-sgx-sdk.git) submodule
    ```bash
    git submodule update --init 
    ```

## Build 

```bash

# as many as you want
mkdir build 
cd build
# default is SIM mode. For hardware mode, use 'cmake -DCMAKE_BUILD_TYPE=Release ..'
cmake ..
make -j
```

## Run

```bash
# the hello-world example
make run-hello-world
```

## Examples

> The mitigration towards incubator-teaclave-sgx-sdk@v1.1.0 is WIP

|                   Project | Description                                                |
| ------------------------: | :--------------------------------------------------------- |
| hello-world-without-cmake | hello-world example with app built with cargo build script |
|                tls-server | a TLS server running within enclaves                       |

## FYI
- In the hardware mode, please employ the specific rust toolchain tagged by `nightly-2019-08-01`
    ```bash
    rustup install nightly-2019-08-01
    rustup default nightly-2019-08-01
    ```
- For errors of pattern as follow, it's because dependencies bring `std` into the `no_std` 
  environment. As for how to address this, check https://github.com/baidu/rust-sgx-sdk/issues/31
    ```bash
    error: duplicate lang item in crate `sgx_tstd`: `f32_runtime`.
    |
    = note: first defined in crate `std`.
    ```
- When generating the trusted and untrusted bridges, projects would need to search rust-sgx-sdk/edl
  for extra edl files. We should keep the these edl synchronized to the version of rust-sgx-sdk in
  use.
- `libcompiler-rt-patch.a` is to address a potential bug, **so it's optional**.

## Git Tips
### delete submodules
```bash
git submodule deinit ${path-to-submodule}
git rm --cached ${path-to-submodule}

rm -rf ${path-to-submodule}
```