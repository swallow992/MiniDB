//! MiniDB - A small database system built with Rust
//!
//! This is the main library crate that provides all the core functionality
//! for the MiniDB database system.

pub mod engine;
pub mod sql;
pub mod storage;
pub mod types;
pub mod utils;

#[cfg(test)]
mod test_enhancements;

// Re-export commonly used types
pub use engine::{Database, QueryResult};
pub use sql::{ParseError, Statement};
pub use storage::{Page, StorageError};
pub use types::{DataType, Schema, Tuple, Value};

/// Database version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default page size in bytes (4KB)
pub const DEFAULT_PAGE_SIZE: usize = 4096;

/// Default buffer pool size in pages
pub const DEFAULT_BUFFER_POOL_SIZE: usize = 1000;
