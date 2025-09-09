//! SQL parser

use crate::sql::lexer::{Lexer, Token};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Statement {
    // Placeholder statement
    Select,
}

pub struct Parser {
    // TODO: Add parser state
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Not implemented")]
    NotImplemented,
}

impl Parser {
    pub fn new(_lexer: Lexer) -> Result<Self, ParseError> {
        Ok(Self {})
    }
    
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::NotImplemented)
    }
}
