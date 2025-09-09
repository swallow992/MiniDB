//! File system management

use thiserror::Error;

pub struct FileManager {
    // TODO: Add file manager data
}

pub struct DatabaseFile {
    // TODO: Add database file data
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Not implemented")]
    NotImplemented,
}

impl FileManager {
    pub fn new() -> Self {
        Self {}
    }
}
