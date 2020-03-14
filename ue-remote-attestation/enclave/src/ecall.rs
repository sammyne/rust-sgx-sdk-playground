#![deny(unused)]

use std::prelude::v1::*;

use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::Arc;

use sgx_types::*;

use crate::attestation;
use crate::crypto::ecdsa;
use crate::x509;

#[no_mangle]
pub extern "C" fn listen_and_serve(socket_fd: c_int) -> sgx_status_t {
    // Generate Keypair
    let priv_key = match ecdsa::generate_key() {
        Ok(v) => v,
        Err(err) => {
            println!("fail to generate ECDSA key");
            return err;
        }
    };

    let (attn_report, sig, cert) =
        match attestation::create_attestation_report(&priv_key.public().xy) {
            Ok(r) => r,
            Err(err) => {
                println!("Error in create_attestation_report: {:?}", err);
                return err;
            }
        };

    let payload = attn_report + "|" + &sig + "|" + &cert;
    let (key_der, cert_der) = match x509::generate_self_signed_cert(payload, &priv_key) {
        Ok(r) => r,
        Err(err) => {
            println!("Error in gen_ecc_cert: {:?}", err);
            return err;
        }
    };

    let ca = {
        let root_ca_bin = include_bytes!("../../pki/ca.cert");
        let mut ca_reader = BufReader::new(&root_ca_bin[..]);

        let mut out = rustls::RootCertStore::empty();
        // Build a root ca storage
        out.add_pem_file(&mut ca_reader).expect("invalid certs");

        out
    };

    // Build a default authenticator which allow every authenticated client
    let config = {
        let authenticator = rustls::AllowAnyAuthenticatedClient::new(ca);
        let certs = vec![rustls::Certificate(cert_der)];
        let privkey = rustls::PrivateKey(key_der);

        let mut c = rustls::ServerConfig::new(authenticator);

        c.set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![])
            .expect("invalid config");

        c
    };

    let mut sess = rustls::ServerSession::new(&Arc::new(config));
    let mut conn = TcpStream::new(socket_fd).expect("invalid socket to build stream");
    let mut stream = rustls::Stream::new(&mut sess, &mut conn);
    let mut plaintext = [0u8; 1024];
    match stream.read(&mut plaintext) {
        Ok(_) => println!("Client said: {}", str::from_utf8(&plaintext).unwrap()),
        Err(err) => {
            panic!("Error in read_to_end: {:?}", err);
        }
    };

    stream.write("hello back".as_bytes()).unwrap();

    sgx_status_t::SGX_SUCCESS
}
