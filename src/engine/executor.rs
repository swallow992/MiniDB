//! Query executor

use crate::types::Tuple;
use thiserror::Error;

pub trait Executor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError>;
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<Tuple>,
}

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Not implemented")]
    NotImplemented,
}
