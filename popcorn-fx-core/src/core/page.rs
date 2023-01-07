use derive_more::Display;

/// A page is a vector of items.
/// It allows to gain information about the position of it in the containing entire collection.
#[derive(Debug, Clone, PartialEq, Display)]
#[display(fmt = "page: {}, total_pages: {}, total_elements: {}", page, total_pages, total_elements)]
pub struct Page<T> {
    page: u32,
    total_pages: u32,
    total_elements: u32,
    content: Vec<T>,
}

impl<T> Page<T> {
    /// Create a new page from the given content.
    /// It takes the current page and the total pages as additional arguments.
    pub fn from(content: Vec<T>, page: u32, total_pages: u32, total_elements: u32) -> Self {
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
            total_elements: content.len() as u32,
            content,
        }
    }

    /// Create a new empty page.
    pub fn empty() -> Self {
        Self {
            page: 1,
            total_pages: 1,
            total_elements: 0,
            content: vec![],
        }
    }

    /// Verify if this is the last page.
    pub fn is_last(&self) -> bool {
        self.page < self.total_pages
    }

    /// Retrieve the current page index.
    pub fn page(&self) -> u32 {
        self.page.clone()
    }

    /// Retrieve the total pages.
    pub fn total_pages(&self) -> u32 {
        self.total_pages.clone()
    }

    /// Retrieve the total amount of items across all pages.
    pub fn total_elements(&self) -> u32 {
        self.total_elements.clone()
    }

    /// Retrieve the size of the current page.
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Retrieve the page request for the next page.
    ///
    /// It returns [Option::None] when there is no next page available.
    pub fn next(&self) -> Option<PageRequest> {
        if self.page == self.total_pages {
            return None;
        }

        Some(PageRequest::new(self.page + 1, None))
    }
    
    /// Retrieve the content of the page.
    pub fn content(&self) -> &Vec<T> {
        &self.content
    }
    
    /// Move the content outside of this [Page].
    /// This means the content is consumed and cannot be used anymore afterwards.
    pub fn into_content(self) -> Vec<T> {
        self.content
    }
}

/// A request for retrieving a new data page.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRequest {
    page: u32,
    size: Option<u32>,
}

impl PageRequest {
    pub fn new(page: u32, size: Option<u32>) -> Self {
        Self {
            page,
            size,
        }
    }
}