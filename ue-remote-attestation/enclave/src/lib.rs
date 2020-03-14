#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;

mod attestation;
mod crypto;
mod ecall;
mod hex;
mod x509;
