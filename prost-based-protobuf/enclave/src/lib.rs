#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

extern crate bytes;
extern crate prost;
extern crate prost_types;

use sgx_types::*;
use std::io::{self, Write};
use std::slice;
use std::string::String;
use std::vec::Vec;

use prost::Message;
use prost_types::Timestamp;

mod person {
    include!(concat!(env!("OUT_DIR"), "/person.rs"));
}

#[no_mangle]
pub extern "C" fn say_something(msg: *const u8, msg_len: usize) -> sgx_status_t {
    let person_slice = unsafe { slice::from_raw_parts(msg, msg_len) };

    let the_one: person::Person = person::Person::decode(person_slice).unwrap();
    println!(
        "name: {}, id: 0x{:08X}, email at: {}",
        the_one.name, the_one.id, the_one.email
    );
    println!("{:?}", the_one);

    let ts = Timestamp {
        seconds: 0x1234,
        nanos: 0x5678,
    };
    println!("well known types ts = {:?}", ts);

    sgx_status_t::SGX_SUCCESS
}
