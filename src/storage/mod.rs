//! Storage system module
//!
//! This module provides low-level storage functionality including
//! page management, buffer pool, and file system operations.

pub mod buffer;
pub mod file;
pub mod index;
pub mod page;

// Re-export commonly used types
pub use buffer::{BufferError, BufferPool, FrameId};
pub use file::{DatabaseFile, FileError, FileManager};
pub use index::{BPlusTreeIndex, Index, IndexError};
pub use page::{Page, PageError, PageId, PageType, SlotId};

use thiserror::Error;

/// Storage system errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Page error: {0}")]
    Page(#[from] PageError),

    #[error("Buffer pool error: {0}")]
    Buffer(#[from] BufferError),

    #[error("File system error: {0}")]
    File(#[from] FileError),

    #[error("Index error: {0}")]
    Index(#[from] IndexError),
}
