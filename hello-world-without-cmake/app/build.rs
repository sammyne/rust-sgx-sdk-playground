use std::{env, fs, process};

struct SGX {
    sdk_dir: String,
    lib_dir: String,
    edger8r: String,
    //signer: String,
    mode: String,

    rts_lib_name: String,

    rust_sdk_path: String,
}

impl SGX {
    fn new(c_sdk_path: &str, mode: &str, rust_sdk_path: &str) -> Result<Self, String> {
        // @TODO check existence of directory and files

        let sdk_dir = c_sdk_path.to_string();
        let lib_dir = format!("{}/lib64", &sdk_dir);
        let edger8r = format!("{}/bin/x64/sgx_edger8r", &sdk_dir);
        //let signer = format!("{}/bin/x64/sgx_sign", &sdk_dir);

        let lib_suffix = match mode {
            "HW" => "",
            _ => "_sim",
        };

        let rts_lib_name = format!("sgx_urts{}", lib_suffix);

        let mode = mode.to_string();
        Ok(SGX {
            sdk_dir,
            lib_dir,
            edger8r,
            //signer,
            mode,

            rts_lib_name,

            rust_sdk_path: rust_sdk_path.to_string(),
        })
    }
}

fn build_bridge_lib(sgx: &SGX, untrusted_dir: &str) -> (String, String) {
    let lib_name = "enclave_u";
    let src = format!("{}/{}.c", untrusted_dir, lib_name);

    let mut flags = match sgx.mode.as_str() {
        "HW" => "-g -O2".to_string(),
        _ => "-m64 -O0 -g".to_string(),
    };
    flags += " -fPIC -Wno-attributes";

    // default is a static library
    let mut build = cc::Build::new();

    build.file(src);

    //let flags = flags.split_whitespace();
    for flag in flags.split_whitespace() {
        build.flag(flag);
    }

    // path to include, **order is important**
    build
        .include(format!("{}/edl", sgx.rust_sdk_path))
        .include(format!("{}/include", sgx.sdk_dir))
        .include(untrusted_dir);

    build.compile(lib_name);

    (untrusted_dir.to_string(), lib_name.to_string())
}

fn generate_bridge(sgx: &SGX, untrusted_dir: &str) {
    let mut cmd = process::Command::new(&sgx.edger8r);
    cmd.args(&["--untrusted", "enclave.edl"]);

    let sgx_edl_dir = format!("{}/include", sgx.sdk_dir);
    let search_paths = vec![
        "../enclave",
        sgx_edl_dir.as_str(),
        "../../vendor/rust-sgx-sdk/edl",
    ];
    for v in search_paths {
        cmd.args(&["--search-path", v]);
    }

    cmd.args(&["--untrusted-dir", untrusted_dir]);

    let _output = cmd.output().unwrap();
    //println!("status: {}", output.status);
    //println!("stdout: {}", String::from_utf8(output.stdout).unwrap());
    //println!("stderr: {}", String::from_utf8(output.stderr).unwrap());
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("missing OUT_DIR");

    println!("------ out_dir = {}", out_dir);
    let sdk_dir = env::var("SGX_SDK").unwrap_or_else(|_| "/opt/sgxsdk".to_string());

    let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "SW".to_string());

    // @TODO: load this from env
    let rust_sdk_dir = fs::canonicalize("../../vendor/rust-sgx-sdk").unwrap();
    let rust_sdk_dir = rust_sdk_dir.to_str().unwrap();
    println!(">> rust_sdk_dir: {}", rust_sdk_dir);

    let sgx = SGX::new(&sdk_dir, &is_sim, rust_sdk_dir).unwrap();

    generate_bridge(&sgx, &out_dir);

    let (bridge_lib_dir, bridge_lib_name) = build_bridge_lib(&sgx, &out_dir);

    println!("cargo:rustc-link-search=native={}", bridge_lib_dir);
    println!("cargo:rustc-link-lib=static={}", bridge_lib_name);

    println!("cargo:rustc-link-search=native={}", sgx.lib_dir);
    println!("cargo:rustc-link-lib=dylib={}", sgx.rts_lib_name);
}
