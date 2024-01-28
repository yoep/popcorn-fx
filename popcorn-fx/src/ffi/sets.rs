use std::fmt::Debug;

use log::trace;

use popcorn_fx_core::{from_c_vec, to_c_vec};

/// A C-compatible set/array of items.
///
/// This struct is used to represent a set of items that can be passed between Rust and C code.
/// It includes a pointer to the items and their length.
#[repr(C)]
#[derive(Debug)]
pub struct CSet<T: Debug + Clone> {
    /// A pointer to the array of items.
    pub items: *mut T,
    /// The length of the array.
    pub len: i32,
}

impl<T: Debug + Clone> From<Vec<T>> for CSet<T> {
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
    /// use popcorn_fx::ffi::CSet;
    ///
    /// let rust_vec = vec![1, 2, 3, 4, 5];
    /// let c_set: CSet<i32> = rust_vec.into();
    /// ```
    fn from(value: Vec<T>) -> Self {
        trace!("Converting vector into C set");
        let (items, len) = to_c_vec(value);

        Self {
            items,
            len,
        }
    }
}

impl<T: Debug + Clone> From<CSet<T>> for Vec<T> {
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
    /// use popcorn_fx::ffi::CSet;
    ///
    /// let c_set = CSet { items: [1, 2, 3].as_mut_ptr(), len: 3 };
    /// let rust_vec: Vec<i32> = c_set.into();
    /// ```
    fn from(value: CSet<T>) -> Self {
        trace!("Converting C set {:?} into vector", value);
        from_c_vec(value.items, value.len)
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use popcorn_fx_core::{from_c_string, into_c_string};

    use crate::ffi::PlaylistItemC;

    use super::*;

    #[test]
    fn test_from_c_set() {
        let url = "https://MyUrl";
        let item = PlaylistItemC {
            url: into_c_string(url.to_string()),
            title: ptr::null(),
            thumb: ptr::null(),
            quality: ptr::null(),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
        };
        let (items, len) = to_c_vec(vec![item]);
        let set = CSet::<PlaylistItemC> {
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
            thumb: ptr::null(),
            quality: ptr::null(),
            parent_media: ptr::null_mut(),
            media: ptr::null_mut(),
            auto_resume_timestamp: ptr::null_mut(),
        };

        let set = CSet::<PlaylistItemC>::from(vec![item]);
        let result = from_c_vec(set.items, set.len);

        assert_eq!(1, result.len());
        assert_eq!(url.to_string(), from_c_string(result.get(0).unwrap().url));
    }
}