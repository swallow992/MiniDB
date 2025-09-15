//! Database engine implementation
//!
//! Main database interface and query execution coordination.

use crate::sql::{parse_sql, Statement};
use crate::storage::{BufferPool, FileManager};
use crate::types::{Schema, Tuple, Value, DataType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// 表数据存储结构
#[derive(Serialize, Deserialize)]
struct TableData {
    schema: Schema,
    rows: Vec<Tuple>,
}

/// 数据库元数据存储结构
#[derive(Serialize, Deserialize)]
struct DatabaseMetadata {
    next_table_id: u32,
    table_catalog: HashMap<String, u32>,
}

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
    /// Table data: table_id -> rows (simplified in-memory storage)
    table_data: HashMap<u32, Vec<Tuple>>,
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
        
        let mut database = Self {
            data_dir,
            file_manager,
            buffer_pool,
            table_catalog: HashMap::new(),
            table_schemas: HashMap::new(),
            table_data: HashMap::new(),
            next_table_id: 1,
        };
        
        // Load existing data if available
        if let Err(e) = database.load_existing_tables() {
            println!("Warning: Failed to load existing tables: {}", e);
        }
        
        Ok(database)
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
            Statement::Select { select_list, from_clause, where_clause, .. } => {
                self.execute_select_simple(select_list, from_clause, where_clause)
            }
            Statement::Update { table_name, assignments, where_clause } => {
                self.execute_update_simple(table_name, assignments, where_clause)
            }
            Statement::Delete { table_name, where_clause } => {
                self.execute_delete_simple(table_name, where_clause)
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
        self.table_data.insert(table_id, Vec::new()); // Initialize empty data storage
        
        // Save table data and metadata
        if let Err(e) = self.save_table(table_id, &name) {
            println!("Warning: Failed to save table data: {}", e);
        }
        if let Err(e) = self.save_metadata() {
            println!("Warning: Failed to save metadata: {}", e);
        }
        
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
            
            // Convert expressions to values
            let mut row_values = Vec::new();
            for (i, expr) in row_expressions.iter().enumerate() {
                let value = self.evaluate_expression(expr, &schema.columns[i].data_type)?;
                row_values.push(value);
            }
            
            // Create tuple and add to table data
            let tuple = Tuple { values: row_values };
            self.table_data.get_mut(&table_id).unwrap().push(tuple);
            inserted_count += 1;
        }
        
        // Save table data after insertion
        if let Err(e) = self.save_table(table_id, &table) {
            println!("Warning: Failed to save table data: {}", e);
        }
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: inserted_count,
            message: format!("Inserted {} row(s) into table '{}'", inserted_count, table),
        })
    }
    
    /// Simple expression evaluation (for literals only)
    fn evaluate_expression(&self, expr: &crate::sql::parser::Expression, expected_type: &DataType) -> Result<Value, ExecutionError> {
        use crate::sql::parser::Expression;
        
        match expr {
            Expression::Literal(value) => {
                // Validate the value type matches expected type (with flexible conversion)
                match (value, expected_type) {
                    (Value::Integer(_), DataType::Integer) => Ok(value.clone()),
                    (Value::BigInt(_), DataType::BigInt) => Ok(value.clone()),
                    (Value::Float(_), DataType::Float) => Ok(value.clone()),
                    (Value::Double(d), DataType::Float) => Ok(Value::Float(*d as f32)), // Convert Double to Float
                    (Value::Float(f), DataType::Double) => Ok(Value::Double(*f as f64)), // Convert Float to Double
                    (Value::Double(_), DataType::Double) => Ok(value.clone()),
                    (Value::Varchar(_), DataType::Varchar(_)) => Ok(value.clone()),
                    (Value::Boolean(_), DataType::Boolean) => Ok(value.clone()),
                    (Value::Date(_), DataType::Date) => Ok(value.clone()),
                    (Value::Timestamp(_), DataType::Timestamp) => Ok(value.clone()),
                    (Value::Null, _) => Ok(Value::Null),
                    // Allow integer to bigint conversion
                    (Value::Integer(i), DataType::BigInt) => Ok(Value::BigInt(*i as i64)),
                    (Value::BigInt(i), DataType::Integer) => {
                        if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                            Ok(Value::Integer(*i as i32))
                        } else {
                            Err(ExecutionError::TypeMismatch {
                                expected: "INTEGER (in range)".to_string(),
                                actual: format!("BIGINT({})", i),
                            })
                        }
                    }
                    _ => Err(ExecutionError::TypeMismatch {
                        expected: format!("{:?}", expected_type),
                        actual: format!("{:?}", value),
                    })
                }
            }
            _ => Err(ExecutionError::NotImplemented {
                feature: format!("Expression evaluation: {:?}", expr)
            })
        }
    }
    
    /// Evaluate WHERE condition for a given row
    fn evaluate_where_condition(
        &self, 
        expr: &crate::sql::parser::Expression, 
        row: &Tuple, 
        schema: &Schema
    ) -> Result<bool, ExecutionError> {
        use crate::sql::parser::Expression;
        use crate::sql::parser::BinaryOperator;
        
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left_value = self.evaluate_where_expression(left, row, schema)?;
                let right_value = self.evaluate_where_expression(right, row, schema)?;
                
                match op {
                    BinaryOperator::Equal => Ok(left_value == right_value),
                    BinaryOperator::NotEqual => Ok(left_value != right_value),
                    BinaryOperator::LessThan => self.compare_values(&left_value, &right_value, |cmp| cmp < 0),
                    BinaryOperator::LessEqual => self.compare_values(&left_value, &right_value, |cmp| cmp <= 0),
                    BinaryOperator::GreaterThan => self.compare_values(&left_value, &right_value, |cmp| cmp > 0),
                    BinaryOperator::GreaterEqual => self.compare_values(&left_value, &right_value, |cmp| cmp >= 0),
                    _ => Err(ExecutionError::NotImplemented {
                        feature: format!("WHERE operator: {:?}", op)
                    })
                }
            }
            Expression::Column(col_name) => {
                // Column reference in WHERE should be evaluated as boolean
                let value = self.evaluate_where_expression(expr, row, schema)?;
                match value {
                    Value::Boolean(b) => Ok(b),
                    Value::Null => Ok(false),
                    _ => Ok(true), // Non-null, non-boolean values are truthy
                }
            }
            Expression::Literal(Value::Boolean(b)) => Ok(*b),
            _ => Err(ExecutionError::NotImplemented {
                feature: format!("WHERE expression: {:?}", expr)
            })
        }
    }
    
    /// Evaluate expression in WHERE context (returns Value)
    fn evaluate_where_expression(
        &self, 
        expr: &crate::sql::parser::Expression, 
        row: &Tuple, 
        schema: &Schema
    ) -> Result<Value, ExecutionError> {
        use crate::sql::parser::Expression;
        
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Column(col_name) => {
                // Find column index
                let col_index = schema.columns.iter()
                    .position(|col| col.name == *col_name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound {
                        table: "current".to_string(), // We don't have table name in this context
                        column: col_name.clone(),
                    })?;
                
                Ok(row.values[col_index].clone())
            }
            _ => Err(ExecutionError::NotImplemented {
                feature: format!("WHERE expression evaluation: {:?}", expr)
            })
        }
    }
    
    /// Compare two values for ordering (returns ordering comparison result)
    fn compare_values<F>(&self, left: &Value, right: &Value, pred: F) -> Result<bool, ExecutionError>
    where 
        F: Fn(i32) -> bool
    {
        use std::cmp::Ordering;
        
        let cmp_result = match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Varchar(a), Value::Varchar(b)) => a.cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            // Type coercion for numbers
            (Value::Integer(a), Value::BigInt(b)) => (*a as i64).cmp(b),
            (Value::BigInt(a), Value::Integer(b)) => a.cmp(&(*b as i64)),
            (Value::Float(a), Value::Double(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Double(a), Value::Float(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
            (Value::Integer(a), Value::Float(b)) => (*a as f32).partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f32)).unwrap_or(Ordering::Equal),
            (Value::Null, _) | (_, Value::Null) => return Ok(false), // NULL comparisons are always false
            _ => return Err(ExecutionError::TypeMismatch {
                expected: format!("{:?}", left),
                actual: format!("{:?}", right),
            })
        };
        
        let cmp_int = match cmp_result {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        
        Ok(pred(cmp_int))
    }
    
    /// Simplified WHERE evaluation to avoid borrowing conflicts
    fn simple_where_eval(
        &self,
        expr: &crate::sql::parser::Expression,
        row: &Tuple,
        schema: &Schema,
    ) -> Result<bool, ExecutionError> {
        // This is a simplified version of evaluate_where_condition
        // to avoid borrowing conflicts in UPDATE/DELETE operations
        use crate::sql::parser::Expression;
        use crate::sql::parser::BinaryOperator;
        
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left_value = self.simple_where_expr_eval(left, row, schema)?;
                let right_value = self.simple_where_expr_eval(right, row, schema)?;
                
                match op {
                    BinaryOperator::Equal => Ok(left_value == right_value),
                    BinaryOperator::NotEqual => Ok(left_value != right_value),
                    BinaryOperator::LessThan => self.compare_values(&left_value, &right_value, |cmp| cmp < 0),
                    BinaryOperator::LessEqual => self.compare_values(&left_value, &right_value, |cmp| cmp <= 0),
                    BinaryOperator::GreaterThan => self.compare_values(&left_value, &right_value, |cmp| cmp > 0),
                    BinaryOperator::GreaterEqual => self.compare_values(&left_value, &right_value, |cmp| cmp >= 0),
                    _ => Ok(false), // Unsupported operations default to false
                }
            }
            Expression::Literal(Value::Boolean(b)) => Ok(*b),
            _ => Ok(false), // Default to false for unsupported expressions
        }
    }
    
    /// Simplified expression evaluation for WHERE conditions
    fn simple_where_expr_eval(
        &self,
        expr: &crate::sql::parser::Expression,
        row: &Tuple,
        schema: &Schema,
    ) -> Result<Value, ExecutionError> {
        use crate::sql::parser::Expression;
        
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Column(col_name) => {
                let col_index = schema.columns.iter()
                    .position(|col| col.name == *col_name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound {
                        table: "current".to_string(),
                        column: col_name.clone(),
                    })?;
                
                Ok(row.values[col_index].clone())
            }
            _ => Err(ExecutionError::NotImplemented {
                feature: "Complex WHERE expressions in UPDATE/DELETE".to_string()
            })
        }
    }
    
    /// Project specific columns from rows (SELECT column filtering)
    fn project_columns(
        &self,
        rows: &[Tuple],
        select_exprs: &[crate::sql::parser::SelectExpr],
        schema: &Schema,
        table_name: &str,
    ) -> Result<(Vec<Tuple>, Schema), ExecutionError> {
        use crate::sql::parser::Expression;
        
        // Build new schema with selected columns
        let mut new_columns = Vec::new();
        let mut column_indices = Vec::new();
        
        for select_expr in select_exprs {
            match &select_expr.expr {
                Expression::Column(col_name) => {
                    // Find column index in original schema
                    let col_index = schema.columns.iter()
                        .position(|col| col.name == *col_name)
                        .ok_or_else(|| ExecutionError::ColumnNotFound {
                            table: table_name.to_string(),
                            column: col_name.clone(),
                        })?;
                    
                    column_indices.push(col_index);
                    
                    // Use alias if provided, otherwise use original column name
                    let column_name = select_expr.alias.as_ref()
                        .unwrap_or(col_name)
                        .clone();
                    
                    let mut new_col = schema.columns[col_index].clone();
                    new_col.name = column_name;
                    new_columns.push(new_col);
                }
                Expression::Literal(_) => {
                    // Literal values in SELECT (e.g., SELECT 1, 'hello')
                    return Err(ExecutionError::NotImplemented {
                        feature: "Literal expressions in SELECT".to_string()
                    });
                }
                _ => {
                    return Err(ExecutionError::NotImplemented {
                        feature: format!("Complex expressions in SELECT: {:?}", select_expr.expr)
                    });
                }
            }
        }
        
        // Create new schema
        let new_schema = Schema {
            columns: new_columns,
        };
        
        // Project rows to selected columns
        let projected_rows: Vec<Tuple> = rows.iter()
            .map(|row| {
                let projected_values: Vec<Value> = column_indices.iter()
                    .map(|&idx| row.values[idx].clone())
                    .collect();
                
                Tuple {
                    values: projected_values,
                }
            })
            .collect();
        
        Ok((projected_rows, new_schema))
    }
    
    /// Execute SELECT statement (simplified)
    fn execute_select_simple(
        &self,
        select_list: crate::sql::parser::SelectList,
        from_clause: Option<crate::sql::parser::FromClause>,
        where_clause: Option<crate::sql::parser::Expression>,
    ) -> Result<QueryResult, ExecutionError> {
        // Extract table name from FROM clause
        let table_name = match from_clause {
            Some(crate::sql::parser::FromClause::Table(name)) => name,
            Some(_) => {
                return Err(ExecutionError::NotImplemented {
                    feature: "Complex FROM clauses".to_string()
                });
            }
            std::option::Option::None => {
                return Err(ExecutionError::ParseError("Missing FROM clause".to_string()));
            }
        };
        
        // Get table data
        let table_id = self.table_catalog.get(&table_name)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let table_id = *table_id;
        let schema = self.table_schemas.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let table_data = self.table_data.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        // Apply WHERE clause filtering
        let filtered_rows: Vec<Tuple> = match where_clause {
            Some(expr) => {
                table_data.iter()
                    .filter(|row| {
                        match self.evaluate_where_condition(&expr, row, schema) {
                            Ok(true) => true,
                            _ => false, // If evaluation fails or returns false, exclude row
                        }
                    })
                    .cloned()
                    .collect()
            }
            std::option::Option::None => table_data.clone(),
        };
        
        // Apply column selection
        let (result_rows, result_schema) = match select_list {
            crate::sql::parser::SelectList::Wildcard => {
                // SELECT * - return all columns
                (filtered_rows.clone(), schema.clone())
            }
            crate::sql::parser::SelectList::Expressions(select_exprs) => {
                // SELECT specific columns
                self.project_columns(&filtered_rows, &select_exprs, schema, &table_name)?
            }
        };
        
        Ok(QueryResult {
            rows: result_rows.clone(),
            schema: Some(result_schema),
            affected_rows: 0,
            message: format!("Retrieved {} row(s) from table '{}' (total: {})", 
                result_rows.len(), table_name, table_data.len()),
        })
    }
    
    /// Execute UPDATE statement (simplified)
    fn execute_update_simple(
        &mut self,
        table_name: String,
        assignments: Vec<crate::sql::parser::Assignment>,
        where_clause: Option<crate::sql::parser::Expression>,
    ) -> Result<QueryResult, ExecutionError> {
        // Get table metadata first
        let table_id = self.table_catalog.get(&table_name)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let table_id = *table_id;
        let schema = self.table_schemas.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?
            .clone();
        
        // Get immutable reference to evaluate WHERE conditions
        let table_data_snapshot = self.table_data.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?
            .clone();
        
        // Evaluate which rows should be updated
        let mut indices_to_update = Vec::new();
        match &where_clause {
            Some(expr) => {
                for (i, row) in table_data_snapshot.iter().enumerate() {
                    if let Ok(true) = self.evaluate_where_condition(expr, row, &schema) {
                        indices_to_update.push(i);
                    }
                }
            }
            std::option::Option::None => {
                // No WHERE clause - update all rows
                for i in 0..table_data_snapshot.len() {
                    indices_to_update.push(i);
                }
            }
        }
        
        // Now get mutable reference and apply updates
        let table_data = self.table_data.get_mut(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let mut updated_count = 0;
        
        for row_index in indices_to_update {
            if row_index < table_data.len() {
                let row = &mut table_data[row_index];
                // Apply assignments
                for assignment in &assignments {
                    // Find column index
                    if let Some(col_index) = schema.columns.iter()
                        .position(|col| col.name == assignment.column) {
                        
                        // Evaluate new value using simplified literal matching
                        let new_value = match &assignment.value {
                            crate::sql::parser::Expression::Literal(val) => val.clone(),
                            _ => {
                                // For now, only support literal values in UPDATE
                                return Err(ExecutionError::NotImplemented { 
                                    feature: "UPDATE with complex expressions".to_string() 
                                });
                            }
                        };
                        
                        // Update the value
                        row.values[col_index] = new_value;
                    } else {
                        return Err(ExecutionError::ColumnNotFound {
                            table: table_name.clone(),
                            column: assignment.column.clone(),
                        });
                    }
                }
                updated_count += 1;
            }
        }
        
        // Save table data after update
        if updated_count > 0 {
            if let Err(e) = self.save_table(table_id, &table_name) {
                println!("Warning: Failed to save table data: {}", e);
            }
        }
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: updated_count,
            message: format!("Updated {} row(s) in table '{}'", updated_count, table_name),
        })
    }
    
    /// Execute DELETE statement (simplified)
    fn execute_delete_simple(
        &mut self,
        table_name: String,
        where_clause: Option<crate::sql::parser::Expression>,
    ) -> Result<QueryResult, ExecutionError> {
        // Get table metadata first
        let table_id = self.table_catalog.get(&table_name)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let table_id = *table_id;
        let schema = self.table_schemas.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?
            .clone();
        
        // Get immutable reference to evaluate WHERE conditions
        let table_data_snapshot = self.table_data.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?
            .clone();
        
        let original_count = table_data_snapshot.len();
        
        // Evaluate which rows should be deleted
        let mut indices_to_delete = Vec::new();
        match where_clause {
            Some(expr) => {
                // Evaluate WHERE condition for each row
                for (i, row) in table_data_snapshot.iter().enumerate() {
                    if let Ok(true) = self.evaluate_where_condition(&expr, row, &schema) {
                        indices_to_delete.push(i);
                    }
                }
            }
            std::option::Option::None => {
                // No WHERE clause - delete all rows
                for i in 0..table_data_snapshot.len() {
                    indices_to_delete.push(i);
                }
            }
        }
        
        // Now get mutable reference and delete rows (from back to front to maintain indices)
        let table_data = self.table_data.get_mut(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        // Sort indices in descending order to delete from back to front
        indices_to_delete.sort_by(|a, b| b.cmp(a));
        
        for &index in &indices_to_delete {
            if index < table_data.len() {
                table_data.remove(index);
            }
        }
        
        let deleted_count = indices_to_delete.len();
        
        // Save table data after deletion
        if deleted_count > 0 {
            if let Err(e) = self.save_table(table_id, &table_name) {
                println!("Warning: Failed to save table data: {}", e);
            }
        }
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: deleted_count,
            message: format!("Deleted {} row(s) from table '{}' (total was: {})", 
                deleted_count, table_name, original_count),
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

    // ===============================
    // 数据持久化相关方法
    // ===============================

    /// 保存表数据到文件
    fn save_table(&self, table_id: u32, table_name: &str) -> Result<(), ExecutionError> {
        // 获取表的schema和数据
        let schema = self.table_schemas.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.to_string() })?;
        
        let rows = self.table_data.get(&table_id).cloned().unwrap_or_default();

        let table_data = TableData {
            schema: schema.clone(),
            rows,
        };

        // 序列化为JSON
        let json = serde_json::to_string_pretty(&table_data)
            .map_err(|e| ExecutionError::StorageError(format!("Serialization error: {}", e)))?;

        // 写入文件
        let file_path = self.data_dir.join(format!("table_{}.json", table_id));
        let mut file = File::create(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("File creation error: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| ExecutionError::StorageError(format!("Write error: {}", e)))?;

        log::debug!("Saved table '{}' (id: {}) to disk", table_name, table_id);
        Ok(())
    }

    /// 从文件加载表数据
    fn load_table(&mut self, table_id: u32) -> Result<Option<String>, ExecutionError> {
        let file_path = self.data_dir.join(format!("table_{}.json", table_id));
        
        if !file_path.exists() {
            return Ok(None); // 文件不存在，跳过
        }

        // 读取文件内容
        let mut file = File::open(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("File open error: {}", e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| ExecutionError::StorageError(format!("Read error: {}", e)))?;

        // 反序列化
        let table_data: TableData = serde_json::from_str(&contents)
            .map_err(|e| ExecutionError::StorageError(format!("Deserialization error: {}", e)))?;

        // 恢复到内存中
        let rows_count = table_data.rows.len();
        self.table_schemas.insert(table_id, table_data.schema);
        self.table_data.insert(table_id, table_data.rows);

        log::debug!("Loaded table with id {} from disk ({} rows)", table_id, rows_count);
        
        // 返回None，因为我们没有从文件中获取表名，需要从元数据中获取
        Ok(None)
    }

    /// 保存数据库元数据
    fn save_metadata(&self) -> Result<(), ExecutionError> {
        let metadata = DatabaseMetadata {
            next_table_id: self.next_table_id,
            table_catalog: self.table_catalog.clone(),
        };

        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ExecutionError::StorageError(format!("Metadata serialization error: {}", e)))?;

        let file_path = self.data_dir.join("metadata.json");
        let mut file = File::create(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("Metadata file creation error: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| ExecutionError::StorageError(format!("Metadata write error: {}", e)))?;

        log::debug!("Saved database metadata (next_id: {}, tables: {})", 
                   self.next_table_id, self.table_catalog.len());
        Ok(())
    }

    /// 加载数据库元数据
    fn load_metadata(&mut self) -> Result<(), ExecutionError> {
        let file_path = self.data_dir.join("metadata.json");
        
        if !file_path.exists() {
            log::debug!("No metadata file found, starting with fresh database");
            return Ok(()); // 没有元数据文件，是新数据库
        }

        let mut file = File::open(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("Metadata file open error: {}", e)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| ExecutionError::StorageError(format!("Metadata read error: {}", e)))?;

        let metadata: DatabaseMetadata = serde_json::from_str(&contents)
            .map_err(|e| ExecutionError::StorageError(format!("Metadata deserialization error: {}", e)))?;

        self.next_table_id = metadata.next_table_id;
        self.table_catalog = metadata.table_catalog;

        log::debug!("Loaded database metadata (next_id: {}, tables: {})", 
                   self.next_table_id, self.table_catalog.len());
        Ok(())
    }

    /// 加载所有现有表
    fn load_existing_tables(&mut self) -> Result<(), ExecutionError> {
        // 先加载元数据
        self.load_metadata()?;

        // 加载所有表的数据
        for (table_name, &table_id) in &self.table_catalog.clone() {
            if let Err(e) = self.load_table(table_id) {
                log::warn!("Failed to load table '{}' (id: {}): {}", table_name, table_id, e);
                // 继续加载其他表，不要因为一个表加载失败就停止
            }
        }

        log::info!("Database loaded: {} tables", self.table_catalog.len());
        Ok(())
    }
}
