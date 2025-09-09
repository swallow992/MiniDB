//! Table management

use thiserror::Error;

pub type TableId = u32;

pub struct Table {
    // TODO: Add table data
}

#[derive(Error, Debug)]
pub enum TableError {
    #[error("Not implemented")]
    NotImplemented,
}

impl Table {
    pub fn new() -> Self {
        Self {}
    }
}
