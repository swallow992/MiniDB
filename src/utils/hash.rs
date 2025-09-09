//! Hash functions

/// FNV-1a hash algorithm
pub fn fnv1a_hash(data: &[u8]) -> u32 {
    const FNV_PRIME: u32 = 16777619;
    const FNV_OFFSET: u32 = 2166136261;
    
    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Simple CRC32 implementation
pub fn crc32_hash(data: &[u8]) -> u32 {
    // Placeholder implementation
    fnv1a_hash(data)
}
