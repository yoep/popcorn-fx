use std::env;

use protobuf_codegen::{Codegen, Customize};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=../Cargo.lock");
    println!("cargo:rerun-if-env-changed=GENERATE_PROTO");
    println!("cargo:rerun-if-changed=protobuf/*.proto");

    let generate_proto = env::var("GENERATE_PROTO").unwrap_or_else(|_| "false".to_string());
    if generate_proto == "true" || true {
        Codegen::new()
            .out_dir("src/ipc/protobuf")
            .includes(["../protobuf"])
            .inputs([
                "../protobuf/application_args.proto",
                "../protobuf/log.proto",
                "../protobuf/message.proto",
                "../protobuf/settings.proto",
                "../protobuf/subtitle.proto",
            ])
            .customize(Customize::default().lite_runtime(true).gen_mod_rs(true))
            .run()
            .expect("protoc");
    }
}
