#![no_std]

#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

extern crate http_req;

use std::ffi::CStr;
use std::net::TcpStream;
use std::os::raw::c_char;
use std::prelude::v1::*;

use sgx_types::*;

use http_req::{request::RequestBuilder, tls, uri::Uri};

#[no_mangle]
pub extern "C" fn send_http_request(uri_str: *const c_char) -> sgx_status_t {
    let uri_str = unsafe {
        CStr::from_ptr(uri_str)
            .to_str()
            .expect("Failed to recover hostname")
    };

    //Parse uri and assign it to variable `addr`
    let addr: Uri = uri_str.parse().unwrap();

    //Connect to remote host
    let stream = {
        // port is required
        let host_port = format!(
            "{}:{}",
            addr.host().expect("missing host"),
            addr.port().unwrap_or(addr.corr_port()),
        );

        println!("connecting to {}", host_port);

        TcpStream::connect(&host_port).expect("failed to connect")
    };

    //Open secure connection over TlsStream, because of `addr` (https)
    let mut stream = tls::Config::default()
        .connect(addr.host().unwrap_or(""), stream)
        .unwrap();

    //Container for response's body
    let mut writer = Vec::new();

    //Add header `Connection: Close`
    let response = RequestBuilder::new(&addr)
        .header("Connection", "Close")
        .send(&mut stream, &mut writer)
        .unwrap();

    println!("{}", String::from_utf8_lossy(&writer));
    println!("Status: {} {}", response.status_code(), response.reason());

    sgx_status_t::SGX_SUCCESS
}
