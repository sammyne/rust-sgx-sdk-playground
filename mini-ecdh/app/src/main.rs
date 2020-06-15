extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn ecall_must_ecdh(eid: sgx_enclave_id_t, alice_pub_key: *const u8) -> sgx_status_t;
}

fn panic_if_not_success(status: sgx_status_t, tip: &str) {
    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => panic!(format!("[-] {} {}!", tip, status.as_str())),
    }
}

fn new_enclave(enclave_path: &str) -> SgxResult<SgxEnclave> {
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

    let enclave = match new_enclave(&args[1]) {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    let alice_pub_key = [
        186, 205, 98, 72, 175, 135, 139, 244, 103, 132, 50, 192, 78, 66, 11, 254, 148, 65, 182,
        210, 107, 67, 45, 45, 185, 74, 141, 243, 142, 39, 170, 4, 80, 184, 135, 102, 120, 221, 105,
        148, 132, 235, 199, 46, 235, 208, 136, 28, 255, 208, 136, 17, 67, 82, 172, 40, 169, 132,
        102, 172, 36, 102, 249, 218,
    ];
    let status = unsafe { ecall_must_ecdh(enclave.geteid(), alice_pub_key.as_ptr()) };

    panic_if_not_success(status, "ecall_ecdh failed status");

    println!("[+] ecall_ecdh success...");

    enclave.destroy();
}
