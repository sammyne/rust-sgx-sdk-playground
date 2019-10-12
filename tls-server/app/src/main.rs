use std::{fs, net, path, ptr};
//use std::net::{SocketAddr,TcpListener,TcpStream};
//use std::os::raw::{}
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

extern crate sgx_types;
extern crate sgx_urts;

#[macro_use(defer)]
extern crate scopeguard;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn tls_server_new(
        eid: sgx_enclave_id_t,
        retval: *mut *const c_void,
        fd: c_int,
        cert: *const c_char,
        key: *const c_char,
    ) -> sgx_status_t;
    fn tls_server_read(
        eid: sgx_enclave_id_t,
        retval: *mut c_int,
        session: *const c_void,
        buf: *mut c_void,
        cnt: c_int,
    ) -> sgx_status_t;
    fn tls_server_write(
        eid: sgx_enclave_id_t,
        retval: *mut c_int,
        session: *const c_void,
        buf: *const c_void,
        cnt: c_int,
    ) -> sgx_status_t;
    // fn tls_server_wants_read(eid: sgx_enclave_id_t, retval: *mut c_int,
    //                  session: *const c_void) -> sgx_status_t;
    // fn tls_server_wants_write(eid: sgx_enclave_id_t, retval: *mut c_int,
    //                  session: *const c_void) -> sgx_status_t;
    fn tls_server_close(eid: sgx_enclave_id_t, session: *const c_void) -> sgx_status_t;
// fn tls_server_send_close(edi: sgx_enclave_id_t,
//                  session: *const c_void) -> sgx_status_t;
}

const BUFFER_SIZE: usize = 1024;

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

    println!("[+] Test tls server in enclave, start!");

    let addr: net::SocketAddr = "0.0.0.0:4433".parse().unwrap();
    let listener = net::TcpListener::bind(&addr).expect("cannot listen on port");

    let cert_path = CString::new(args[2].as_bytes()).unwrap();
    let key_path = CString::new(args[3].as_bytes()).unwrap();

    for stream in listener.incoming().take(1) {
        let stream = stream.unwrap();

        let mut session: *const c_void = ptr::null();

        let status = unsafe {
            tls_server_new(
                enclave.geteid(),
                &mut session as *mut *const c_void,
                stream.as_raw_fd(),
                cert_path.as_bytes_with_nul().as_ptr() as *const c_char,
                key_path.as_bytes_with_nul().as_ptr() as *const c_char,
            )
        };

        if status != sgx_status_t::SGX_SUCCESS {
            println!("[-] failed tls_server_new");
            return;
        }

        if session.is_null() {
            println!("[-] tls_server_new return nil server");
            return;
        }
        defer!{{
            let status = unsafe {
                tls_server_close(enclave.geteid(), session)
            };
            match status {
                sgx_status_t::SGX_SUCCESS => println!("session closed"),
                _ => println!("[-] tls_server_close failed: {}", status),
            };
        }};

        let mut buf: Vec<u8> = vec![0; BUFFER_SIZE];
        let mut buf_len = -1;
        let status = unsafe {
            tls_server_read(
                enclave.geteid(),
                &mut buf_len,
                session,
                buf.as_mut_slice().as_ptr() as *mut c_void,
                buf.len() as c_int,
            )
        };

        match status {
            sgx_status_t::SGX_SUCCESS => {
                println!("buf_len = {}", buf_len);
                buf.resize(buf_len as usize, 0);
                if buf.is_empty() {
                    println!("empty incoming message");
                    return;
                }
            }
            _ => {
                println!("[-] tls_server_read failed: {:?}", status);
                return;
            }
        };

        let req = String::from_utf8(buf).unwrap();
        println!("incoming: {:?}", req);

        let status = unsafe {
            let mut retval = -1;
            let s = tls_server_write(
                enclave.geteid(),
                &mut retval,
                session,
                req.as_ptr() as *const c_void,
                req.len() as c_int,
            );

            if retval == -1 {
                println!("[-] tls_server_write failed: {}", retval);
                return;
            }

            s
        };

        match status {
            sgx_status_t::SGX_SUCCESS => {}
            _ => {
                println!("[-] tls_server_write failed: {:?}", status);
                return;
            }
        };

        //unsafe {
        //    tls_server_close(enclave.geteid(), session);
        //}
    }

    println!("done");
}
