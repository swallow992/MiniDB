//! SQL lexical analyzer

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Placeholder tokens
    EOF,
}

pub struct Lexer {
    // TODO: Add lexer state
}

#[derive(Error, Debug)]
pub enum LexError {
    #[error("Not implemented")]
    NotImplemented,
}

impl Lexer {
    pub fn new(_input: &str) -> Self {
        Self {}
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        Err(LexError::NotImplemented)
    }
}
