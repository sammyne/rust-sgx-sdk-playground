// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate dirs;
extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::io::{Read, Write};
use std::{fs, path};

static ENCLAVE_TOKEN: &'static str = "enclave.token";

extern "C" {
    fn ecall_aes_cmac(
        eid: sgx_enclave_id_t,
        status: *mut sgx_status_t,
        msg: *const u8,
        msg_len: usize,
        key: &[u8; 16],
        cmac: &mut [u8; 16],
    ) -> sgx_status_t;

    fn ecall_aes_gcm_128_decrypt(
        eid: sgx_enclave_id_t,
        status: *mut sgx_status_t,
        key: &[u8; 16],
        ciphertext: *const u8,
        ciphertext_len: usize,
        iv: &[u8; 12],
        mac: &[u8; 16],
        plaintext: *mut u8,
    ) -> sgx_status_t;

    fn ecall_aes_gcm_128_encrypt(
        eid: sgx_enclave_id_t,
        status: *mut sgx_status_t,
        key: &[u8; 16],
        msg: *const u8,
        msg_len: usize,
        iv: &[u8; 12],
        ciphertext: *mut u8,
        mac: &mut [u8; 16],
    ) -> sgx_status_t;

    fn ecall_sha256(
        eid: sgx_enclave_id_t,
        status: *mut sgx_status_t,
        msg: *const u8,
        msg_len: usize,
        hash: *mut u8,
    ) -> sgx_status_t;
}

fn hexlify(arr: &[u8]) -> String {
    let vec: Vec<String> = arr.iter().map(|b| format!("{:02x}", b)).collect();
    vec.join("")
}

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

fn test_ecall_aes_cmac(enclave: &SgxEnclave) -> Result<(), String> {
    // AES-CMAC test case comes from
    // https://tools.ietf.org/html/rfc4493
    // Example 3

    let key: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];
    let msg: &[u8] = &[
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf,
        0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11,
    ];

    let mut mac: [u8; 16] = [0; 16];
    let mut status = sgx_status_t::SGX_SUCCESS;
    let result = unsafe {
        ecall_aes_cmac(
            enclave.geteid(),
            &mut status,
            msg.as_ptr(),
            msg.len(),
            &key,
            &mut mac,
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return Err(result.as_str().to_string());
        }
    };

    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] sha256 ailed {}!", status.as_str());
            return Err(status.as_str().to_string());
        }
    };

    let expect = "dfa66747de9ae63030ca32611497c827";
    let got = hexlify(&mac);
    assert_eq!(got, expect);

    Ok(())
}

fn test_aes_gcm_decrypt(enclave: &SgxEnclave) -> Result<(), String> {
    // AES-GCM-128 test case comes from
    // http://csrc.nist.gov/groups/ST/toolkit/BCM/documents/proposedmodes/gcm/gcm-revised-spec.pdf
    // Test case 2
    let ciphertext = unhexlify("0388dace60b6a392f328c2b971b2fe78");

    let mut mac: [u8; 16] = [0; 16];
    mac.copy_from_slice(unhexlify("ab6e47d42cec13bdf53a67b21257bddf").as_slice());

    let sk: [u8; 16] = [0; 16];
    let iv: [u8; 12] = [0; 12];

    let expect = "00000000000000000000000000000000";

    let mut status = sgx_status_t::SGX_SUCCESS;
    let mut msg: [u8; 16] = [0; 16];
    let result = unsafe {
        ecall_aes_gcm_128_decrypt(
            enclave.geteid(),
            &mut status,
            &sk,
            ciphertext.as_ptr(),
            ciphertext.len(),
            &iv,
            &mac,
            msg.as_mut_ptr(),
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return Err(result.as_str().to_string());
        }
    };

    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] test_aes_gcm_decrypt failed {}!", status.as_str());
            return Err(status.as_str().to_string());
        }
    };

    let got = hexlify(&msg);
    assert_eq!(got, expect);

    Ok(())
}

fn test_aes_gcm_encrypt(enclave: &SgxEnclave) -> Result<(), String> {
    // AES-GCM-128 test case comes from
    // http://csrc.nist.gov/groups/ST/toolkit/BCM/documents/proposedmodes/gcm/gcm-revised-spec.pdf
    // Test case 2
    let msg: [u8; 16] = [0; 16];
    let sk: [u8; 16] = [0; 16];
    let iv: [u8; 12] = [0; 12];

    let expect = "0388dace60b6a392f328c2b971b2fe78";

    let mut status = sgx_status_t::SGX_SUCCESS;
    let mut ciphertext: [u8; 16] = [0; 16];
    let mut mac: [u8; 16] = [0; 16];
    let result = unsafe {
        ecall_aes_gcm_128_encrypt(
            enclave.geteid(),
            &mut status,
            &sk,
            msg.as_ptr(),
            msg.len(),
            &iv,
            ciphertext.as_mut_ptr(),
            &mut mac,
        )
    };

    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return Err(result.as_str().to_string());
        }
    };

    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] test_aes_gcm_encrypt failed {}!", status.as_str());
            return Err(status.as_str().to_string());
        }
    };

    let got = hexlify(&ciphertext);
    assert_eq!(got, expect);

    Ok(())
}

fn test_sha256(enclave: &SgxEnclave) -> Result<(), String> {
    struct TestCase {
        msg: String,
        expect: String,
    }

    let cases = vec![TestCase {
        msg: String::from("abc"),
        expect: String::from("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
    }];

    for c in cases {
        let mut hash: [u8; 32] = [0; 32];
        let mut status = sgx_status_t::SGX_SUCCESS;

        let result = unsafe {
            ecall_sha256(
                enclave.geteid(),
                &mut status,
                c.msg.as_ptr(),
                c.msg.len(),
                hash.as_mut_ptr(),
            )
        };

        match result {
            sgx_status_t::SGX_SUCCESS => {}
            _ => {
                println!("[-] ECALL Enclave Failed {}!", result.as_str());
                return Err(result.as_str().to_string());
            }
        };

        match status {
            sgx_status_t::SGX_SUCCESS => {}
            _ => {
                println!("[-] sha256 ailed {}!", status.as_str());
                return Err(status.as_str().to_string());
            }
        };

        let got = hexlify(&hash);
        assert_eq!(got, c.expect);
    }

    Ok(())
}

fn unhexlify(s: &str) -> Vec<u8> {
    if s.len() % 2 != 0 {
        panic!("odd-length hex string is invalid");
    }

    let ss = s.as_bytes();

    let from_hex = |x: u8| -> u8 {
        match x {
            y @ b'0'..=b'9' => y - b'0',
            z @ b'a'..=b'f' => z - b'a' + 10,
            _ => panic!("invalid hex digit"),
        }
    };

    let mut v: Vec<u8> = Vec::with_capacity(ss.len() / 2);
    for x in ss.chunks_exact(2) {
        v.push((from_hex(x[0]) << 4) | from_hex(x[1]));
    }

    v
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

    test_ecall_aes_cmac(&enclave).unwrap();

    test_aes_gcm_encrypt(&enclave).unwrap();

    test_aes_gcm_decrypt(&enclave).unwrap();

    test_sha256(&enclave).unwrap();

    //let out = unhexlify("0123456789").unwrap();
    //println!("{:?}", out);

    enclave.destroy();
}
