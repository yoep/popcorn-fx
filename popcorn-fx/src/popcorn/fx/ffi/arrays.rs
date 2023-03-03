use std::ffi::c_char;

use popcorn_fx_core::{into_c_string, to_c_vec};

/// Structure holding the values of a string array.
#[repr(C)]
pub struct StringArray {
    values: *mut *const c_char,
    len: i32,
}

impl StringArray {
    pub fn from(values: Vec<String>) -> Self {
        let (values, len) = to_c_vec(values.into_iter()
            .map(|e| into_c_string(e))
            .collect());

        Self {
            values,
            len,
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::{from_c_string, from_c_vec};

    use super::*;

    #[test]
    fn test_string_array_from() {
        let vec = vec![
            "lorem".to_string(),
            "ipsum".to_string(),
        ];

        let array = StringArray::from(vec.clone());
        let result: Vec<String> = from_c_vec(array.values, array.len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert_eq!(vec, result)
    }
}