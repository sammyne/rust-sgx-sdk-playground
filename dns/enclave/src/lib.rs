#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

use std::ffi;
use std::net::ToSocketAddrs;

use sgx_types::*;

use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn resolve(host_and_port: *const c_char) -> sgx_status_t {
    let host_and_port = unsafe {
        ffi::CStr::from_ptr(host_and_port)
            .to_str()
            .expect("invalid c-string")
    };

    let addrs = host_and_port
        .to_socket_addrs()
        .expect("fail to convert as socket addresses");

    for v in addrs {
        println!("{:?}", v);
    }

    sgx_status_t::SGX_SUCCESS
}
