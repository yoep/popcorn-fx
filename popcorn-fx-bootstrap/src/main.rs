use std::env;
use std::process::Command;

///The main entry of the bootstrap application.
fn main() {
    let mut child = Command::new("popcorn-time.exe")
        .args(env::args())
        .spawn()
        .expect("expected the application to be started");

    let ecode = child.wait().expect("expected exit status");


}