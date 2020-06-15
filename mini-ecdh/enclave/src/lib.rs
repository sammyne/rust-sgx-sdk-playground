#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;

use sgx_tcrypto as crypto;
use sgx_types::*;

use crypto::SgxEccHandle;

#[no_mangle]
pub extern "C" fn ecall_must_ecdh(alice_pub_key: &[u8; 64]) {
    let curve = SgxEccHandle::new();
    curve.open().expect("fail to initialize curve");

    let (priv_key, pub_key) = curve.create_key_pair().expect("fail to generate key pair");

    let mut alice = sgx_ec256_public_t::default();

    //println!("alice = {:?}",&alice_pub_key[..]);
    //println!("alice.len() = {}",alice_pub_key.len());

    alice.gx.clone_from_slice(&alice_pub_key[..32]);
    alice.gx.reverse();

    alice.gy.clone_from_slice(&alice_pub_key[32..]);
    alice.gy.reverse();

    if !curve
        .check_point(&alice)
        .expect("fail to check point on curve")
    {
        panic!("point out of curve");
    }

    let dhkey = curve
        .compute_shared_dhkey(&priv_key, &alice)
        .expect("fail to generate shared key");

        let s = reverse(dhkey.s);
    println!("dhkey = {:?}", &s[..]);

    let x = reverse(pub_key.gx);
    println!("bob's gx = {:?}", &x[..]);

    let y = reverse(pub_key.gy);
    println!("bob's gy = {:?}", &y[..]);
}

fn reverse(arr: [u8;32]) -> [u8;32] {
    let mut x = arr;
    x.reverse();

    x
}