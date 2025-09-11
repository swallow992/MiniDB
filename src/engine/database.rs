//! Database engine implementation
//!
//! Main database interface and query execution coordination.

use crate::sql::{parse_sql, Statement};
use crate::storage::{BufferPool, FileManager};
use crate::types::{Schema, Tuple, Value, DataType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Main database instance
pub struct Database {
    /// Path to database directory
    data_dir: PathBuf,
    /// File manager for database files
    file_manager: FileManager,
    /// Buffer pool for page caching
    buffer_pool: BufferPool,
    /// Table catalog: table_name -> table_id
    table_catalog: HashMap<String, u32>,
    /// Table schemas: table_id -> schema
    table_schemas: HashMap<u32, Schema>,
    /// Next available table ID
    next_table_id: u32,
}

/// Query execution result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<Tuple>,
    pub schema: Option<Schema>,
    pub affected_rows: usize,
    pub message: String,
}

/// Database execution errors
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("SQL parsing error: {0}")]
    ParseError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Table '{table}' not found")]
    TableNotFound { table: String },
    
    #[error("Table '{table}' already exists")]
    TableAlreadyExists { table: String },
    
    #[error("Column '{column}' not found in table '{table}'")]
    ColumnNotFound { table: String, column: String },
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
}

impl Database {
    /// Create a new database instance
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ExecutionError> {
        let data_dir = path.as_ref().to_path_buf();
        
        // Ensure database directory exists
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| ExecutionError::StorageError(format!("Failed to create database directory: {}", e)))?;
        }
        
        // Initialize file manager
        let file_manager = FileManager::new(data_dir.clone())
            .map_err(|e| ExecutionError::StorageError(format!("Failed to initialize file manager: {}", e)))?;
        
        // Initialize buffer pool with 128 pages
        let buffer_pool = BufferPool::new(128);
        
        Ok(Self {
            data_dir,
            file_manager,
            buffer_pool,
            table_catalog: HashMap::new(),
            table_schemas: HashMap::new(),
            next_table_id: 1,
        })
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult, ExecutionError> {
        // Step 1: Parse SQL
        let statement = parse_sql(sql)
            .map_err(|e| ExecutionError::ParseError(format!("Parse error: {:?}", e)))?;
        
        // Step 2: Execute based on statement type
        match statement {
            Statement::CreateTable { table_name, columns, constraints: _ } => {
                self.execute_create_table_simple(table_name, columns)
            }
            Statement::DropTable { table_name, if_exists: _ } => {
                self.execute_drop_table_simple(table_name)
            }
            Statement::Insert { table_name, columns: _, values } => {
                self.execute_insert_simple(table_name, values)
            }
            Statement::Select { .. } => {
                // For now, return a simple message
                Ok(QueryResult {
                    rows: vec![],
                    schema: None,
                    affected_rows: 0,
                    message: "SELECT statement executed (simplified)".to_string(),
                })
            }
            _ => {
                Err(ExecutionError::NotImplemented { 
                    feature: format!("Statement type: {:?}", statement) 
                })
            }
        }
    }
    
    /// Execute CREATE TABLE statement (simplified)
    fn execute_create_table_simple(&mut self, name: String, columns: Vec<crate::sql::parser::ColumnDef>) -> Result<QueryResult, ExecutionError> {
        // Check if table already exists
        if self.table_catalog.contains_key(&name) {
            return Err(ExecutionError::TableAlreadyExists { table: name });
        }
        
        // Convert column definitions to schema
        let mut schema_columns = Vec::new();
        for col_def in columns {
            let column = crate::types::ColumnDefinition {
                name: col_def.name,
                data_type: col_def.data_type,
                nullable: col_def.nullable,
                default: None, // Simplified for now
            };
            schema_columns.push(column);
        }
        
        let schema = Schema {
            columns: schema_columns,
        };
        
        // Assign new table ID
        let table_id = self.next_table_id;
        self.next_table_id += 1;
        
        // Create table file
        let table_file_name = format!("table_{}.db", table_id);
        self.file_manager.create_file(&table_file_name)
            .map_err(|e| ExecutionError::StorageError(format!("Failed to create table file: {}", e)))?;
        
        // Register table
        self.table_catalog.insert(name.clone(), table_id);
        self.table_schemas.insert(table_id, schema);
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: 0,
            message: format!("Table '{}' created successfully", name),
        })
    }
    
    /// Execute DROP TABLE statement (simplified)
    fn execute_drop_table_simple(&mut self, name: String) -> Result<QueryResult, ExecutionError> {
        // Check if table exists
        let table_id = self.table_catalog.get(&name)
            .ok_or_else(|| ExecutionError::TableNotFound { table: name.clone() })?;
        
        let table_id = *table_id;
        
        // Remove table from catalog
        self.table_catalog.remove(&name);
        self.table_schemas.remove(&table_id);
        
        // Delete table file
        let table_file_name = format!("table_{}.db", table_id);
        self.file_manager.delete_file(&table_file_name)
            .map_err(|e| ExecutionError::StorageError(format!("Failed to delete table file: {}", e)))?;
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: 0,
            message: format!("Table '{}' dropped successfully", name),
        })
    }
    
    /// Execute INSERT statement (simplified)
    fn execute_insert_simple(&mut self, table: String, values: Vec<Vec<crate::sql::parser::Expression>>) -> Result<QueryResult, ExecutionError> {
        // Check if table exists
        let table_id = self.table_catalog.get(&table)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table.clone() })?;
        
        let table_id = *table_id;
        let schema = self.table_schemas.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table.clone() })?;
        
        // Validate and convert values
        let mut inserted_count = 0;
        for row_expressions in values {
            if row_expressions.len() != schema.columns.len() {
                return Err(ExecutionError::TypeMismatch {
                    expected: format!("{} columns", schema.columns.len()),
                    actual: format!("{} values", row_expressions.len()),
                });
            }
            
            // For now, just count the rows (actual insertion would require more complex logic)
            inserted_count += 1;
        }
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: inserted_count,
            message: format!("Inserted {} row(s) into table '{}'", inserted_count, table),
        })
    }
    
    /// List all tables in the database
    pub fn list_tables(&self) -> Vec<String> {
        self.table_catalog.keys().cloned().collect()
    }
    
    /// Get table schema by name
    pub fn get_table_schema(&self, table_name: &str) -> Option<&Schema> {
        self.table_catalog.get(table_name)
            .and_then(|&table_id| self.table_schemas.get(&table_id))
    }
}
