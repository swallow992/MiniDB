//! Index structures
//!
//! This module implements various index structures for efficient data retrieval.
//! Currently supports B+ Tree indices with extensible design for other index types.

use crate::storage::page::PageId;
use crate::types::{DataType, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use thiserror::Error;

/// Index key type that can hold various data types
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IndexKey {
    values: Vec<Value>,
}

/// Index entry containing key and record identifier
#[derive(Debug, Clone, PartialEq)]
pub struct IndexEntry {
    pub key: IndexKey,
    pub rid: RecordId,
}

/// Record identifier (page_id, slot_id)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordId {
    pub page_id: PageId,
    pub slot_id: u16,
}

/// Index iterator for range scans
pub struct IndexIterator {
    entries: Vec<IndexEntry>,
    current: usize,
}

/// Generic index trait
pub trait Index {
    /// Insert a key-value pair into the index
    fn insert(&mut self, key: IndexKey, rid: RecordId) -> Result<(), IndexError>;

    /// Delete a key from the index
    fn delete(&mut self, key: &IndexKey) -> Result<bool, IndexError>;

    /// Search for a specific key
    fn search(&self, key: &IndexKey) -> Result<Option<RecordId>, IndexError>;

    /// Range scan from start_key to end_key (inclusive)
    fn range_scan(
        &self,
        start_key: Option<&IndexKey>,
        end_key: Option<&IndexKey>,
    ) -> Result<IndexIterator, IndexError>;

    /// Get index size (number of entries)
    fn size(&self) -> usize;

    /// Check if index is empty
    fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

/// B+ Tree index implementation (simplified in-memory version)
pub struct BPlusTreeIndex {
    /// In-memory B+ tree using BTreeMap
    tree: BTreeMap<IndexKey, RecordId>,
    /// Index metadata
    key_types: Vec<DataType>,
}

/// Index-related errors
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Key not found: {0:?}")]
    KeyNotFound(IndexKey),

    #[error("Duplicate key: {0:?}")]
    DuplicateKey(IndexKey),

    #[error("Invalid key format: expected {expected} columns, got {actual}")]
    InvalidKeyFormat { expected: usize, actual: usize },

    #[error("Type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },

    #[error("Index is corrupted: {reason}")]
    IndexCorrupted { reason: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl IndexKey {
    /// Create a new index key
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Create a single-column key
    pub fn single(value: Value) -> Self {
        Self::new(vec![value])
    }

    /// Get the values in the key
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Get number of columns in the key
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if key is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl PartialOrd for IndexKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IndexKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare lexicographically
        for (a, b) in self.values.iter().zip(other.values.iter()) {
            match a.partial_cmp(b) {
                Some(Ordering::Equal) => continue,
                Some(ordering) => return ordering,
                None => return Ordering::Equal, // Handle incomparable values
            }
        }

        // If all compared values are equal, compare lengths
        self.values.len().cmp(&other.values.len())
    }
}

impl Eq for IndexKey {}

impl RecordId {
    /// Create a new record ID
    pub fn new(page_id: PageId, slot_id: u16) -> Self {
        Self { page_id, slot_id }
    }
}

impl IndexEntry {
    /// Create a new index entry
    pub fn new(key: IndexKey, rid: RecordId) -> Self {
        Self { key, rid }
    }
}

impl IndexIterator {
    fn new(entries: Vec<IndexEntry>) -> Self {
        Self {
            entries,
            current: 0,
        }
    }

    /// Check if iterator has more entries
    pub fn has_next(&self) -> bool {
        self.current < self.entries.len()
    }

    /// Get next entry
    pub fn next(&mut self) -> Option<&IndexEntry> {
        if self.has_next() {
            let entry = &self.entries[self.current];
            self.current += 1;
            Some(entry)
        } else {
            None
        }
    }

    /// Reset iterator to beginning
    pub fn reset(&mut self) {
        self.current = 0;
    }

    /// Get all remaining entries
    pub fn collect(mut self) -> Vec<IndexEntry> {
        let mut result = Vec::new();
        while let Some(entry) = self.next() {
            result.push(entry.clone());
        }
        result
    }
}

impl BPlusTreeIndex {
    /// Create a new B+ tree index
    pub fn new(key_types: Vec<DataType>) -> Self {
        Self {
            tree: BTreeMap::new(),
            key_types,
        }
    }

    /// Validate key format against expected types
    fn validate_key(&self, key: &IndexKey) -> Result<(), IndexError> {
        if key.len() != self.key_types.len() {
            return Err(IndexError::InvalidKeyFormat {
                expected: self.key_types.len(),
                actual: key.len(),
            });
        }

        for (value, expected_type) in key.values().iter().zip(&self.key_types) {
            let value_type = value.data_type();
            if !value_type.is_compatible_with(expected_type) {
                return Err(IndexError::TypeMismatch {
                    expected: expected_type.clone(),
                    actual: value_type,
                });
            }
        }

        Ok(())
    }
}

impl Index for BPlusTreeIndex {
    fn insert(&mut self, key: IndexKey, rid: RecordId) -> Result<(), IndexError> {
        self.validate_key(&key)?;

        if self.tree.contains_key(&key) {
            return Err(IndexError::DuplicateKey(key));
        }

        self.tree.insert(key, rid);
        Ok(())
    }

    fn delete(&mut self, key: &IndexKey) -> Result<bool, IndexError> {
        self.validate_key(key)?;

        Ok(self.tree.remove(key).is_some())
    }

    fn search(&self, key: &IndexKey) -> Result<Option<RecordId>, IndexError> {
        self.validate_key(key)?;

        Ok(self.tree.get(key).cloned())
    }

    fn range_scan(
        &self,
        start_key: Option<&IndexKey>,
        end_key: Option<&IndexKey>,
    ) -> Result<IndexIterator, IndexError> {
        if let Some(key) = start_key {
            self.validate_key(key)?;
        }
        if let Some(key) = end_key {
            self.validate_key(key)?;
        }

        let mut entries = Vec::new();

        // Collect entries in range
        for (key, rid) in &self.tree {
            let in_range = match (start_key, end_key) {
                (Some(start), Some(end)) => key >= start && key <= end,
                (Some(start), None) => key >= start,
                (None, Some(end)) => key <= end,
                (None, None) => true,
            };

            if in_range {
                entries.push(IndexEntry::new(key.clone(), *rid));
            }
        }

        Ok(IndexIterator::new(entries))
    }

    fn size(&self) -> usize {
        self.tree.len()
    }
}

/// Hash index implementation (for equality lookups)
#[derive(Debug)]
pub struct HashIndex {
    buckets: std::collections::HashMap<IndexKey, RecordId>,
    key_types: Vec<DataType>,
}

impl HashIndex {
    /// Create a new hash index
    pub fn new(key_types: Vec<DataType>) -> Self {
        Self {
            buckets: std::collections::HashMap::new(),
            key_types,
        }
    }

    /// Validate key format
    fn validate_key(&self, key: &IndexKey) -> Result<(), IndexError> {
        if key.len() != self.key_types.len() {
            return Err(IndexError::InvalidKeyFormat {
                expected: self.key_types.len(),
                actual: key.len(),
            });
        }
        Ok(())
    }
}

impl Index for HashIndex {
    fn insert(&mut self, key: IndexKey, rid: RecordId) -> Result<(), IndexError> {
        self.validate_key(&key)?;

        if self.buckets.contains_key(&key) {
            return Err(IndexError::DuplicateKey(key));
        }

        self.buckets.insert(key, rid);
        Ok(())
    }

    fn delete(&mut self, key: &IndexKey) -> Result<bool, IndexError> {
        self.validate_key(key)?;
        Ok(self.buckets.remove(key).is_some())
    }

    fn search(&self, key: &IndexKey) -> Result<Option<RecordId>, IndexError> {
        self.validate_key(key)?;
        Ok(self.buckets.get(key).cloned())
    }

    fn range_scan(
        &self,
        start_key: Option<&IndexKey>,
        end_key: Option<&IndexKey>,
    ) -> Result<IndexIterator, IndexError> {
        if start_key.is_some() || end_key.is_some() {
            // Hash index doesn't support efficient range scans
            // Fall back to full scan with filtering
            let mut entries = Vec::new();

            for (key, rid) in &self.buckets {
                let in_range = match (start_key, end_key) {
                    (Some(start), Some(end)) => key >= start && key <= end,
                    (Some(start), None) => key >= start,
                    (None, Some(end)) => key <= end,
                    (None, None) => true,
                };

                if in_range {
                    entries.push(IndexEntry::new(key.clone(), *rid));
                }
            }

            // Sort entries for consistent ordering
            entries.sort_by(|a, b| a.key.cmp(&b.key));

            Ok(IndexIterator::new(entries))
        } else {
            // Full scan
            let mut entries: Vec<_> = self
                .buckets
                .iter()
                .map(|(k, r)| IndexEntry::new(k.clone(), *r))
                .collect();
            entries.sort_by(|a, b| a.key.cmp(&b.key));
            Ok(IndexIterator::new(entries))
        }
    }

    fn size(&self) -> usize {
        self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_key_ordering() {
        let key1 = IndexKey::single(Value::Integer(1));
        let key2 = IndexKey::single(Value::Integer(2));
        let key3 = IndexKey::single(Value::Integer(1));

        assert!(key1 < key2);
        assert!(key1 == key3);
        assert!(key2 > key1);
    }

    #[test]
    fn test_btree_index_basic_operations() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);

        let key1 = IndexKey::single(Value::Integer(1));
        let key2 = IndexKey::single(Value::Integer(2));
        let rid1 = RecordId::new(1, 0);
        let rid2 = RecordId::new(1, 1);

        // Insert
        index.insert(key1.clone(), rid1).unwrap();
        index.insert(key2.clone(), rid2).unwrap();

        assert_eq!(index.size(), 2);

        // Search
        assert_eq!(index.search(&key1).unwrap(), Some(rid1));
        assert_eq!(index.search(&key2).unwrap(), Some(rid2));

        let nonexistent = IndexKey::single(Value::Integer(999));
        assert_eq!(index.search(&nonexistent).unwrap(), None);

        // Delete
        assert!(index.delete(&key1).unwrap());
        assert!(!index.delete(&key1).unwrap()); // Already deleted
        assert_eq!(index.size(), 1);
    }

    #[test]
    fn test_btree_range_scan() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);

        // Insert some data
        for i in 1..=10 {
            let key = IndexKey::single(Value::Integer(i));
            let rid = RecordId::new(1, i as u16);
            index.insert(key, rid).unwrap();
        }

        // Range scan [3, 7]
        let start_key = IndexKey::single(Value::Integer(3));
        let end_key = IndexKey::single(Value::Integer(7));
        let iter = index.range_scan(Some(&start_key), Some(&end_key)).unwrap();

        let results = iter.collect();
        assert_eq!(results.len(), 5); // 3, 4, 5, 6, 7

        for (i, entry) in results.iter().enumerate() {
            let expected_value = (i as i32) + 3;
            assert_eq!(entry.key, IndexKey::single(Value::Integer(expected_value)));
        }
    }

    #[test]
    fn test_hash_index_operations() {
        let mut index = HashIndex::new(vec![DataType::Varchar(50)]);

        let key1 = IndexKey::single(Value::Varchar("alice".to_string()));
        let key2 = IndexKey::single(Value::Varchar("bob".to_string()));
        let rid1 = RecordId::new(1, 0);
        let rid2 = RecordId::new(1, 1);

        // Insert
        index.insert(key1.clone(), rid1).unwrap();
        index.insert(key2.clone(), rid2).unwrap();

        // Search
        assert_eq!(index.search(&key1).unwrap(), Some(rid1));
        assert_eq!(index.search(&key2).unwrap(), Some(rid2));

        // Delete
        assert!(index.delete(&key1).unwrap());
        assert_eq!(index.size(), 1);
    }

    #[test]
    fn test_multi_column_index() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Varchar(50), DataType::Integer]);

        let key1 = IndexKey::new(vec![
            Value::Varchar("alice".to_string()),
            Value::Integer(25),
        ]);
        let key2 = IndexKey::new(vec![Value::Varchar("bob".to_string()), Value::Integer(30)]);

        let rid1 = RecordId::new(1, 0);
        let rid2 = RecordId::new(1, 1);

        index.insert(key1.clone(), rid1).unwrap();
        index.insert(key2.clone(), rid2).unwrap();

        assert_eq!(index.search(&key1).unwrap(), Some(rid1));
        assert_eq!(index.search(&key2).unwrap(), Some(rid2));
    }

    #[test]
    fn test_duplicate_key_error() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);

        let key = IndexKey::single(Value::Integer(1));
        let rid1 = RecordId::new(1, 0);
        let rid2 = RecordId::new(1, 1);

        index.insert(key.clone(), rid1).unwrap();

        // Try to insert duplicate
        let result = index.insert(key, rid2);
        assert!(matches!(result, Err(IndexError::DuplicateKey(_))));
    }

    #[test]
    fn test_invalid_key_format() {
        let mut index = BPlusTreeIndex::new(vec![DataType::Integer]);

        // Try to insert key with wrong number of columns
        let wrong_key = IndexKey::new(vec![Value::Integer(1), Value::Integer(2)]);
        let rid = RecordId::new(1, 0);

        let result = index.insert(wrong_key, rid);
        assert!(matches!(result, Err(IndexError::InvalidKeyFormat { .. })));
    }
}
