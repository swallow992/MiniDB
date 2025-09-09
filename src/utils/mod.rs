//! Utility functions and helpers
//!
//! This module contains various utility functions used across the codebase.

pub mod bitset;
pub mod hash;
pub mod serialize;

/// Generate a simple hash for a byte slice
pub fn simple_hash(data: &[u8]) -> u32 {
    hash::fnv1a_hash(data)
}

/// Calculate a checksum for data integrity verification
pub fn checksum(data: &[u8]) -> u32 {
    hash::crc32_hash(data)
}

/// Align a size to the next multiple of alignment
pub fn align_to(size: usize, alignment: usize) -> usize {
    (size + alignment - 1) & !(alignment - 1)
}

/// Check if a number is a power of 2
pub fn is_power_of_2(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Round up to the next power of 2
pub fn next_power_of_2(n: usize) -> usize {
    if n == 0 {
        return 1;
    }

    let mut power = 1;
    while power < n {
        power <<= 1;
    }
    power
}
