#[derive(Debug, Clone)]
pub struct SortBy {
    key: String,
    text: String,
}

impl SortBy {
    pub fn new(key: String, text: String) -> Self {
        Self {
            key,
            text,
        }
    }

    pub fn key(&self) -> &String {
        &self.key
    }

    pub fn text(&self) -> &String {
        &self.text
    }
}