pub use checkbox::*;
pub use input::*;

mod checkbox;
mod input;

/// Print the given optional string value.
pub fn print_optional_string<S: ToString>(value: Option<S>) -> String {
    value
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or(String::default())
}
