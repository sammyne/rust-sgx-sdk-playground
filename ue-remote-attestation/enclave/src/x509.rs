#![deny(unused)]

use std::prelude::v1::*;
use std::str;
use std::time::*;
use std::untrusted::time::SystemTimeEx;

use sgx_types::*;

use bit_vec::BitVec;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc as TzUtc;
use num_bigint::BigUint;
use yasna::models::ObjectIdentifier;

use crate::crypto::ecdsa::{self, PrivateKey};

const ISSUER: &str = "MesaTEE";
const SUBJECT: &str = "MesaTEE";

pub const CERTEXPIRYDAYS: i64 = 90i64;

lazy_static! {
    static ref OID_ECDSA_WITH_SHA256: ObjectIdentifier =
        ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 4, 3, 2]);
    static ref OID_COMMON_NAME: ObjectIdentifier = ObjectIdentifier::from_slice(&[2, 5, 4, 3]);

    /// defined in https://tools.ietf.org/pdf/rfc3279.pdf#2.3.5
    static ref OID_EC_PUBLIC_KEY: ObjectIdentifier =
        ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 2, 1]);
    static ref OID_ECPK_PARAMETERS_NAMED_CURVE_PRIME256V1:  ObjectIdentifier= ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 3, 1, 7]);

    /// http://oid-info.com/cgi-bin/display?oid=2.16.840.1.113730.1.13&action=display
    static ref OID_FREE_FORM_TEXT_COMMENT:  ObjectIdentifier= ObjectIdentifier::from_slice(&[2, 16, 840, 1, 113730, 1, 13]);
}

pub fn generate_self_signed_cert(
    payload: String,
    priv_key: &PrivateKey,
) -> Result<(Vec<u8>, Vec<u8>), sgx_status_t> {
    let pub_key_bytes = priv_key.public().marshal_as_uncompressed();

    // Generate Certificate DER
    let cert_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_sequence(|writer| {
                // Certificate Version
                writer
                    .next()
                    .write_tagged(yasna::Tag::context(0), |writer| {
                        writer.write_i8(2);
                    });
                // Certificate Serial Number (unused but required)
                writer.next().write_u8(1);
                // Signature Algorithm: ecdsa-with-SHA256
                writer.next().write_sequence(|writer| {
                    writer.next().write_oid(&OID_ECDSA_WITH_SHA256);
                });
                // Issuer: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&OID_COMMON_NAME);
                            writer.next().write_utf8_string(&ISSUER);
                        });
                    });
                });
                // Validity: Issuing/Expiring Time (unused but required)
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let issue_ts = TzUtc.timestamp(now.as_secs() as i64, 0);
                let expire = now + Duration::days(CERTEXPIRYDAYS).to_std().unwrap();
                let expire_ts = TzUtc.timestamp(expire.as_secs() as i64, 0);
                writer.next().write_sequence(|writer| {
                    writer
                        .next()
                        .write_utctime(&yasna::models::UTCTime::from_datetime(&issue_ts));
                    writer
                        .next()
                        .write_utctime(&yasna::models::UTCTime::from_datetime(&expire_ts));
                });
                // Subject: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&OID_COMMON_NAME);
                            writer.next().write_utf8_string(&SUBJECT);
                        });
                    });
                });
                writer.next().write_sequence(|writer| {
                    // Public Key Algorithm
                    writer.next().write_sequence(|writer| {
                        // id-ecPublicKey
                        writer.next().write_oid(&OID_EC_PUBLIC_KEY);
                        // prime256v1
                        writer
                            .next()
                            .write_oid(&OID_ECPK_PARAMETERS_NAMED_CURVE_PRIME256V1);
                    });
                    // Public Key
                    writer
                        .next()
                        .write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                });
                // Certificate V3 Extension
                writer
                    .next()
                    .write_tagged(yasna::Tag::context(3), |writer| {
                        writer.write_sequence(|writer| {
                            writer.next().write_sequence(|writer| {
                                writer.next().write_oid(&OID_FREE_FORM_TEXT_COMMENT);
                                writer.next().write_bytes(&payload.into_bytes());
                            });
                        });
                    });
            });
            // Signature Algorithm: ecdsa-with-SHA256
            writer.next().write_sequence(|writer| {
                writer.next().write_oid(&OID_ECDSA_WITH_SHA256);
            });
            // Signature
            let sig = {
                let tbs = &writer.buf[4..];
                ecdsa::sign(priv_key, tbs).expect("fail to sign TBS")
            };
            let sig_der = yasna::construct_der(|writer| {
                writer.write_sequence(|writer| {
                    let mut sig_x = sig.x.clone();
                    sig_x.reverse();
                    let mut sig_y = sig.y.clone();
                    sig_y.reverse();
                    writer.next().write_biguint(&BigUint::from_slice(&sig_x));
                    writer.next().write_biguint(&BigUint::from_slice(&sig_y));
                });
            });
            writer.next().write_bitvec(&BitVec::from_bytes(&sig_der));
        });
    });

    // Generate Private Key DER
    let key_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_u8(0);
            writer.next().write_sequence(|writer| {
                writer
                    .next()
                    .write_oid(&ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 2, 1]));
                writer
                    .next()
                    .write_oid(&ObjectIdentifier::from_slice(&[1, 2, 840, 10045, 3, 1, 7]));
            });
            let inner_key_der = yasna::construct_der(|writer| {
                writer.write_sequence(|writer| {
                    writer.next().write_u8(1);
                    let prv_k_r = priv_key.as_bytes();
                    writer.next().write_bytes(&prv_k_r);
                    writer
                        .next()
                        .write_tagged(yasna::Tag::context(1), |writer| {
                            writer.write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                        });
                });
            });
            writer.next().write_bytes(&inner_key_der);
        });
    });

    Ok((key_der, cert_der))
}

pub fn percent_decode(orig: String) -> String {
    let v: Vec<&str> = orig.split("%").collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            ret.push(u8::from_str_radix(&s[0..2], 16).unwrap() as char);
            ret.push_str(&s[2..]);
        }
    }
    ret
}
