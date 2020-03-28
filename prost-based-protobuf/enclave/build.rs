use std::path::PathBuf;

fn main() {
    let src = PathBuf::from("../pb");
    let includes = &[src.clone()];
    let mut config = prost_build::Config::new();

    config
        .compile_protos(&[src.join("person.proto")], includes)
        .expect("fail to generate protobuf");
}
