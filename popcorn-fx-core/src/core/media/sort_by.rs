use derive_more::Display;

/// A struct representing a sorting criteria.
#[derive(Debug, Display, Clone, PartialEq)]
#[display("sort by {}", key)]
pub struct SortBy {
    /// The key used for sorting.
    key: String,
    /// The text describing the sorting criteria.
    text: String,
}

impl SortBy {
    /// Creates a new SortBy instance.
    ///
    /// # Arguments
    ///
    /// * `key` - A String representing the key used for sorting.
    /// * `text` - A String describing the sorting criteria.
    ///
    /// # Returns
    ///
    /// A new SortBy instance.
    pub fn new(key: String, text: String) -> Self {
        Self { key, text }
    }

    /// Retrieves the key used for sorting.
    ///
    /// # Returns
    ///
    /// A reference to the key used for sorting.
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Retrieves the text describing the sorting criteria.
    ///
    /// # Returns
    ///
    /// A reference to the text describing the sorting criteria.
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}
