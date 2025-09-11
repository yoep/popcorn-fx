pub use checkbox::*;
pub use combobox::*;
pub use input::*;
use std::iter::repeat;

mod checkbox;
mod combobox;
mod input;

/// Print the given optional string value.
pub fn print_optional_string<S: ToString>(value: Option<S>) -> String {
    value
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or(String::default())
}

/// Print the given string value as a fixed string length with additional suffix spaces if needed, or truncating if exceeding.
pub fn print_string_len<S: AsRef<str>>(value: S, len: usize) -> String {
    let mut s: String = value.as_ref().chars().take(len).collect();
    let current_len = s.len();

    if current_len < len {
        s.extend(repeat(' ').take(len - current_len));
    }

    s
}
