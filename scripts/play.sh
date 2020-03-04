#!/bin/bash

# this script should be run in the project root, i.e., rust-sgx-sdk-playground

# --privileged is to enable GDB

docker run -it --rm -v ${PWD}:/workspace \
    -v ${PWD}/scripts/Cargo.config:/root/.cargo/config \
    -w /workspace \
    -p 4433:4433 \
    --privileged \
    hub.baidubce.com/jpaas-public/baiduxlab-sgx-rust:1804-1.1.0
#    baiduxlab/sgx-rust:1804-1.1.0 bash
