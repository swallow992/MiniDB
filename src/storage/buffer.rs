//! Buffer pool management

use thiserror::Error;

pub type FrameId = usize;

pub struct BufferPool {
    // TODO: Add buffer pool data
}

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Not implemented")]
    NotImplemented,
}

impl BufferPool {
    pub fn new(_size: usize) -> Self {
        Self {}
    }
}
