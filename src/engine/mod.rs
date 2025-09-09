//! Query execution engine
//!
//! This module provides the core database functionality including
//! query execution, table management, and transaction processing.

pub mod database;
pub mod executor;
pub mod table;
pub mod transaction;

// Re-export commonly used types
pub use database::Database;
pub use executor::{Executor, ExecutorError, QueryResult};
pub use table::{Table, TableError, TableId};
pub use transaction::{Transaction, TransactionError, TransactionManager};

use thiserror::Error;

/// Engine execution errors
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("SQL parsing error: {0}")]
    Parse(#[from] crate::sql::ParseError),

    #[error("Semantic analysis error: {0}")]
    Semantic(#[from] crate::sql::SemanticError),

    #[error("Query planning error: {0}")]
    Planning(#[from] crate::sql::PlanError),

    #[error("Execution error: {0}")]
    Executor(#[from] ExecutorError),

    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
}
