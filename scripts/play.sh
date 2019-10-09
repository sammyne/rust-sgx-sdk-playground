#!/bin/bash

# this script should be run in the project root, i.e., rust-sgx-sdk-playground

docker run -it -v ${PWD}:/workspace \
    -v ${PWD}/scripts/Cargo.config:/root/.cargo/config \
    -w /workspace \
    baiduxlab/sgx-rust:1804-1.0.9 bash