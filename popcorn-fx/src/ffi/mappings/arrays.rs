use std::ffi::c_char;

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
}