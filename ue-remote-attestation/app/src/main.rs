use std::io::{Read, Write};
use std::os::unix::io::IntoRawFd;
use std::{fs, net, path};

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn listen_and_serve(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        socket_fd: c_int,
        sign_type: sgx_quote_sign_type_t,
    ) -> sgx_status_t;
}

mod ocall;

pub use ocall::{
    ocall_get_ias_socket,
    ocall_get_quote,
    ocall_get_update_info,
    ocall_sgx_init_quote,
};

static ENCLAVE_TOKEN: &'static str = "enclave.token";

fn init_enclave(enclave_path: &str) -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = path::PathBuf::new();
    let use_token = match dirs::home_dir() {
        Some(path) => {
            println!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        }
        None => {
            println!("[-] Cannot get home dir");
            false
        }
    };

    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
    if use_token == true {
        match fs::File::open(&token_file) {
            Err(_) => {
                println!(
                    "[-] Open token file {} error! Will create one.",
                    token_file.as_path().to_str().unwrap()
                );
            }
            Ok(mut f) => {
                println!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(1024) => {
                        println!("[+] Token file valid!");
                    }
                    _ => println!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = try!(SgxEnclave::create(
        enclave_path,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr
    ));

    // Step 3: save the launch token if it is updated
    if use_token == true && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => match f.write_all(&launch_token) {
                Ok(()) => println!("[+] Saved updated launch token!"),
                Err(_) => println!("[-] Failed to save updated launch token!"),
            },
            Err(_) => {
                println!("[-] Failed to save updated enclave token, but doesn't matter");
            }
        }
    }

    Ok(enclave)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // args[0]: program name
    // args[1]: enclave path
    // args[2]: CA cert chain
    // args[3]: server key
    if args.len() < 4 {
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

    let addr: net::SocketAddr = "0.0.0.0:4433".parse().unwrap();
    let listener = net::TcpListener::bind(&addr).expect("cannot listen on port");

    let sign_type = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE;
    for stream in listener.incoming().take(1) {
        println!("new incoming session ...");
        let socket = stream.unwrap();

        let mut retval = sgx_status_t::SGX_SUCCESS;
        let status = unsafe {
            listen_and_serve(
                enclave.geteid(),
                &mut retval,
                socket.into_raw_fd(),
                sign_type,
            )
        };

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
