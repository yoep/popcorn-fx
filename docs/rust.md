# Rust

The Popcorn FX application uses a Rust backend for some modules which
handle certain system specific functionalities. 
The reason behind this is the performance and reliability of the system calls
which couldn't be relied upon with JNA.

## Tooling

The following tools are recommended to be used within Cargo:

- [cargo-edit](https://github.com/killercup/cargo-edit)

_These plugins can also be installed through Make with `make tooling`_