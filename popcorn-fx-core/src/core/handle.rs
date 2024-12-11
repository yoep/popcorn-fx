use std::time::{SystemTime, UNIX_EPOCH};

use derive_more::Display;
use rand::Rng;

/// A unique opaque handle which can be used as resource identifier.
///
/// Handles are commonly used to uniquely identify objects or resources within a system.
/// This `Handle` struct provides a convenient way to generate and manage unique handles.
///
/// # Example
///
/// ```
/// use popcorn_fx_core::core::Handle;
///
/// let handle = Handle::new();
/// println!("Generated Handle: {:?}", handle);
/// ```
#[derive(Debug, Display, Copy, Clone, PartialEq, Eq, Hash)]
#[display(fmt = "handle {}", handle)]
pub struct Handle {
    handle: i64,
}

impl Handle {
    /// Creates a new `Handle` with a unique identifier.
    ///
    /// # Returns
    ///
    /// A new `Handle` instance with a unique identifier.
    ///
    /// # Panics
    ///
    /// This function may panic if the system time goes backward during its execution. However, such cases are extremely rare and unlikely to occur under normal circumstances.
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        let mut rng = rand::thread_rng();
        let random_number: i64 = rng.gen();

        Self {
            handle: (timestamp << 32) | (random_number & 0xFFFF_FFFF),
        }
    }

    /// Retrieve the underlying value of the handle.
    ///
    /// This method is primarily used when mapping a `Handle` to a C function, allowing access to the raw integer value of the handle.
    ///
    /// # Returns
    ///
    /// The raw integer value of the handle.
    pub fn value(&self) -> i64 {
        self.handle.clone()
    }
}

impl Default for Handle {
    /// Creates a default `Handle` using the `new` constructor.
    ///
    /// # Returns
    ///
    /// A new `Handle` instance with a unique identifier generated from the current timestamp and a random number.
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for Handle {
    fn from(value: i64) -> Self {
        Self { handle: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_new() {
        let result = Handle::new();

        assert_ne!(
            result.handle, 0,
            "expected a unique id to have been generated"
        );
    }

    #[test]
    fn test_handle_default() {
        let result = Handle::default();

        assert_ne!(
            result.handle, 0,
            "expected a unique id to have been generated"
        );
    }

    #[test]
    fn test_handle_from() {
        let id = 458775i64;

        let result = Handle::from(id);

        assert_eq!(id, result.handle);
    }
}
