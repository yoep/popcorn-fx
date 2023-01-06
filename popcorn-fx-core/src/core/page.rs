/// A page is a vector of items.
/// It allows to gain information about the position of it in the containing entire collection.
#[derive(Debug, Clone, PartialEq)]
pub struct Page<T> {
    page: i32,
    total_pages: i32,
    total_elements: i32,
    content: Vec<T>,
}

impl<T> Page<T> {
    /// Create a new page from the given content.
    /// It takes the current page and the total pages as additional arguments.
    pub fn from(content: Vec<T>, page: i32, total_pages: i32, total_elements: i32) -> Self {
        Self {
            page,
            total_pages,
            total_elements,
            content,
        }
    }

    /// Create a new page with only the given content.
    /// This will return a page of size 1.
    pub fn from_content(content: Vec<T>) -> Self {
        Self {
            page: 1,
            total_pages: 1,
            total_elements: content.len() as i32,
            content,
        }
    }

    /// Create a new empty page.
    pub fn empty() -> Self {
        Self {
            page: 1,
            total_pages: 1,
            total_elements: 0,
            content: vec![]
        }
    }

    /// Verify if this is the last page.
    pub fn is_last(&self) -> bool {
        self.page < self.total_pages
    }

    /// Retrieve the current page index.
    pub fn page(&self) -> i32 {
        self.page.clone()
    }
    
    /// Retrieve the total pages.
    pub fn total_pages(&self) -> i32 {
        self.total_pages.clone()
    }

    /// Retrieve the total amount of items across all pages.
    pub fn total_elements(&self) -> i32 {
        self.total_elements.clone()
    }

    /// Retrieve the size of the current page.
    pub fn size(&self) -> usize {
        self.content.len()
    }
}