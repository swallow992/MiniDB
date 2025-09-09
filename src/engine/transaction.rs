//! Transaction management

use thiserror::Error;

pub struct Transaction {
    // TODO: Add transaction data
}

pub struct TransactionManager {
    // TODO: Add transaction manager data
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Not implemented")]
    NotImplemented,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {}
    }
}
