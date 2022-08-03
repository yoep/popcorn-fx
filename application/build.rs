use std::env;
use std::path::PathBuf;

use cbindgen::Config;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("../include/")
        .join(format!("{}.hpp", package_name))
        .display()
        .to_string();
    let config = Config::from_file("../cbindgen.toml").unwrap();

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
