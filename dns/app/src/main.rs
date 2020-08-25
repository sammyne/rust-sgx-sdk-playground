extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn resolve(
        eid: sgx_enclave_id_t,
        status: *mut sgx_status_t,
        host_and_port: *const c_char,
    ) -> sgx_status_t;
}

fn panic_if_not_success(status: sgx_status_t, tip: &str) {
    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => panic!(format!("[-] {} {}!", tip, status.as_str())),
    }
}

fn init_enclave(enclave_path: &str) -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // [DEPRECATED since v2.6] Step 1: try to retrieve the launch token saved by last transaction
    // if there is no token, then create a new one.
    //

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    const DEBUG: i32 = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = SgxEnclave::create(
        enclave_path,
        DEBUG,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )?;

    // [DEPRECATED since v2.6] Step 3: save the launch token if it is updated

    Ok(enclave)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("missing enclave path");
        std::process::exit(-1);
    }

    let enclave = match init_enclave(&args[1]) {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    // port is a must
    //let host_and_port =
    //    CString::new("172.168.0.12:80").expect("failed to initialize host_and_port");
    let host_and_port =
        CString::new("www.baidu.com:80").expect("failed to initialize host_and_port");
    let mut status = sgx_status_t::SGX_SUCCESS;
    let result = unsafe { resolve(enclave.geteid(), &mut status, host_and_port.as_ptr()) };

    panic_if_not_success(result, "resolve failed result");
    panic_if_not_success(status, "resolve failed retval");

    println!("[+] resolve ok...");

    enclave.destroy();
}
