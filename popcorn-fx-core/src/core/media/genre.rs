pub struct Genre {
    key: String,
    text: String
}

impl Genre {
    /// Create a new genre.
    pub fn new(key: String, text: String) -> Self {
        Self {
            key,
            text
        }
    }

    /// Create the "all" genre.
    pub fn all() -> Self {
        Self {
            key: "all".to_string(),
            text: "All".to_string()
        }
    }
}