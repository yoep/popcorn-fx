use derive_more::Display;

/// Represents a genre with a key and text.
#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "genre {}", key)]
pub struct Genre {
    /// The key of the genre.
    key: String,
    /// The text description of the genre.
    text: String,
}

impl Genre {
    /// Creates a new `Genre` instance.
    ///
    /// # Arguments
    ///
    /// * `key` - A String representing the key of the genre.
    /// * `text` - A String representing the text description of the genre.
    ///
    /// # Returns
    ///
    /// A new `Genre` instance.
    pub fn new(key: String, text: String) -> Self {
        Self { key, text }
    }

    /// Creates a `Genre` instance representing "all" genres.
    ///
    /// # Returns
    ///
    /// A `Genre` instance representing "all" genres.
    pub fn all() -> Self {
        Self {
            key: "all".to_string(),
            text: "All".to_string(),
        }
    }

    /// Retrieves the key of the genre.
    ///
    /// # Returns
    ///
    /// A reference to the key of the genre.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Retrieves the text description of the genre.
    ///
    /// # Returns
    ///
    /// A reference to the text description of the genre.
    pub fn text(&self) -> &str {
        &self.text
    }
}
