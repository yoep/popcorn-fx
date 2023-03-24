# Rust

The Popcorn FX application uses a Rust backend for some modules which
handle certain system specific functionalities. 
The reason behind this is the performance and reliability of the system calls
which couldn't be relied upon with JNA.

## Tooling

The following tools are recommended to be used within Cargo:

- [cargo-edit](https://github.com/killercup/cargo-edit)
- [cargo-nextest](https://github.com/nextest-rs/nextest)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

_These plugins can also be installed through Make with `make prerequisites`_