use std::io::{self, Write};
use std::{env, fs, process};

struct SGX {
    sdk_dir: String,
    lib_dir: String,
    edger8r: String,
    is_sim: bool,

    rts_lib: String,

    rust_sdk_path: String,
}

impl SGX {
    fn new(c_sdk_path: &str, is_sim: bool, rust_sdk_path: &str) -> Result<Self, String> {
        // @TODO check existence of directory and files

        let sdk_dir = c_sdk_path.to_string();
        let lib_dir = format!("{}/lib64", &sdk_dir);
        let edger8r = format!("{}/bin/x64/sgx_edger8r", &sdk_dir);

        let rts_lib = {
            let suffix = match is_sim {
                true => "_sim",
                _ => "",
            };

            format!("sgx_urts{}", suffix)
        };

        Ok(SGX {
            sdk_dir,
            lib_dir,
            edger8r,
            is_sim,

            rts_lib,

            rust_sdk_path: rust_sdk_path.to_string(),
        })
    }

    /// @dev 3 env variables are read by this method
    ///     - SGX_SDK: path of intel SGX sdk
    ///     - SGX_MODE: mode of the built app
    ///     - RUST_SGX_SDK: path of rust-sgx-sdk
    /// @dev since SGX_MODE is used by some unknown process, the replacement by the 'PROFILE'
    ///     option of 'cargo build' still failed
    fn from_env() -> Result<Self, String> {
        let c_sdk_path = env::var("SGX_SDK").unwrap_or_else(|_| "/opt/sgxsdk".to_string());

        let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "SW".to_string()) != "HW";

        let rust_sdk_path = env::var("RUST_SGX_SDK").unwrap_or_else(|_| {
            let dir = fs::canonicalize("../../vendor/incubator-teaclave-sgx-sdk").unwrap();
            dir.to_str().unwrap().to_string()
        });

        Self::new(&c_sdk_path, is_sim, &rust_sdk_path)
    }
}

fn build_bridge_lib(sgx: &SGX, out_dir: &str) -> String {
    let lib_name = "enclave_u";
    let src = format!("{}/{}.c", out_dir, lib_name);

    let mut flags = match sgx.is_sim {
        true => "-g -O2".to_string(),
        _ => "-m64 -O0 -g".to_string(),
    };
    flags += " -fPIC -Wno-attributes";

    // default is a static library
    let mut build = cc::Build::new();

    build.file(src);

    for flag in flags.split_whitespace() {
        build.flag(flag);
    }

    // path to include, **order is important**
    build
        .include(format!("{}/edl", sgx.rust_sdk_path))
        .include(format!("{}/include", sgx.sdk_dir))
        .include(out_dir);

    build.compile(lib_name);

    lib_name.to_string()
}

fn generate_bridge(sgx: &SGX, untrusted_dir: &str) {
    let mut cmd = process::Command::new(&sgx.edger8r);

    cmd.args(&["--untrusted", "enclave.edl"]);

    let search_paths = format!(
        "../enclave {}/include {}/edl",
        &sgx.sdk_dir, &sgx.rust_sdk_path
    );
    for v in search_paths.split_whitespace() {
        cmd.args(&["--search-path", v]);
    }

    cmd.args(&["--untrusted-dir", untrusted_dir]);

    let output = cmd.output().expect("fail to generate bridge");

    if !output.status.success() {
        eprintln!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        panic!("failed to generate bridges for app");
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("missing OUT_DIR");

    let sgx = SGX::from_env().expect("failed to load config for SGX from env");

    generate_bridge(&sgx, &out_dir);

    let bridge_lib = build_bridge_lib(&sgx, &out_dir);

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", bridge_lib);

    println!("cargo:rustc-link-search=native={}", sgx.lib_dir);
    println!("cargo:rustc-link-lib=dylib={}", sgx.rts_lib);
}
