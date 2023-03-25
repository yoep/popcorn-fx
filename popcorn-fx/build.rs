use std::env;
use std::path::PathBuf;

use cbindgen::Config;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    println!("cargo:rerun-if-changed=src/ffi/mappings/mod.rs");
    println!("cargo:rerun-if-changed=../Cargo.lock");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("../include/")
        .join(format!("{}.hpp", package_name))
        .display()
        .to_string();
    let config = Config::from_file("../cbindgen.toml").unwrap();

    println!("Writing headers to {}", &output_file);
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
