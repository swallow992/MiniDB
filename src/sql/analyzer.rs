//! SQL semantic analyzer

use crate::sql::parser::Statement;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    // TODO: Add analyzed statement data
}

pub struct SemanticAnalyzer {
    // TODO: Add analyzer state
}

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Not implemented")]
    NotImplemented,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn analyze(&self, _stmt: Statement) -> Result<AnalyzedStatement, SemanticError> {
        Err(SemanticError::NotImplemented)
    }
}
