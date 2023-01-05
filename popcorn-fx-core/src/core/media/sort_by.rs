pub struct SortBy {
    key: String,
    text: String
}

impl SortBy {
    pub fn new(key: String, text: String) -> Self {
        Self {
            key,
            text
        }
    }
}