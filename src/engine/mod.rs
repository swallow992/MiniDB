//! 查询执行引擎
//!
//! 此模块提供核心数据库功能，包括
//! 查询执行、表管理和事务处理。

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
