use std::ffi::c_char;

use popcorn_fx_core::{to_c_string, to_c_vec};

/// Structure holding the values of a string array.
#[repr(C)]
pub struct StringArray {
    values: *mut *const c_char,
    len: i32,
}

impl StringArray {
    pub fn from(values: Vec<String>) -> Self {
        let (values, len) = to_c_vec(values.into_iter()
            .map(|e| to_c_string(e))
            .collect());

        Self {
            values,
            len,
        }
    }
}