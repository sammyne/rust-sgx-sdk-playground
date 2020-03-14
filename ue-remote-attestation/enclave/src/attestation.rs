#![deny(unused)]

use std::prelude::v1::*;

use sgx_rand::{os::SgxRng, Rng};
use sgx_tse::*;
use sgx_types::*;

use itertools::Itertools;
use std::net::TcpStream;
use std::{ptr, str};

use http_req::request::{Method, RequestBuilder};
use http_req::response::Response;
use http_req::{tls, uri::Uri};

pub const DEV_HOSTNAME: &'static str = "api.trustedservices.intel.com";
pub const SIGRL_SUFFIX: &'static str = "/sgx/dev/attestation/v3/sigrl/";
pub const REPORT_SUFFIX: &'static str = "/sgx/dev/attestation/v3/report";

use crate::hex;
use crate::x509;

extern "C" {
    pub fn ocall_sgx_init_quote(
        ret_val: *mut sgx_status_t,
        ret_ti: *mut sgx_target_info_t,
        ret_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;

    pub fn ocall_get_quote(
        ret_val: *mut sgx_status_t,
        p_sigrl: *const u8,
        sigrl_len: u32,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        maxlen: u32,
        p_quote_len: *mut u32,
    ) -> sgx_status_t;
}

fn parse_response_attn_report(response: Response, body: Vec<u8>) -> (String, String, String) {
    let status_msg = match Into::<u16>::into(response.status_code()) {
        200 => "OK Operation Successful",
        401 => "Unauthorized Failed to authenticate or authorize request.",
        404 => "Not Found GID does not refer to a valid EPID group ID.",
        500 => "Internal error occurred",
        503 => {
            "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. "
        }
        _ => {
            println!("DBG:{}", response.status_code());
            "Unknown error occured"
        }
    };

    println!("status: {}", status_msg);

    let headers = response.headers();
    let signing_cert = {
        let c = headers
            .get("X-IASReport-Signing-Certificate")
            .expect("missing 'X-IASReport-Signing-Certificate'")
            .replace("%0A", "");

        let c = x509::percent_decode(c);
        let v: Vec<&str> = c.split("-----").collect();

        v[2].to_string()
    };
    let sig = headers
        .get("X-IASReport-Signature")
        .expect("missing 'X-IASReport-Signature'")
        .to_string();

    let report = if response.content_len().is_ok() {
        str::from_utf8(&body).expect("non-utf8 report").to_string()
    } else {
        "".to_string()
    };

    (report, sig, signing_cert)
}

fn parse_response_sigrl(response: Response, body: Vec<u8>) -> Vec<u8> {
    println!("parsing response for sigRL...");

    let status_msg = match Into::<u16>::into(response.status_code()) {
        200 => "OK Operation Successful",
        401 => "Unauthorized Failed to authenticate or authorize request.",
        404 => "Not Found GID does not refer to a valid EPID group ID.",
        500 => "Internal error occurred",
        503 => {
            "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. "
        }
        _ => {
            println!("DBG:{}", response.status_code());
            "Unknown error occured"
        }
    };

    println!("status: {}", status_msg);

    if body.len() != 0 {
        println!("Base64-encoded SigRL: {:?}", String::from_utf8_lossy(&body));

        return base64::decode(str::from_utf8(&body).expect("non-utf8 body"))
            .expect("non-base64 body");
    }

    // len_num == 0
    Vec::new()
}

pub fn get_sigrl_from_intel(gid: u32) -> Vec<u8> {
    println!("uri: {}{}{:08x}", DEV_HOSTNAME, SIGRL_SUFFIX, gid);
    let addr: Uri = format!("https://{}{}{:08x}", DEV_HOSTNAME, SIGRL_SUFFIX, gid)
        .parse()
        .expect("invalid uri");
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

    let mut stream = tls::Config::default()
        .connect(addr.host().unwrap_or(""), stream)
        .expect("failed to connect");

    let mut response_body = vec![];
    let (_, _, ias_key) = ias_config();
    let response = RequestBuilder::new(&addr)
        .header("Connection", "Close")
        .header("Ocp-Apim-Subscription-Key", ias_key)
        .send(&mut stream, &mut response_body)
        .expect("failed to send request");

    println!("response: '{}'", String::from_utf8_lossy(&response_body));

    parse_response_sigrl(response, response_body)
}

// TODO: support pse
pub fn get_report_from_intel(quote: Vec<u8>) -> (String, String, String) {
    let addr: Uri = format!("https://{}{}", DEV_HOSTNAME, REPORT_SUFFIX)
        .parse()
        .expect("invalid uri");

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

    let mut stream = tls::Config::default()
        .connect(addr.host().unwrap_or(""), stream)
        .expect("failed to connect");

    let (_, _, ias_key) = ias_config();
    let encoded_quote = base64::encode(&quote[..]);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let mut response_body = vec![];

    let response = RequestBuilder::new(&addr)
        .body(encoded_json.as_bytes())
        .method(Method::POST)
        .header("Connection", "Close")
        .header("Content-Length", &encoded_json.len())
        .header("Content-Type", "application/json")
        .header("Ocp-Apim-Subscription-Key", ias_key)
        .send(&mut stream, &mut response_body)
        .expect("failed to send request");

    println!("response: '{}'", String::from_utf8_lossy(&response_body));
    println!("status: {} {}", response.status_code(), response.reason());

    //let (attn_report, sig, cert) = parse_response_attn_report(&plaintext);
    //(attn_report, sig, cert)
    parse_response_attn_report(response, response_body)
}

#[allow(const_err)]
pub fn create_attestation_report(
    pub_k: &sgx_ec256_public_t,
) -> Result<(String, String, String), sgx_status_t> {
    // Workflow:
    // (1) ocall to get the target_info structure (target_info) and epid group id (eg)
    // (1.5) get sigrl
    // (2) call sgx_create_report with target_info+data, produce an sgx_report_t
    // (3) ocall to sgx_get_quote to generate (*mut sgx-quote_t, uint32_t)

    // (1) get target_info + eg

    let (target_info, epid_group_id) = {
        let mut target_info: sgx_target_info_t = sgx_target_info_t::default();
        let mut eg: sgx_epid_group_id_t = sgx_epid_group_id_t::default();

        let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
        let res = unsafe {
            ocall_sgx_init_quote(
                &mut rt as *mut sgx_status_t,
                &mut target_info as *mut sgx_target_info_t,
                &mut eg as *mut sgx_epid_group_id_t,
            )
        };

        error_out_if_not_success(res, None)?;
        error_out_if_not_success(rt, None)?;

        (target_info, u32::from_le_bytes(eg))
    };
    println!("EPID group ID = {:?}", epid_group_id);

    // Now sigrl_vec is the revocation list, a vec<u8>
    let sigrl: Vec<u8> = get_sigrl_from_intel(epid_group_id);

    // (2) Generate the report
    // Fill ecc256 public key into report_data
    let report_data = {
        let mut d = [0u8; 64];

        d[..32].copy_from_slice(&pub_k.gx);
        d[32..].copy_from_slice(&pub_k.gy);

        d[..32].reverse();
        d[32..].reverse();

        sgx_report_data_t { d }
    };

    let report = rsgx_create_report(&target_info, &report_data).expect("fail to create report");
    println!("MRSIGNER: {:x?}", report.body.mr_signer.m);

    let quote_nonce = {
        let mut rand = [0u8; 16];
        let mut rng = SgxRng::new().expect("fail to new RNG");
        rng.fill_bytes(&mut rand);

        println!("quote_nonce: {:x?}", rand);

        sgx_quote_nonce_t { rand }
    };

    let mut qe_report = sgx_report_t::default();
    //const RET_QUOTE_BUF_LEN: u32 = 2048;
    //let mut return_quote_buf: [u8; RET_QUOTE_BUF_LEN as usize] = [0; RET_QUOTE_BUF_LEN as usize];
    let mut quote = vec![0u8; 2048];
    let mut quote_len: u32 = 0;

    // (3) Generate the quote
    // Args:
    //       1. sigrl: ptr + len
    //       2. report: ptr 432bytes
    //       3. linkable: u32, unlinkable=0, linkable=1
    //       4. spid: sgx_spid_t ptr 16bytes
    //       5. sgx_quote_nonce_t ptr 16bytes
    //       6. p_sig_rl + sigrl size ( same to sigrl)
    //       7. [out]p_qe_report need further check
    //       8. [out]p_quote
    //       9. quote_size

    let (quote_type, spid) = {
        let (quote_type, spid, _) = ias_config();
        (quote_type, hex::decode_spid(spid))
    };

    // the pointer passed to ocall_get_quote must be null if sigrl.len()==0
    let sigrl_ptr = if sigrl.len() == 0 {
        ptr::null()
    } else {
        sigrl.as_ptr()
    };

    {
        let mut rt = sgx_status_t::SGX_SUCCESS;
        let result = unsafe {
            ocall_get_quote(
                &mut rt as *mut sgx_status_t,
                sigrl_ptr,
                sigrl.len() as u32,
                &report,
                quote_type,
                &spid,
                &quote_nonce,
                &mut qe_report,
                quote.as_mut_ptr(),
                quote.len() as u32,
                &mut quote_len,
            )
        };

        error_out_if_not_success(result, None)?;
        error_out_if_not_success(rt, Some("ocall_get_quote returned"))?;
    }
    quote.resize(quote_len as usize, 0);

    // Added 09-28-2018
    // Perform a check on qe_report to verify if the qe_report is valid
    match rsgx_verify_report(&qe_report) {
        Ok(_) => println!("rsgx_verify_report passed!"),
        Err(err) => {
            println!("rsgx_verify_report failed with {:?}", err);
            return Err(err);
        }
    }

    // Check if the qe_report is produced on the same platform
    if target_info.mr_enclave.m != qe_report.body.mr_enclave.m
        || target_info.attributes.flags != qe_report.body.attributes.flags
        || target_info.attributes.xfrm != qe_report.body.attributes.xfrm
    {
        println!("qe_report does not match current target_info!");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    println!("qe_report check passed");

    // Check qe_report to defend against replay attack
    // The purpose of p_qe_report is for the ISV enclave to confirm the QUOTE
    // it received is not modified by the untrusted SW stack, and not a replay.
    // The implementation in QE is to generate a REPORT targeting the ISV
    // enclave (target info from p_report) , with the lower 32Bytes in
    // report.data = SHA256(p_nonce||p_quote). The ISV enclave can verify the
    // p_qe_report and report.data to confirm the QUOTE has not be modified and
    // is not a replay. It is optional.

    let rhs_hash = {
        let mut rhs_vec: Vec<u8> = quote_nonce.rand.to_vec();
        rhs_vec.extend(&quote);
        sgx_tcrypto::rsgx_sha256_slice(&rhs_vec[..]).expect("fail to do SHA256")
    };
    let lhs_hash = &qe_report.body.report_data.d[..32];

    if rhs_hash != lhs_hash {
        println!(
            "Quote is tampered! Invalid hash: expect {:02X}, got {:02X}",
            rhs_hash.iter().format(""),
            lhs_hash.iter().format("")
        );
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    Ok(get_report_from_intel(quote))
}

// subscription type, SPID, API key
fn ias_config() -> (sgx_quote_sign_type_t, &'static str, &'static str) {
    (
        sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE,
        "1825E5672DE30E2F0C29C0CCBD193B74",
        "3e8c48b2c12344b7807fd25761d06067",
    )
}

fn error_out_if_not_success(
    status: sgx_status_t,
    description: Option<&str>,
) -> Result<(), sgx_status_t> {
    if status == sgx_status_t::SGX_SUCCESS {
        return Ok(());
    }

    if let Some(v) = description {
        println!("{}: {:?}", v, status);
    }

    Err(status)
}
