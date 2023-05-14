
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/windows/popcorn-time.ico");
        res.compile().unwrap();
    }
}