//! Page management

use thiserror::Error;

pub type PageId = u32;
pub type SlotId = u16;

#[derive(Debug, Clone, Copy)]
pub enum PageType {
    Data = 1,
    Index = 2,
}

pub struct Page {
    // TODO: Add page data
}

#[derive(Error, Debug)]
pub enum PageError {
    #[error("Not implemented")]
    NotImplemented,
}

impl Page {
    pub fn new(_page_id: PageId, _page_type: PageType) -> Self {
        Self {}
    }
}
