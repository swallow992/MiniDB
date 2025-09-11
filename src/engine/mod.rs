//! Query execution engine
//!
//! This module provides the core database functionality including
//! query execution, table management, and transaction processing.

pub mod database;
pub mod executor;
pub mod table;
pub mod transaction;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use database::{Database, QueryResult};
pub use executor::{Executor, ExecutorError};
pub use table::{Table, TableError, TableId};
pub use transaction::{Transaction, TransactionError, TransactionManager};
