#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_trts;
extern crate sgx_types;

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_trts::trts::{rsgx_lfence, rsgx_raw_is_outside_enclave};
use sgx_types::*;
use std::mem;

use std::io::BufReader;
use std::untrusted::fs;

use std::ffi::CStr;
use std::os::raw::c_char;

use std::boxed::Box;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::vec::Vec;

extern crate rustls;
extern crate webpki;
use rustls::{NoClientAuth, Session, Stream};

pub struct TlsServer {
    socket: TcpStream,
    tls_session: rustls::ServerSession,
}
impl TlsServer {
    fn new(fd: c_int, cfg: Arc<rustls::ServerConfig>) -> TlsServer {
        TlsServer {
            socket: TcpStream::new(fd).unwrap(),
            tls_session: rustls::ServerSession::new(&cfg),
        }
    }
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let rsa_keys = {
        let keyfile = fs::File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::rsa_private_keys(&mut reader)
            .expect("file contains invalid rsa private key")
    };

    let pkcs8_keys = {
        let keyfile = fs::File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
            .expect("file contains invalid pkcs8 private key (encrypted keys not supported)")
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        pkcs8_keys[0].clone()
    } else {
        assert!(!rsa_keys.is_empty());
        rsa_keys[0].clone()
    }
}

fn make_config(cert: &str, key: &str) -> Arc<rustls::ServerConfig> {
    let mut config = rustls::ServerConfig::new(NoClientAuth::new());

    let certs = load_certs(cert);
    let privkey = load_private_key(key);
    config
        .set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![])
        .unwrap();

    Arc::new(config)
}

#[no_mangle]
pub extern "C" fn tls_server_close(session: *const c_void) {
    if session.is_null() {
        return;
    }

    if rsgx_raw_is_outside_enclave(session as *const u8, mem::size_of::<TlsServer>()) {
        return;
    }
    rsgx_lfence();

    let _ = unsafe { Box::<TlsServer>::from_raw(session as *mut _) };
}

#[no_mangle]
pub extern "C" fn tls_server_new(
    fd: c_int,
    cert: *const c_char,
    key: *const c_char,
) -> *const c_void {
    let certfile = unsafe { CStr::from_ptr(cert).to_str() };
    if certfile.is_err() {
        return ptr::null();
    }
    let keyfile = unsafe { CStr::from_ptr(key).to_str() };
    if keyfile.is_err() {
        return ptr::null();
    }
    let config = make_config(certfile.unwrap(), keyfile.unwrap());

    Box::into_raw(Box::new(TlsServer::new(fd, config))) as *const c_void
}

#[no_mangle]
pub extern "C" fn tls_server_read(session: *const c_void, buf: *mut c_char, cnt: c_int) -> c_int {
    if session.is_null() {
        return -1;
    }

    let session = unsafe { &mut *(session as *mut TlsServer) };

    let mut stream = rustls::Stream::new(&mut session.tls_session, &mut session.socket);

    let mut plaintext: Vec<u8> = vec![0; cnt as usize];
    let ell = stream.read(&mut plaintext).unwrap();

    let raw_buf = unsafe { slice::from_raw_parts_mut(buf as *mut u8, ell as usize) };
    raw_buf.copy_from_slice(&plaintext[..ell]);

    ell as i32
}

#[no_mangle]
pub extern "C" fn tls_server_write(
    session: *const c_void,
    buf: *const c_char,
    cnt: c_int,
) -> c_int {
    if session.is_null() {
        return -1;
    }

    let session = unsafe { &mut *(session as *mut TlsServer) };

    let mut stream = rustls::Stream::new(&mut session.tls_session, &mut session.socket);

    // cache buffer, waitting for next write_tls
    let cnt = cnt as usize;
    let plaintext = unsafe { slice::from_raw_parts(buf as *mut u8, cnt) };

    let result = stream.write(plaintext).unwrap();
    stream.flush().unwrap();

    result as i32
}
