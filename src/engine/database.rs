//! Database engine implementation
//!
//! Main database interface and query execution coordination.

use crate::sql::{parse_sql, Statement};
use crate::sql::parser::OrderByExpr;
use crate::sql::diagnostics::{DiagnosticEngine, DiagnosticContext};
use crate::sql::optimizer::QueryOptimizer;
use crate::storage::{BufferPool, FileManager};
use crate::types::{Schema, Tuple, Value, DataType, ColumnDefinition};
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
    /// Error diagnostics engine
    diagnostic_engine: DiagnosticEngine,
    /// Query optimizer
    optimizer: QueryOptimizer,
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
    
    #[error("Primary key constraint violation: duplicate key value {key}")]
    PrimaryKeyViolation { key: String },
    
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
    
    #[error("Evaluation error: {message}")]
    EvaluationError { message: String },
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
            diagnostic_engine: DiagnosticEngine::new(),
            optimizer: QueryOptimizer::new(),
        };
        
        // Load existing data if available
        if let Err(e) = database.load_existing_tables() {
            println!("Warning: Failed to load existing tables: {}", e);
        }
        
        Ok(database)
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult, ExecutionError> {
        // Step 1: Parse SQL with enhanced error diagnostics
        let statement = parse_sql(sql)
            .map_err(|e| {
                let context = DiagnosticContext::new(
                    self.table_catalog.keys().cloned().collect(),
                    self.get_all_column_names(),
                );
                let suggestions = self.diagnostic_engine.diagnose(&e.to_string(), Some(&context));
                let enhanced_error = self.diagnostic_engine.format_enhanced_error(
                    &e.to_string(),
                    &suggestions
                );
                ExecutionError::ParseError(enhanced_error)
            })?;
        
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
            Statement::Select { select_list, from_clause, where_clause, group_by, having, order_by, limit, offset } => {
                self.execute_select_complete(select_list, from_clause, where_clause, group_by, having, order_by, limit, offset)
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
        
        // Convert column definitions to schema and extract primary key info
        let mut schema_columns = Vec::new();
        let mut primary_key_columns = Vec::new();
        
        for (i, col_def) in columns.iter().enumerate() {
            let column = crate::types::ColumnDefinition {
                name: col_def.name.clone(),
                data_type: col_def.data_type.clone(),
                nullable: col_def.nullable,
                default: None, // Simplified for now
            };
            schema_columns.push(column);
            
            // Check for column-level primary key
            if col_def.primary_key {
                primary_key_columns.push(i);
            }
        }
        
        let primary_key = if primary_key_columns.is_empty() {
            None
        } else {
            Some(primary_key_columns)
        };
        
        let schema = Schema {
            columns: schema_columns,
            primary_key,
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
            
            // Create tuple
            let tuple = Tuple { values: row_values };
            
            // Check primary key constraint before inserting
            if let Some(ref primary_key_columns) = schema.primary_key {
                self.check_primary_key_constraint(&tuple, primary_key_columns, table_id)?;
            }
            
            // Add to table data
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
                    
                    // Logical operators: evaluate as boolean conditions
                    BinaryOperator::And => {
                        let left_bool = self.evaluate_where_condition(left, row, schema)?;
                        let right_bool = self.evaluate_where_condition(right, row, schema)?;
                        Ok(left_bool && right_bool)
                    }
                    BinaryOperator::Or => {
                        let left_bool = self.evaluate_where_condition(left, row, schema)?;
                        let right_bool = self.evaluate_where_condition(right, row, schema)?;
                        Ok(left_bool || right_bool)
                    }
                    
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
            (Value::Integer(a), Value::Double(b)) => (*a as f64).partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Double(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(Ordering::Equal),
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
                Expression::FunctionCall { name, args } => {
                    // 聚合函数调用 (e.g., COUNT(*), AVG(age))
                    // 注意：在 project_columns 中，我们不直接计算聚合函数
                    // 这里只是为了构建结果 schema，实际计算在 GROUP BY 处理中完成
                    
                    let column_name = select_expr.alias.as_ref()
                        .unwrap_or(&format!("{}(...)", name))
                        .clone();
                    
                    // 根据函数类型确定返回值类型
                    let data_type = match name.to_uppercase().as_str() {
                        "COUNT" => crate::types::DataType::Integer,
                        "SUM" | "AVG" | "MAX" | "MIN" => crate::types::DataType::Double, // 默认为 Double
                        _ => crate::types::DataType::Varchar(50), // 未知函数默认为字符串
                    };
                    
                    new_columns.push(crate::types::ColumnDefinition {
                        name: column_name,
                        data_type,
                        nullable: true,
                        default: None,
                    });
                    
                    // 对于聚合函数，我们需要特殊处理，暂时使用 -1 作为标记
                    column_indices.push(usize::MAX);
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
            primary_key: None, // Projected query results don't have primary key
        };
        
        // Project rows to selected columns
        let projected_rows: Vec<Tuple> = rows.iter()
            .map(|row| {
                let projected_values: Vec<Value> = column_indices.iter()
                    .map(|&idx| {
                        if idx == usize::MAX {
                            // 对于聚合函数，暂时返回 NULL（将在 GROUP BY 中处理）
                            crate::types::Value::Null
                        } else {
                            row.values[idx].clone()
                        }
                    })
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

    /// Execute SELECT statement with full feature support (ORDER BY, GROUP BY, LIMIT, etc.)
    fn execute_select_complete(
        &self,
        select_list: crate::sql::parser::SelectList,
        from_clause: Option<crate::sql::parser::FromClause>,
        where_clause: Option<crate::sql::parser::Expression>,
        group_by: Option<Vec<crate::sql::parser::Expression>>,
        having: Option<crate::sql::parser::Expression>,
        order_by: Option<Vec<crate::sql::parser::OrderByExpr>>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<QueryResult, ExecutionError> {
        use crate::engine::executor::{Executor, HashJoinExecutor, SortExecutor, LimitExecutor, GroupByExecutor, AggregateFunction};
        use crate::sql::planner::{JoinType, SortKey};
        use crate::sql::parser::{FromClause, OrderByExpr};
        
        // 检测并报告高级功能
        let mut detected_features = Vec::new();
        if group_by.is_some() { detected_features.push("GROUP BY"); }
        if having.is_some() { detected_features.push("HAVING"); }
        if order_by.is_some() { detected_features.push("ORDER BY"); }
        if limit.is_some() { detected_features.push("LIMIT"); }
        if offset.is_some() { detected_features.push("OFFSET"); }
        
        if !detected_features.is_empty() {
            println!("🚀 执行高级SQL功能: {}", detected_features.join(", "));
        }
        
        // 检测 SELECT 列表是否包含聚合函数
        let has_aggregate_functions = self.select_list_contains_aggregates(&select_list);
        if has_aggregate_functions && !detected_features.contains(&"GROUP BY") {
            detected_features.push("IMPLICIT GROUP BY (aggregate functions)");
        }

        // 开始构建执行计划
        // 1. 如果有 GROUP BY 或者 SELECT 包含聚合函数，需要特殊处理执行流程
        let mut base_result = if group_by.is_some() || has_aggregate_functions {
            // GROUP BY 查询：先获取原始数据（不进行列投影），然后应用分组聚合
            let table_name = match &from_clause {
                Some(crate::sql::parser::FromClause::Table(name)) => name.clone(),
                _ => return Err(ExecutionError::NotImplemented { 
                    feature: "Complex FROM clauses with GROUP BY".to_string() 
                }),
            };
            
            // 获取原始表数据和 schema（不进行列投影）
            let table_id = self.table_catalog.get(&table_name)
                .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
            let table_id = *table_id;
            let original_schema = self.table_schemas.get(&table_id)
                .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?.clone();
            let table_data = self.table_data.get(&table_id)
                .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
            
            // 应用 WHERE 过滤但保持原始 schema
            let filtered_rows: Vec<Tuple> = match where_clause {
                Some(expr) => {
                    table_data.iter()
                        .filter(|row| {
                            match self.evaluate_where_condition(&expr, row, &original_schema) {
                                Ok(true) => true,
                                _ => false,
                            }
                        })
                        .cloned()
                        .collect()
                }
                None => table_data.clone(),
            };
            
            let filtered_result = QueryResult {
                rows: filtered_rows,
                schema: Some(original_schema),
                affected_rows: 0,
                message: "Filtered data for GROUP BY".to_string(),
            };
            
            // 应用 GROUP BY 分组聚合
            let group_expressions = group_by.unwrap_or_else(|| Vec::new()); // 如果没有 GROUP BY，使用空的分组表达式
            self.apply_group_by_with_select(filtered_result, group_expressions, select_list, having)?
        } else {
            // 普通查询：执行基础查询（表扫描 + WHERE 过滤 + 列投影）
            self.execute_select_simple(select_list.clone(), from_clause.clone(), where_clause)?
        };
        
        // 2. 如果有 GROUP BY，上面已经处理了，这里跳过
        // GROUP BY 已经在上面处理完成
        
        // 3. 如果有 ORDER BY，应用排序
        if let Some(order_exprs) = order_by {
            base_result = self.apply_order_by(base_result, order_exprs)?;
        }
        
        // 4. 如果有 LIMIT/OFFSET，应用分页
        if limit.is_some() || offset.is_some() {
            base_result = self.apply_limit_offset(base_result, limit.unwrap_or(u64::MAX), offset.unwrap_or(0))?;
        }
        
        Ok(base_result)
    }
    
    /// 应用 GROUP BY 分组聚合 (支持聚合函数)
    fn apply_group_by_with_select(
        &self,
        input_result: QueryResult,
        group_exprs: Vec<crate::sql::parser::Expression>,
        select_list: crate::sql::parser::SelectList,
        _having: Option<crate::sql::parser::Expression>,
    ) -> Result<QueryResult, ExecutionError> {
        use std::collections::HashMap;
        use crate::sql::parser::{Expression, SelectList};
        
        // 创建分组哈希表
        let mut groups: HashMap<Vec<Value>, Vec<Tuple>> = HashMap::new();
        
        // 按分组表达式对元组进行分组
        for tuple in input_result.rows {
            let mut group_key = Vec::new();
            
            // 评估分组表达式
            for expr in &group_exprs {
                let schema = input_result.schema.as_ref().unwrap();
                let group_value = self.evaluate_expression_for_tuple(expr, &tuple, schema)?;
                group_key.push(group_value);
            }
            
            groups.entry(group_key).or_insert_with(Vec::new).push(tuple);
        }
        
        // 解析 SELECT 子句中的表达式
        let select_expressions = match select_list {
            SelectList::Expressions(exprs) => exprs,
            SelectList::Wildcard => {
                return Err(ExecutionError::NotImplemented {
                    feature: "GROUP BY with SELECT * not supported".to_string()
                });
            }
        };
        
        // 构建结果 schema
        let mut result_columns = Vec::new();
        for select_expr in &select_expressions {
            let column_name = if let Some(alias) = &select_expr.alias {
                alias.clone()
            } else {
                match &select_expr.expr {
                    Expression::Column(col_name) => col_name.clone(),
                    Expression::FunctionCall { name, .. } => {
                        format!("{}()", name) // COUNT(), AVG(), etc.
                    }
                    _ => "expr".to_string(),
                }
            };
            
            let data_type = match &select_expr.expr {
                Expression::Column(_) => crate::types::DataType::Varchar(50), // 分组列
                Expression::FunctionCall { name, .. } => {
                    match name.to_uppercase().as_str() {
                        "COUNT" => crate::types::DataType::Integer,
                        "AVG" | "SUM" | "MAX" | "MIN" => crate::types::DataType::Double,
                        _ => crate::types::DataType::Double,
                    }
                }
                _ => crate::types::DataType::Varchar(50),
            };
            
            result_columns.push(crate::types::ColumnDefinition {
                name: column_name,
                data_type,
                nullable: true,
                default: None,
            });
        }
        
        // 生成聚合结果
        let mut result_rows = Vec::new();
        
        for (group_key, group_tuples) in groups {
            let mut result_values = Vec::new();
            
            for select_expr in &select_expressions {
                match &select_expr.expr {
                    Expression::Column(col_name) => {
                        // 分组列：从 group_key 中取值
                        // 找到这个列在 GROUP BY 表达式中的位置
                        let mut found = false;
                        for (i, group_expr) in group_exprs.iter().enumerate() {
                            if let Expression::Column(group_col_name) = group_expr {
                                if group_col_name == col_name {
                                    result_values.push(group_key[i].clone());
                                    found = true;
                                    break;
                                }
                            }
                        }
                        
                        if !found {
                            result_values.push(Value::Null);
                        }
                    }
                    Expression::FunctionCall { name, args } => {
                        // 聚合函数：使用原始输入的 schema
                        let original_schema = input_result.schema.as_ref().unwrap();
                        let agg_value = self.compute_aggregate_function(name, args, &group_tuples, original_schema)?;
                        result_values.push(agg_value);
                    }
                    _ => {
                        result_values.push(Value::Null);
                    }
                }
            }
            
            result_rows.push(Tuple { values: result_values });
        }
        
        let row_count = result_rows.len();
        Ok(QueryResult {
            rows: result_rows,
            schema: Some(crate::types::Schema { columns: result_columns, primary_key: None }),
            affected_rows: row_count,
            message: format!("📊 GROUP BY 查询完成，返回 {} 行聚合结果", row_count),
        })
    }
    
    /// 计算聚合函数值
    fn compute_aggregate_function(
        &self,
        func_name: &str,
        args: &[crate::sql::parser::Expression],
        group_tuples: &[crate::types::Tuple],
        schema: &crate::types::Schema,
    ) -> Result<crate::types::Value, ExecutionError> {
        use crate::types::Value;
        
        match func_name.to_uppercase().as_str() {
            "COUNT" => {
                // COUNT(*) 或 COUNT(column)
                if args.is_empty() || (args.len() == 1 && matches!(args[0], crate::sql::parser::Expression::Literal(Value::Varchar(ref s)) if s == "*")) {
                    // COUNT(*) - 计算行数
                    Ok(Value::Integer(group_tuples.len() as i32))
                } else {
                    // COUNT(column) - 计算非NULL值的数量
                    let mut count = 0;
                    for tuple in group_tuples {
                        if let Ok(val) = self.evaluate_expression_for_tuple(&args[0], tuple, schema) {
                            if !matches!(val, Value::Null) {
                                count += 1;
                            }
                        }
                    }
                    Ok(Value::Integer(count))
                }
            }
            "SUM" => {
                if args.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: "SUM function requires an argument".to_string()
                    });
                }
                
                let mut sum = 0.0;
                for tuple in group_tuples {
                    if let Ok(val) = self.evaluate_expression_for_tuple(&args[0], tuple, schema) {
                        sum += self.value_to_f64(&val);
                    }
                }
                Ok(Value::Double(sum))
            }
            "AVG" => {
                if args.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: "AVG function requires an argument".to_string()
                    });
                }
                
                let mut sum = 0.0;
                let mut count = 0;
                for tuple in group_tuples {
                    if let Ok(val) = self.evaluate_expression_for_tuple(&args[0], tuple, schema) {
                        if !matches!(val, Value::Null) {
                            sum += self.value_to_f64(&val);
                            count += 1;
                        }
                    }
                }
                
                if count > 0 {
                    Ok(Value::Double(sum / count as f64))
                } else {
                    Ok(Value::Null)
                }
            }
            "MAX" => {
                if args.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: "MAX function requires an argument".to_string()
                    });
                }
                
                let mut max_val: Option<f64> = None;
                for tuple in group_tuples {
                    if let Ok(val) = self.evaluate_expression_for_tuple(&args[0], tuple, schema) {
                        if !matches!(val, Value::Null) {
                            let num_val = self.value_to_f64(&val);
                            max_val = Some(max_val.map_or(num_val, |current| current.max(num_val)));
                        }
                    }
                }
                
                Ok(max_val.map(Value::Double).unwrap_or(Value::Null))
            }
            "MIN" => {
                if args.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: "MIN function requires an argument".to_string()
                    });
                }
                
                let mut min_val: Option<f64> = None;
                for tuple in group_tuples {
                    if let Ok(val) = self.evaluate_expression_for_tuple(&args[0], tuple, schema) {
                        if !matches!(val, Value::Null) {
                            let num_val = self.value_to_f64(&val);
                            min_val = Some(min_val.map_or(num_val, |current| current.min(num_val)));
                        }
                    }
                }
                
                Ok(min_val.map(Value::Double).unwrap_or(Value::Null))
            }
            _ => {
                Err(ExecutionError::NotImplemented {
                    feature: format!("Aggregate function: {}", func_name)
                })
            }
        }
    }

    /// Check if SELECT list contains aggregate functions
    fn select_list_contains_aggregates(&self, select_list: &crate::sql::parser::SelectList) -> bool {
        use crate::sql::parser::{SelectList, Expression};
        
        match select_list {
            SelectList::Wildcard => false,
            SelectList::Expressions(expressions) => {
                expressions.iter().any(|select_expr| {
                    self.expression_contains_aggregates(&select_expr.expr)
                })
            }
        }
    }

    /// Check if an expression contains aggregate functions (recursively)
    fn expression_contains_aggregates(&self, expr: &crate::sql::parser::Expression) -> bool {
        use crate::sql::parser::Expression;
        
        match expr {
            Expression::FunctionCall { name, .. } => {
                // Check if this is an aggregate function
                matches!(name.to_uppercase().as_str(), "COUNT" | "SUM" | "AVG" | "MIN" | "MAX")
            }
            // For other expression types, we can add recursive checks if needed
            _ => false
        }
    }
    
    /// 应用 GROUP BY 分组聚合
    fn apply_group_by(
        &self,
        input_result: QueryResult,
        group_exprs: Vec<crate::sql::parser::Expression>,
        _having: Option<crate::sql::parser::Expression>,
    ) -> Result<QueryResult, ExecutionError> {
        use std::collections::HashMap;
        
        // 创建分组哈希表
        let mut groups: HashMap<Vec<Value>, Vec<Tuple>> = HashMap::new();
        
        // 按分组表达式对元组进行分组
        for tuple in input_result.rows {
            let mut group_key = Vec::new();
            
            // 评估分组表达式
            for expr in &group_exprs {
                let schema = input_result.schema.as_ref().unwrap();
                let group_value = self.evaluate_expression_for_tuple(expr, &tuple, schema)?;
                group_key.push(group_value);
            }
            
            groups.entry(group_key).or_insert_with(Vec::new).push(tuple);
        }
        
        // 生成聚合结果
        let mut result_rows = Vec::new();
        
        for (group_key, group_tuples) in groups {
            // 计算完整的聚合函数
            let count = group_tuples.len() as i32;
            let numeric_values: Vec<f64> = group_tuples.iter()
                .filter_map(|t| t.values.iter().find(|v| matches!(v, Value::Integer(_) | Value::Float(_) | Value::Double(_))))
                .map(|v| self.value_to_f64(v))
                .collect();
            
            // 构建结果元组（分组键 + 聚合值）
            let mut result_values = group_key;
            
            // COUNT(*) - 总行数
            result_values.push(Value::Integer(count));
            
            // 如果有数值列，计算其他聚合函数
            if !numeric_values.is_empty() {
                // SUM - 求和
                let sum = numeric_values.iter().sum::<f64>();
                result_values.push(Value::Double(sum));
                
                // AVG - 平均值
                let avg = sum / (numeric_values.len() as f64);
                result_values.push(Value::Double(avg));
                
                // MAX - 最大值
                let max = numeric_values.iter()
                    .fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));
                result_values.push(Value::Double(max));
                
                // MIN - 最小值
                let min = numeric_values.iter()
                    .fold(f64::INFINITY, |acc, &x| acc.min(x));
                result_values.push(Value::Double(min));
            } else {
                // 如果没有数值列，添加NULL值
                result_values.push(Value::Null); // SUM
                result_values.push(Value::Null); // AVG
                result_values.push(Value::Null); // MAX
                result_values.push(Value::Null); // MIN
            }
            
            result_rows.push(Tuple { values: result_values });
        }
        
        // 构建结果schema
        let mut result_columns = Vec::new();
        
        // 添加分组列
        for (i, _) in group_exprs.iter().enumerate() {
            result_columns.push(ColumnDefinition {
                name: format!("group_col_{}", i),
                data_type: DataType::Varchar(50),
                nullable: true,
                default: None,
            });
        }
        
        // 添加聚合列
        result_columns.push(ColumnDefinition {
            name: "COUNT(*)".to_string(),
            data_type: DataType::Integer,
            nullable: false,
            default: None,
        });
        
        result_columns.push(ColumnDefinition {
            name: "SUM".to_string(),
            data_type: DataType::Double,
            nullable: true,
            default: None,
        });
        
        result_columns.push(ColumnDefinition {
            name: "AVG".to_string(),
            data_type: DataType::Double,
            nullable: true,
            default: None,
        });
        
        let row_count = result_rows.len();
        Ok(QueryResult {
            rows: result_rows,
            schema: Some(Schema { columns: result_columns, primary_key: None }),
            affected_rows: row_count,
            message: format!("📊 GROUP BY 查询完成，返回 {} 行聚合结果", row_count),
        })
    }
    
    /// 应用 ORDER BY 排序
    fn apply_order_by(
        &self,
        mut input_result: QueryResult,
        order_exprs: Vec<OrderByExpr>,
    ) -> Result<QueryResult, ExecutionError> {
        // 按照 ORDER BY 表达式进行排序
        let schema = input_result.schema.as_ref().unwrap();
        input_result.rows.sort_by(|a, b| {
            for order_expr in &order_exprs {
                let a_value = self.evaluate_expression_for_tuple(&order_expr.expr, a, schema)
                    .unwrap_or(Value::Null);
                let b_value = self.evaluate_expression_for_tuple(&order_expr.expr, b, schema)
                    .unwrap_or(Value::Null);
                
                let cmp = self.compare_values_for_sort(&a_value, &b_value);
                match cmp {
                    std::cmp::Ordering::Equal => continue,
                    other => {
                        return if order_expr.desc {
                            other.reverse()
                        } else {
                            other
                        };
                    }
                }
            }
            std::cmp::Ordering::Equal
        });
        
        Ok(input_result)
    }
    
    /// 应用 LIMIT 和 OFFSET
    fn apply_limit_offset(
        &self,
        mut input_result: QueryResult,
        limit: u64,
        offset: u64,
    ) -> Result<QueryResult, ExecutionError> {
        let start_index = offset as usize;
        let end_index = if limit == u64::MAX {
            input_result.rows.len()
        } else {
            std::cmp::min(start_index + (limit as usize), input_result.rows.len())
        };
        
        if start_index >= input_result.rows.len() {
            input_result.rows = Vec::new();
        } else {
            input_result.rows = input_result.rows[start_index..end_index].to_vec();
        }
        
        Ok(input_result)
    }
    
    /// 评估元组上下文中的表达式
    fn evaluate_expression_for_tuple(
        &self,
        expr: &crate::sql::parser::Expression,
        tuple: &Tuple,
        schema: &Schema,
    ) -> Result<Value, ExecutionError> {
        use crate::sql::parser::Expression;
        
        // 边界检查：确保tuple不为空
        if tuple.values.is_empty() {
            return Ok(Value::Null);
        }
        
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Column(col_name) => {
                // 增强错误处理：检查列名有效性
                if col_name.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: "Empty column name in expression".to_string(),
                    });
                }
                
                let col_index = schema.columns.iter()
                    .position(|col| col.name == *col_name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound {
                        table: "current".to_string(),
                        column: col_name.clone(),
                    })?;
                
                // 边界检查：确保索引有效
                if col_index >= tuple.values.len() {
                    return Err(ExecutionError::EvaluationError {
                        message: format!("Column index {} out of bounds for tuple with {} values", 
                                       col_index, tuple.values.len()),
                    });
                }
                
                Ok(tuple.values[col_index].clone())
            }
            Expression::QualifiedColumn { table, column } => {
                // 增强错误处理：检查表名和列名
                if column.is_empty() {
                    return Err(ExecutionError::EvaluationError {
                        message: format!("Empty column name in qualified expression for table {}", table),
                    });
                }
                
                // 优化的表别名解析：支持多种匹配策略
                let col_index = self.resolve_qualified_column_index(table, column, schema)?;
                
                // 边界检查：确保索引有效
                if col_index >= tuple.values.len() {
                    return Err(ExecutionError::EvaluationError {
                        message: format!("Column index {} out of bounds for tuple with {} values", 
                                       col_index, tuple.values.len()),
                    });
                }
                
                Ok(tuple.values[col_index].clone())
            }
            Expression::BinaryOp { left, op, right } => {
                // 支持算术运算表达式
                let left_val = self.evaluate_expression_for_tuple(left, tuple, schema)?;
                let right_val = self.evaluate_expression_for_tuple(right, tuple, schema)?;
                
                use crate::sql::parser::BinaryOperator;
                match op {
                    BinaryOperator::Add => {
                        match (left_val, right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a + b)),
                            (Value::Integer(a), Value::Double(b)) => Ok(Value::Double(a as f64 + b)),
                            (Value::Double(a), Value::Integer(b)) => Ok(Value::Double(a + b as f64)),
                            _ => Err(ExecutionError::EvaluationError {
                                message: "Cannot add non-numeric values".to_string(),
                            })
                        }
                    }
                    BinaryOperator::Subtract => {
                        match (left_val, right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a - b)),
                            (Value::Integer(a), Value::Double(b)) => Ok(Value::Double(a as f64 - b)),
                            (Value::Double(a), Value::Integer(b)) => Ok(Value::Double(a - b as f64)),
                            _ => Err(ExecutionError::EvaluationError {
                                message: "Cannot subtract non-numeric values".to_string(),
                            })
                        }
                    }
                    BinaryOperator::Multiply => {
                        match (left_val, right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                            (Value::Double(a), Value::Double(b)) => Ok(Value::Double(a * b)),
                            (Value::Integer(a), Value::Double(b)) => Ok(Value::Double(a as f64 * b)),
                            (Value::Double(a), Value::Integer(b)) => Ok(Value::Double(a * b as f64)),
                            _ => Err(ExecutionError::EvaluationError {
                                message: "Cannot multiply non-numeric values".to_string(),
                            })
                        }
                    }
                    BinaryOperator::Divide => {
                        match (left_val, right_val) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if b == 0 {
                                    Err(ExecutionError::EvaluationError {
                                        message: "Division by zero".to_string(),
                                    })
                                } else {
                                    Ok(Value::Double(a as f64 / b as f64))
                                }
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                if b == 0.0 {
                                    Err(ExecutionError::EvaluationError {
                                        message: "Division by zero".to_string(),
                                    })
                                } else {
                                    Ok(Value::Float(a / b))
                                }
                            }
                            (Value::Double(a), Value::Double(b)) => {
                                if b == 0.0 {
                                    Err(ExecutionError::EvaluationError {
                                        message: "Division by zero".to_string(),
                                    })
                                } else {
                                    Ok(Value::Double(a / b))
                                }
                            }
                            (Value::Integer(a), Value::Double(b)) => {
                                if b == 0.0 {
                                    Err(ExecutionError::EvaluationError {
                                        message: "Division by zero".to_string(),
                                    })
                                } else {
                                    Ok(Value::Double(a as f64 / b))
                                }
                            }
                            (Value::Double(a), Value::Integer(b)) => {
                                if b == 0 {
                                    Err(ExecutionError::EvaluationError {
                                        message: "Division by zero".to_string(),
                                    })
                                } else {
                                    Ok(Value::Double(a / b as f64))
                                }
                            }
                            _ => Err(ExecutionError::EvaluationError {
                                message: "Cannot divide non-numeric values".to_string(),
                            })
                        }
                    }
                    _ => {
                        // 对于比较运算符和其他操作符，暂时不支持
                        Err(ExecutionError::EvaluationError {
                            message: format!("Unsupported binary operator: {:?}", op),
                        })
                    }
                }
            }
            _ => {
                // 对于其他不支持的表达式类型，返回第一个值但记录警告
                println!("⚠️ 不支持的表达式类型，使用元组第一个值");
                Ok(tuple.values.get(0).cloned().unwrap_or(Value::Null))
            }
        }
    }
    
    /// 值转换为浮点数（用于聚合计算）
    fn value_to_f64(&self, value: &Value) -> f64 {
        match value {
            Value::Integer(i) => *i as f64,
            Value::Float(f) => *f as f64,
            Value::Double(d) => *d,
            _ => 0.0,
        }
    }
    
    /// 比较值用于排序
    fn compare_values_for_sort(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Varchar(a), Value::Varchar(b)) => a.cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
    
    /// 解析限定列名的索引（支持表别名）
    fn resolve_qualified_column_index(
        &self,
        table_name: &str,
        column_name: &str,
        schema: &Schema,
    ) -> Result<usize, ExecutionError> {
        // 策略1：直接匹配列名（忽略表名）
        if let Some(index) = schema.columns.iter().position(|col| col.name == column_name) {
            return Ok(index);
        }
        
        // 策略2：尝试匹配 "table.column" 格式的列名
        let qualified_name = format!("{}.{}", table_name, column_name);
        if let Some(index) = schema.columns.iter().position(|col| col.name == qualified_name) {
            return Ok(index);
        }
        
        // 策略3：查找以表名开头的列名
        let table_prefix = format!("{}.", table_name);
        if let Some(index) = schema.columns.iter().position(|col| {
            col.name.starts_with(&table_prefix) && 
            col.name[table_prefix.len()..] == *column_name
        }) {
            return Ok(index);
        }
        
        // 策略4：模糊匹配（对于JOIN后的合并schema）
        // 在JOIN的情况下，列名可能被重命名为 table1_column, table2_column 等形式
        let possible_names = vec![
            format!("{}_{}", table_name, column_name),
            format!("{}__{}", table_name, column_name), // 双下划线分隔
            format!("{}:{}", table_name, column_name),   // 冒号分隔
        ];
        
        for possible_name in possible_names {
            if let Some(index) = schema.columns.iter().position(|col| col.name == possible_name) {
                return Ok(index);
            }
        }
        
        // 如果所有策略都失败，返回错误
        Err(ExecutionError::ColumnNotFound {
            table: table_name.to_string(),
            column: column_name.to_string(),
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
        
        // Pre-compute new values for each row to avoid borrowing issues
        let mut updated_rows = Vec::new();
        for row_index in &indices_to_update {
            if *row_index < table_data_snapshot.len() {
                let row = &table_data_snapshot[*row_index];
                let mut new_row = row.clone();
                
                // Apply assignments
                for assignment in &assignments {
                    // Find column index
                    if let Some(col_index) = schema.columns.iter()
                        .position(|col| col.name == assignment.column) {
                        
                        // Evaluate new value - support both literals and expressions
                        let new_value = match &assignment.value {
                            crate::sql::parser::Expression::Literal(val) => val.clone(),
                            _ => {
                                // Support complex expressions like age = age + 1
                                match self.evaluate_expression_for_tuple(&assignment.value, row, &schema) {
                                    Ok(val) => val,
                                    Err(_) => {
                                        return Err(ExecutionError::NotImplemented { 
                                            feature: "Complex UPDATE expression evaluation failed".to_string() 
                                        });
                                    }
                                }
                            }
                        };
                        
                        // Update the value in the new row
                        new_row.values[col_index] = new_value;
                    } else {
                        return Err(ExecutionError::ColumnNotFound {
                            table: table_name.clone(),
                            column: assignment.column.clone(),
                        });
                    }
                }
                updated_rows.push((*row_index, new_row));
            }
        }
        
        // Now get mutable reference and apply the pre-computed updates
        let table_data = self.table_data.get_mut(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { table: table_name.clone() })?;
        
        let mut updated_count = 0;
        for (row_index, new_row) in updated_rows {
            if row_index < table_data.len() {
                table_data[row_index] = new_row;
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
    
    /// Check primary key constraint for a tuple against existing data
    fn check_primary_key_constraint(
        &self,
        new_tuple: &Tuple,
        primary_key_columns: &[usize],
        table_id: u32
    ) -> Result<(), ExecutionError> {
        // Get existing table data
        let existing_data = self.table_data.get(&table_id)
            .ok_or_else(|| ExecutionError::TableNotFound { 
                table: format!("table_id_{}", table_id) 
            })?;
        
        // Extract primary key values from the new tuple
        let mut new_key_values = Vec::new();
        for &col_index in primary_key_columns {
            if col_index >= new_tuple.values.len() {
                return Err(ExecutionError::EvaluationError {
                    message: format!("Primary key column index {} out of bounds", col_index)
                });
            }
            new_key_values.push(new_tuple.values[col_index].clone());
        }
        
        // Check against existing tuples
        for existing_tuple in existing_data {
            let mut existing_key_values = Vec::new();
            for &col_index in primary_key_columns {
                if col_index >= existing_tuple.values.len() {
                    continue; // Skip malformed data
                }
                existing_key_values.push(existing_tuple.values[col_index].clone());
            }
            
            // Compare key values
            if new_key_values == existing_key_values {
                // Found duplicate primary key
                let key_str = new_key_values.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                return Err(ExecutionError::PrimaryKeyViolation {
                    key: format!("({})", key_str)
                });
            }
        }
        
        Ok(())
    }

    /// 获取所有列名，用于错误诊断
    fn get_all_column_names(&self) -> Vec<String> {
        let mut column_names = Vec::new();
        for schema in self.table_schemas.values() {
            for column in &schema.columns {
                if !column_names.contains(&column.name) {
                    column_names.push(column.name.clone());
                }
            }
        }
        column_names
    }
}
