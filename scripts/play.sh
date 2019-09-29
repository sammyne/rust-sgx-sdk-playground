#!/bin/bash

docker run -it -v ${PWD}:/workspace -w /workspace baiduxlab/sgx-rust:1804-1.0.9 bash