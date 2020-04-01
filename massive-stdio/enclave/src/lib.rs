#![no_std]

//extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

//use sgx_types::*;
use std::{slice, str};

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };

    //println!("string length: {}", str_slice.len());

    match str::from_utf8(str_slice) {
        Ok(v) => println!("string: {}", v),
        Err(err) => {
            panic!("failed to parse param from data: {:?}", err);
        }
    }
}
