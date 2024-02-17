use std::ffi::c_char;
use std::fmt::Debug;

use log::trace;

use popcorn_fx_core::{from_c_string, from_c_vec, from_c_vec_owned, into_c_string, into_c_vec};

/// The C compatible string array.
/// It's mainly used for returning string arrays as result of C function calls.
#[repr(C)]
#[derive(Debug)]
pub struct StringArray {
    /// The string array
    pub values: *mut *const c_char,
    /// The length of the string array
    pub len: i32,
}

impl From<Vec<String>> for StringArray {
    fn from(value: Vec<String>) -> Self {
        let (values, len) = into_c_vec(value.into_iter()
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
        let (values, len) = into_c_vec(value.into_iter()
            .map(|e| into_c_string(e.clone()))
            .collect());

        Self {
            values,
            len,
        }
    }
}

impl Drop for StringArray {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        let _ = from_c_vec_owned(self.values, self.len).into_iter()
            .map(|e| from_c_string(e))
            .collect::<Vec<String>>();
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
        let (values, len) = into_c_vec(value);

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

impl Drop for ByteArray {
    fn drop(&mut self) {
        trace!("Dropping ByteArray {:?}", self);
        from_c_vec_owned(self.values, self.len);
    }
}

/// A C-compatible set/array of items.
///
/// This struct is used to represent a set of items that can be passed between Rust and C code.
/// It includes a pointer to the items and their length.
#[repr(C)]
#[derive(Debug)]
pub struct CArray<T: Debug + Clone> {
    /// A pointer to the array of items.
    pub items: *mut T,
    /// The length of the array.
    pub len: i32,
}

impl<T: Debug + Clone> From<Vec<T>> for CArray<T> {
    /// Converts a Rust `Vec<T>` into a CSet<T>.
    ///
    /// This function takes a Rust Vec<T>, converts it into a C-compatible array and returns a CSet<T> with a pointer
    /// to the array and its length.
    ///
    /// # Arguments
    ///
    /// * `value` - The Vec<T> to be converted into a CSet<T>.
    ///
    /// # Example
    ///
    /// ```rust
    /// use popcorn_fx::ffi::CArray;
    ///
    /// let rust_vec = vec![1, 2, 3, 4, 5];
    /// let c_set: CArray<i32> = rust_vec.into();
    /// ```
    fn from(value: Vec<T>) -> Self {
        trace!("Converting vector into C set");
        let (items, len) = into_c_vec(value);

        Self {
            items,
            len,
        }
    }
}

impl<T: Debug + Clone> From<CArray<T>> for Vec<T> {
    /// Converts a CSet<T> into a Rust Vec<T>.
    ///
    /// This function takes a CSet<T> and creates a Rust Vec<T> by copying the elements from the C-compatible array.
    ///
    /// # Arguments
    ///
    /// * `value` - The CSet<T> to be converted into a Vec<T>.
    ///
    /// # Example
    ///
    /// ```rust
    /// use popcorn_fx::ffi::CArray;
    ///
    /// let c_set = CArray { items: [1, 2, 3].as_mut_ptr(), len: 3 };
    /// let rust_vec: Vec<i32> = c_set.into();
    /// ```
    fn from(value: CArray<T>) -> Self {
        trace!("Converting C set {:?} into vector", value);
        from_c_vec(value.items, value.len)
    }
}

impl<T: Debug + Clone> Drop for CArray<T> {
    fn drop(&mut self) {
        trace!("Dropping {:?}", self);
        // TODO: Fix crash on cleanup
        // from_c_vec_owned(self.items, self.len);
    }
}

#[cfg(test)]
mod test {
    use std::ptr;

    use popcorn_fx_core::{from_c_string, from_c_vec};

    use crate::ffi::PlaylistItemC;

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

    #[test]
    fn test_from_c_set() {
        let url = "https://MyUrl";
        let item = PlaylistItemC {
            url: into_c_string(url.to_string()),
            title: ptr::null(),
            caption: ptr::null(),
            thumb: ptr::null(),
            quality: ptr::null(),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
            subtitles_enabled: false,
        };
        let (items, len) = into_c_vec(vec![item]);
        let set = CArray::<PlaylistItemC> {
            items,
            len,
        };

        let result = Vec::<PlaylistItemC>::from(set);

        assert_eq!(1, result.len());
        assert_eq!(url.to_string(), from_c_string(result.get(0).unwrap().url));
    }

    #[test]
    fn test_from_vec() {
        let url = "https://localhost:8080/MyUri.mp4";
        let item = PlaylistItemC {
            url: into_c_string(url.to_string()),
            title: into_c_string("MyTitle".to_string()),
            caption: ptr::null(),
            thumb: ptr::null(),
            quality: ptr::null(),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
            subtitles_enabled: false,
        };

        let set = CArray::<PlaylistItemC>::from(vec![item]);
        let result = from_c_vec(set.items, set.len);

        assert_eq!(1, result.len());
        assert_eq!(url.to_string(), from_c_string(result.get(0).unwrap().url));
    }
}