/// The C-compatible logging level for log messages sent over FFI.
///
/// This enum represents the different logging levels that can be used to send log messages from Rust to C code.
/// It includes five different levels of logging: `Trace`, `Debug`, `Info`, `Warn`, and `Error`.
#[repr(i32)]
#[derive(Debug)]
pub enum LogLevel {
    Off = 0,
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}