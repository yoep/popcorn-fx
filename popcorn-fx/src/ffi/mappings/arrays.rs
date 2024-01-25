use std::ffi::c_char;

use log::trace;

use popcorn_fx_core::{into_c_string, to_c_vec};

/// The C compatible string array.
/// It's mainly used for returning string arrays as result of C function calls.
#[repr(C)]
pub struct StringArray {
    /// The string array
    pub values: *mut *const c_char,
    /// The length of the string array
    pub len: i32,
}

impl From<Vec<String>> for StringArray {
    fn from(value: Vec<String>) -> Self {
        let (values, len) = to_c_vec(value.into_iter()
            .map(|e| into_c_string(e))
            .collect());

        Self {
            values,
            len,
        }
    }
}

impl From<&[String]> for StringArray {
    fn from(value: &[String]) -> Self {
        let (values, len) = to_c_vec(value.into_iter()
            .map(|e| into_c_string(e.clone()))
            .collect());

        Self {
            values,
            len,
        }
    }
}

/// A C-compatible byte array that can be used to return byte array data from Rust functions.
///
/// This struct contains a pointer to the byte array data and the length of the byte array.
/// It is intended for use in C code that needs to interact with Rust functions that return byte array data.
#[repr(C)]
#[derive(Debug)]
pub struct ByteArray {
    /// A pointer to the byte array data.
    pub values: *mut u8,
    /// The length of the byte array.
    pub len: i32,
}

impl From<Vec<u8>> for ByteArray {
    fn from(value: Vec<u8>) -> Self {
        trace!("Converting Vec<u8> to ByteArray");
        let (values, len) = to_c_vec(value);

        Self {
            values,
            len,
        }
    }
}

impl From<&ByteArray> for Vec<u8> {
    fn from(value: &ByteArray) -> Self {
        trace!("Converting ByteArray to Vec<u8>");
        if !value.values.is_null() && value.len > 0 {
            let slice = unsafe { std::slice::from_raw_parts(value.values, value.len as usize) };
            Vec::from(slice)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod test {
    use popcorn_fx_core::{from_c_string, from_c_vec};

    use super::*;

    #[test]
    fn test_from_string_array_vec() {
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

    #[test]
    fn test_from_string_array_slice() {
        let vec = vec![
            "ipsum".to_string(),
            "dol.or".to_string(),
        ];

        let array = StringArray::from(&vec[..]);
        let result: Vec<String> = from_c_vec(array.values, array.len).into_iter()
            .map(|e| from_c_string(e))
            .collect();

        assert_eq!(vec, result)
    }

    #[test]
    fn test_from_byte_array() {
        let vec: Vec<u8> = vec![13, 12];

        let array = ByteArray::from(vec.clone());
        let result1 = Vec::from(&array);
        let result2 = Vec::from(&array);

        assert_eq!(vec, result1);
        assert_eq!(vec, result2, "failed to read byte array multiple times");
    }
}