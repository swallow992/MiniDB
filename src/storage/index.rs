//! Index structures

use thiserror::Error;

pub trait Index {
    // TODO: Add index trait methods
}

pub struct BPlusTreeIndex {
    // TODO: Add B+ tree data
}

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Not implemented")]
    NotImplemented,
}

impl Index for BPlusTreeIndex {
    // TODO: Implement index methods
}
