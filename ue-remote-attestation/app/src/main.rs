use std::net;
use std::os::unix::io::IntoRawFd;

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn listen_and_serve(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        socket: c_int,
    ) -> sgx_status_t;
}

mod ocall;

pub use ocall::{
    ocall_get_quote, ocall_get_update_info, ocall_sgx_init_quote,
};

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

    // args[0]: program name
    // args[1]: enclave path
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

    println!("[+] Test server in enclave, start!");

    let listener = net::TcpListener::bind("0.0.0.0:4433").expect("cannot listen on port");

    for stream in listener.incoming().take(1) {
        println!("new incoming session ...");
        let socket = stream.expect("invalid incoming stream");

        let mut retval = sgx_status_t::SGX_SUCCESS;
        let status =
            unsafe { listen_and_serve(enclave.geteid(), &mut retval, socket.into_raw_fd()) };

        if status != sgx_status_t::SGX_SUCCESS {
            println!("[-] failed run_server");
            return;
        }

        if retval != sgx_status_t::SGX_SUCCESS {
            println!("[-] failed run_server");
            return;
        }
    }

    println!("done");
}
