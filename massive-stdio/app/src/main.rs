extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern "C" {
    fn say_something(eid: sgx_enclave_id_t, some_string: *const u8, len: usize) -> sgx_status_t;
}

fn panic_if_not_success(status: sgx_status_t, tip: &str) {
    match status {
        sgx_status_t::SGX_SUCCESS => {}
        _ => panic!(format!("[-] {} {}!", tip, status.as_str())),
    }
}

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

    let x = String::from("x");

    let mut input_string_len = 1usize;
    // should panic as 1024 in case of input_string is specified as 'user_check'
    loop {
        let input_string = x.repeat(input_string_len);

        let result =
            unsafe { say_something(enclave.geteid(), input_string.as_ptr(), input_string.len()) };

        panic_if_not_success(result, "say_something failed result");

        println!("[+] say_something success... {}", input_string_len);
        input_string_len += 1;
    }

    //enclave.destroy();
}
