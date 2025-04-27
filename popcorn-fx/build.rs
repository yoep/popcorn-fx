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
            .out_dir("src/ipc/proto")
            .includes(["../protobuf"])
            .inputs([
                "../protobuf/application.proto",
                "../protobuf/events.proto",
                "../protobuf/favorites.proto",
                "../protobuf/images.proto",
                "../protobuf/loader.proto",
                "../protobuf/log.proto",
                "../protobuf/media.proto",
                "../protobuf/message.proto",
                "../protobuf/playback.proto",
                "../protobuf/player.proto",
                "../protobuf/playlist.proto",
                "../protobuf/settings.proto",
                "../protobuf/subtitle.proto",
                "../protobuf/torrent.proto",
                "../protobuf/tracking.proto",
                "../protobuf/update.proto",
                "../protobuf/watched.proto",
            ])
            .customize(Customize::default().lite_runtime(true).gen_mod_rs(true))
            .run()
            .expect("protoc");
    }
}
