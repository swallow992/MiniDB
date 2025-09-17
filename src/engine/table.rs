//! 表管理

use crate::storage::index::{BPlusTreeIndex, Index, IndexKey, RecordId, IndexError};
use crate::types::{Schema, Tuple};
use std::collections::HashMap;
use thiserror::Error;

pub type TableId = u32;

/// 支持索引的表
pub struct Table {
    /// 表标识符
    pub id: TableId,
    /// 表名
    pub name: String,
    /// 表模式
    pub schema: Schema,
    /// 主索引（如果有的话）
    pub primary_index: Option<BPlusTreeIndex>,
    /// 辅助索引：索引名 -> 索引
    pub secondary_indices: HashMap<String, BPlusTreeIndex>,
    /// 索引元数据：索引名 -> (列索引, 是否唯一)
    pub index_metadata: HashMap<String, (Vec<usize>, bool)>,
}

#[derive(Error, Debug)]
pub enum TableError {
    #[error("Index error: {0}")]
    IndexError(#[from] IndexError),
    
    #[error("Index already exists: {name}")]
    IndexAlreadyExists { name: String },
    
    #[error("Index not found: {name}")]
    IndexNotFound { name: String },
    
    #[error("Column not found: {column}")]
    ColumnNotFound { column: String },
    
    #[error("Invalid index definition: {reason}")]
    InvalidIndexDefinition { reason: String },
    
    #[error("Not implemented")]
    NotImplemented,
}

impl Table {
    /// 创建一个新表
    pub fn new(id: TableId, name: String, schema: Schema) -> Self {
        Self {
            id,
            name,
            schema,
            primary_index: None,
            secondary_indices: HashMap::new(),
            index_metadata: HashMap::new(),
        }
    }
    
    /// 在指定列上创建主索引
    pub fn create_primary_index(&mut self, column_names: Vec<String>) -> Result<(), TableError> {
        let column_indices = self.resolve_column_indices(&column_names)?;
        let column_types = column_indices.iter()
            .map(|&idx| self.schema.columns[idx].data_type.clone())
            .collect();
            
        let index = BPlusTreeIndex::new(column_types);
        self.primary_index = Some(index);
        self.index_metadata.insert("PRIMARY".to_string(), (column_indices, true));
        
        Ok(())
    }
    
    /// 创建辅助索引
    pub fn create_index(
        &mut self, 
        index_name: String, 
        column_names: Vec<String>,
        is_unique: bool
    ) -> Result<(), TableError> {
        if self.secondary_indices.contains_key(&index_name) {
            return Err(TableError::IndexAlreadyExists { name: index_name });
        }
        
        let column_indices = self.resolve_column_indices(&column_names)?;
        let column_types = column_indices.iter()
            .map(|&idx| self.schema.columns[idx].data_type.clone())
            .collect();
            
        let index = BPlusTreeIndex::new(column_types);
        self.secondary_indices.insert(index_name.clone(), index);
        self.index_metadata.insert(index_name, (column_indices, is_unique));
        
        Ok(())
    }
    
    /// Drop an index
    pub fn drop_index(&mut self, index_name: &str) -> Result<(), TableError> {
        if index_name == "PRIMARY" {
            self.primary_index = None;
            self.index_metadata.remove("PRIMARY");
        } else {
            self.secondary_indices.remove(index_name)
                .ok_or_else(|| TableError::IndexNotFound { name: index_name.to_string() })?;
            self.index_metadata.remove(index_name);
        }
        
        Ok(())
    }
    
    /// Insert a tuple and update all indices
    pub fn insert_with_indices(&mut self, tuple: &Tuple, record_id: RecordId) -> Result<(), TableError> {
        // Update primary index
        if self.primary_index.is_some() {
            if let Some((column_indices, _)) = self.index_metadata.get("PRIMARY").cloned() {
                let key = Self::extract_key_values_static(tuple, &column_indices)?;
                if let Some(ref mut primary_index) = self.primary_index {
                    primary_index.insert(key, record_id)?;
                }
            }
        }
        
        // Collect secondary index operations to avoid borrowing conflicts
        let mut operations = Vec::new();
        for index_name in self.secondary_indices.keys() {
            if let Some((column_indices, _)) = self.index_metadata.get(index_name).cloned() {
                let key = Self::extract_key_values_static(tuple, &column_indices)?;
                operations.push((index_name.clone(), key));
            }
        }
        
        // Apply secondary index operations
        for (index_name, key) in operations {
            if let Some(index) = self.secondary_indices.get_mut(&index_name) {
                index.insert(key, record_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Delete a tuple and update all indices
    pub fn delete_from_indices(&mut self, tuple: &Tuple) -> Result<(), TableError> {
        // Update primary index
        if self.primary_index.is_some() {
            if let Some((column_indices, _)) = self.index_metadata.get("PRIMARY").cloned() {
                let key = Self::extract_key_values_static(tuple, &column_indices)?;
                if let Some(ref mut primary_index) = self.primary_index {
                    primary_index.delete(&key)?;
                }
            }
        }
        
        // Collect secondary index operations to avoid borrowing conflicts
        let mut operations = Vec::new();
        for index_name in self.secondary_indices.keys() {
            if let Some((column_indices, _)) = self.index_metadata.get(index_name).cloned() {
                let key = Self::extract_key_values_static(tuple, &column_indices)?;
                operations.push((index_name.clone(), key));
            }
        }
        
        // Apply secondary index operations
        for (index_name, key) in operations {
            if let Some(index) = self.secondary_indices.get_mut(&index_name) {
                index.delete(&key)?;
            }
        }
        
        Ok(())
    }
    
    /// Get index by name
    pub fn get_index(&self, index_name: &str) -> Option<&BPlusTreeIndex> {
        if index_name == "PRIMARY" {
            self.primary_index.as_ref()
        } else {
            self.secondary_indices.get(index_name)
        }
    }
    
    /// Get mutable index by name
    pub fn get_index_mut(&mut self, index_name: &str) -> Option<&mut BPlusTreeIndex> {
        if index_name == "PRIMARY" {
            self.primary_index.as_mut()
        } else {
            self.secondary_indices.get_mut(index_name)
        }
    }
    
    /// List all index names
    pub fn list_indices(&self) -> Vec<String> {
        let mut indices = Vec::new();
        if self.primary_index.is_some() {
            indices.push("PRIMARY".to_string());
        }
        indices.extend(self.secondary_indices.keys().cloned());
        indices
    }
    
    /// Get index metadata
    pub fn get_index_metadata(&self, index_name: &str) -> Option<&(Vec<usize>, bool)> {
        self.index_metadata.get(index_name)
    }
    
    // Helper methods
    
    /// Resolve column names to indices
    fn resolve_column_indices(&self, column_names: &[String]) -> Result<Vec<usize>, TableError> {
        let mut indices = Vec::new();
        for column_name in column_names {
            let index = self.schema.columns.iter()
                .position(|col| col.name == *column_name)
                .ok_or_else(|| TableError::ColumnNotFound { column: column_name.clone() })?;
            indices.push(index);
        }
        Ok(indices)
    }
    
    /// Extract key values from tuple based on column indices (static version)
    fn extract_key_values_static(tuple: &Tuple, column_indices: &[usize]) -> Result<IndexKey, TableError> {
        let values = column_indices.iter()
            .map(|&idx| {
                if idx < tuple.values.len() {
                    Ok(tuple.values[idx].clone())
                } else {
                    Err(TableError::InvalidIndexDefinition { 
                        reason: format!("Column index {} out of range", idx) 
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
            
        Ok(IndexKey::new(values))
    }
    
    /// Extract key values from tuple based on column indices
    fn extract_key_values(&self, tuple: &Tuple, column_indices: &[usize]) -> Result<IndexKey, TableError> {
        Self::extract_key_values_static(tuple, column_indices)
    }
}
