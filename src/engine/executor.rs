//! Query executor

use crate::sql::parser::Expression;
use crate::sql::planner::{JoinType, SortKey};
use crate::types::{DataType, Schema, Tuple, Value, ColumnDefinition};
use std::collections::HashMap;
use thiserror::Error;

pub trait Executor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError>;
    fn schema(&self) -> &Schema;
    fn reset(&mut self) -> Result<(), ExecutorError>;
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<Tuple>,
    pub schema: Schema,
}

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Not implemented")]
    NotImplemented,
    
    #[error("Type error: {message}")]
    TypeError { message: String },
    
    #[error("Evaluation error: {message}")]
    EvaluationError { message: String },
    
    #[error("Join error: {message}")]
    JoinError { message: String },
}

/// Hash Join executor - builds hash table from left input, probes with right input
pub struct HashJoinExecutor {
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    join_type: JoinType,
    condition: Option<Expression>,
    hash_table: HashMap<String, Vec<Tuple>>,
    right_tuples: Vec<Tuple>,
    current_right_index: usize,
    current_matches: Vec<Tuple>,
    current_match_index: usize,
    schema: Schema,
    built: bool,
}

impl HashJoinExecutor {
    pub fn new(
        left: Box<dyn Executor>,
        right: Box<dyn Executor>,
        join_type: JoinType,
        condition: Option<Expression>,
    ) -> Result<Self, ExecutorError> {
        // Combine schemas from left and right
        let left_schema = left.schema().clone();
        let right_schema = right.schema().clone();
        
        let mut combined_columns = left_schema.columns;
        combined_columns.extend(right_schema.columns);
        
        let schema = Schema {
            columns: combined_columns,
            primary_key: None, // JOIN results don't have primary key
        };

        Ok(Self {
            left,
            right,
            join_type,
            condition,
            hash_table: HashMap::new(),
            right_tuples: Vec::new(),
            current_right_index: 0,
            current_matches: Vec::new(),
            current_match_index: 0,
            schema,
            built: false,
        })
    }

    fn build_hash_table(&mut self) -> Result<(), ExecutorError> {
        if self.built {
            return Ok(());
        }

        // Build hash table from left input
        while let Some(tuple) = self.left.next()? {
            // For simplicity, use first column as hash key
            // In a real implementation, this would be based on join condition
            let key = if !tuple.values.is_empty() {
                format!("{:?}", tuple.values[0])
            } else {
                "NULL".to_string()
            };
            
            self.hash_table.entry(key).or_insert_with(Vec::new).push(tuple);
        }

        // Collect all right tuples
        while let Some(tuple) = self.right.next()? {
            self.right_tuples.push(tuple);
        }

        self.built = true;
        Ok(())
    }

    fn evaluate_join_condition(&self, left_tuple: &Tuple, right_tuple: &Tuple) -> Result<bool, ExecutorError> {
        match &self.condition {
            Some(expr) => {
                // For simplicity, we'll implement basic equality check
                // In a real implementation, this would be a full expression evaluator
                self.evaluate_expression(expr, left_tuple, right_tuple)
            }
            None => Ok(true), // Cross join
        }
    }

    fn evaluate_expression(&self, _expr: &Expression, _left: &Tuple, _right: &Tuple) -> Result<bool, ExecutorError> {
        // Simplified implementation - assume condition is always true for now
        // In a real implementation, this would evaluate the full expression
        Ok(true)
    }

    fn combine_tuples(&self, left: &Tuple, right: &Tuple) -> Tuple {
        let mut combined_values = left.values.clone();
        combined_values.extend(right.values.clone());
        
        Tuple {
            values: combined_values,
        }
    }
}

impl Executor for HashJoinExecutor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError> {
        self.build_hash_table()?;

        loop {
            // If we have matches for the current right tuple, return them
            if self.current_match_index < self.current_matches.len() {
                let right_tuple = &self.right_tuples[self.current_right_index];
                let left_tuple = &self.current_matches[self.current_match_index];
                self.current_match_index += 1;
                
                if self.evaluate_join_condition(left_tuple, right_tuple)? {
                    return Ok(Some(self.combine_tuples(left_tuple, right_tuple)));
                }
            }

            // Move to next right tuple
            if self.current_right_index >= self.right_tuples.len() {
                return Ok(None);
            }

            let right_tuple = &self.right_tuples[self.current_right_index];
            let key = if !right_tuple.values.is_empty() {
                format!("{:?}", right_tuple.values[0])
            } else {
                "NULL".to_string()
            };

            // Find matches in hash table
            self.current_matches = self.hash_table.get(&key).cloned().unwrap_or_default();
            self.current_match_index = 0;
            self.current_right_index += 1;

            // Handle different join types
            match self.join_type {
                JoinType::Inner => {
                    if self.current_matches.is_empty() {
                        continue; // No matches, skip this right tuple
                    }
                }
                JoinType::Left => {
                    if self.current_matches.is_empty() {
                        // Left join: return right tuple with NULL values for left
                        let null_values = vec![Value::Null; self.left.schema().columns.len()];
                        let null_tuple = Tuple { values: null_values };
                        return Ok(Some(self.combine_tuples(&null_tuple, right_tuple)));
                    }
                }
                _ => {
                    return Err(ExecutorError::NotImplemented);
                }
            }
        }
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn reset(&mut self) -> Result<(), ExecutorError> {
        self.left.reset()?;
        self.right.reset()?;
        self.hash_table.clear();
        self.right_tuples.clear();
        self.current_right_index = 0;
        self.current_matches.clear();
        self.current_match_index = 0;
        self.built = false;
        Ok(())
    }
}

/// Sort executor
pub struct SortExecutor {
    input: Box<dyn Executor>,
    sort_keys: Vec<SortKey>,
    sorted_tuples: Vec<Tuple>,
    current_index: usize,
    schema: Schema,
    sorted: bool,
}

impl SortExecutor {
    pub fn new(input: Box<dyn Executor>, sort_keys: Vec<SortKey>) -> Self {
        let schema = input.schema().clone();
        
        Self {
            input,
            sort_keys,
            sorted_tuples: Vec::new(),
            current_index: 0,
            schema,
            sorted: false,
        }
    }

    fn sort_tuples(&mut self) -> Result<(), ExecutorError> {
        if self.sorted {
            return Ok(());
        }

        // Collect all tuples
        while let Some(tuple) = self.input.next()? {
            self.sorted_tuples.push(tuple);
        }

        // Sort based on sort keys - clone to avoid borrowing issues
        let sort_keys = self.sort_keys.clone();
        self.sorted_tuples.sort_by(|a, b| {
            for sort_key in &sort_keys {
                // Simplified comparison - just compare first values for now
                let a_val = a.values.get(0).unwrap_or(&Value::Null);
                let b_val = b.values.get(0).unwrap_or(&Value::Null);
                
                if let Some(cmp) = a_val.partial_cmp(b_val) {
                    match cmp {
                        std::cmp::Ordering::Equal => continue,
                        other => {
                            return if sort_key.descending {
                                other.reverse()
                            } else {
                                other
                            };
                        }
                    }
                }
            }
            std::cmp::Ordering::Equal
        });

        self.sorted = true;
        Ok(())
    }

    #[allow(dead_code)]
    fn compare_tuples_by_expression(&self, a: &Tuple, b: &Tuple, expr: &Expression) -> std::cmp::Ordering {
        // Simplified comparison - in a real implementation, this would evaluate expressions
        match expr {
            Expression::Column(col_name) => {
                // Find column index and compare values
                if let Some(col_index) = self.find_column_index(col_name) {
                    if col_index < a.values.len() && col_index < b.values.len() {
                        return self.compare_values(&a.values[col_index], &b.values[col_index]);
                    }
                }
                std::cmp::Ordering::Equal
            }
            _ => std::cmp::Ordering::Equal, // TODO: Handle other expression types
        }
    }

    #[allow(dead_code)]
    fn find_column_index(&self, col_name: &str) -> Option<usize> {
        self.schema.columns.iter().position(|c| c.name == col_name)
    }

    #[allow(dead_code)]
    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::BigInt(a), Value::BigInt(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Varchar(a), Value::Varchar(b)) => a.cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,
            _ => Ordering::Equal, // Type coercion would be handled here
        }
    }
}

impl Executor for SortExecutor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError> {
        self.sort_tuples()?;

        if self.current_index < self.sorted_tuples.len() {
            let tuple = self.sorted_tuples[self.current_index].clone();
            self.current_index += 1;
            Ok(Some(tuple))
        } else {
            Ok(None)
        }
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn reset(&mut self) -> Result<(), ExecutorError> {
        self.input.reset()?;
        self.sorted_tuples.clear();
        self.current_index = 0;
        self.sorted = false;
        Ok(())
    }
}

/// Limit executor
pub struct LimitExecutor {
    input: Box<dyn Executor>,
    limit: u64,
    offset: u64,
    current_count: u64,
    skipped_count: u64,
    schema: Schema,
}

impl LimitExecutor {
    pub fn new(input: Box<dyn Executor>, limit: u64, offset: u64) -> Self {
        let schema = input.schema().clone();
        
        Self {
            input,
            limit,
            offset,
            current_count: 0,
            skipped_count: 0,
            schema,
        }
    }
}

impl Executor for LimitExecutor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError> {
        // Skip tuples for OFFSET
        while self.skipped_count < self.offset {
            if self.input.next()?.is_some() {
                self.skipped_count += 1;
            } else {
                return Ok(None);
            }
        }

        // Return tuples up to LIMIT
        if self.current_count < self.limit {
            if let Some(tuple) = self.input.next()? {
                self.current_count += 1;
                Ok(Some(tuple))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn reset(&mut self) -> Result<(), ExecutorError> {
        self.input.reset()?;
        self.current_count = 0;
        self.skipped_count = 0;
        Ok(())
    }
}

/// Aggregate function types for GROUP BY
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum(String),   // column name
    Avg(String),   // column name
    Min(String),   // column name
    Max(String),   // column name
}

/// Accumulator for aggregate functions
#[derive(Debug, Clone)]
pub struct AggregateAccumulator {
    pub count: u64,
    pub sum: Option<f64>,
    pub min: Option<Value>,
    pub max: Option<Value>,
}

impl AggregateAccumulator {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: None,
            min: None,
            max: None,
        }
    }

    pub fn update(&mut self, value: &Value) -> Result<(), ExecutorError> {
        self.count += 1;

        match value {
            Value::Integer(i) => {
                let val = *i as f64;
                self.sum = Some(self.sum.unwrap_or(0.0) + val);
                
                let int_val = Value::Integer(*i);
                if self.min.is_none() || self.compare_values(&int_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(int_val.clone());
                }
                if self.max.is_none() || self.compare_values(&int_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(int_val);
                }
            },
            Value::Float(f) => {
                let val = *f as f64;
                self.sum = Some(self.sum.unwrap_or(0.0) + val);
                
                let float_val = Value::Float(*f);
                if self.min.is_none() || self.compare_values(&float_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(float_val.clone());
                }
                if self.max.is_none() || self.compare_values(&float_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(float_val);
                }
            },
            Value::Double(d) => {
                self.sum = Some(self.sum.unwrap_or(0.0) + d);
                
                let double_val = Value::Double(*d);
                if self.min.is_none() || self.compare_values(&double_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(double_val.clone());
                }
                if self.max.is_none() || self.compare_values(&double_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(double_val);
                }
            },
            Value::Varchar(s) => {
                let str_val = Value::Varchar(s.clone());
                if self.min.is_none() || self.compare_values(&str_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(str_val.clone());
                }
                if self.max.is_none() || self.compare_values(&str_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(str_val);
                }
            },
            Value::Boolean(_) => {
                // For boolean values, we only count
            },
            Value::BigInt(i) => {
                let val = *i as f64;
                self.sum = Some(self.sum.unwrap_or(0.0) + val);
                
                let bigint_val = Value::BigInt(*i);
                if self.min.is_none() || self.compare_values(&bigint_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(bigint_val.clone());
                }
                if self.max.is_none() || self.compare_values(&bigint_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(bigint_val);
                }
            },
            Value::Date(d) => {
                let date_val = Value::Date(*d);
                if self.min.is_none() || self.compare_values(&date_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(date_val.clone());
                }
                if self.max.is_none() || self.compare_values(&date_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(date_val);
                }
            },
            Value::Timestamp(ts) => {
                let ts_val = Value::Timestamp(*ts);
                if self.min.is_none() || self.compare_values(&ts_val, self.min.as_ref().unwrap())? < 0 {
                    self.min = Some(ts_val.clone());
                }
                if self.max.is_none() || self.compare_values(&ts_val, self.max.as_ref().unwrap())? > 0 {
                    self.max = Some(ts_val);
                }
            },
            Value::Null => {
                // Null values are typically ignored in aggregation
                self.count -= 1; // Don't count nulls
            },
        }

        Ok(())
    }

    fn compare_values(&self, a: &Value, b: &Value) -> Result<i32, ExecutorError> {
        match (a, b) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a.cmp(b) as i32),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(a.cmp(b) as i32),
            (Value::Float(a), Value::Float(b)) => Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32),
            (Value::Double(a), Value::Double(b)) => Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32),
            (Value::Varchar(a), Value::Varchar(b)) => Ok(a.cmp(b) as i32),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(a.cmp(b) as i32),
            (Value::Date(a), Value::Date(b)) => Ok(a.cmp(b) as i32),
            (Value::Timestamp(a), Value::Timestamp(b)) => Ok(a.cmp(b) as i32),
            _ => Err(ExecutorError::TypeError {
                message: format!("Cannot compare {:?} and {:?}", a, b),
            }),
        }
    }

    pub fn get_result(&self, func: &AggregateFunction) -> Value {
        match func {
            AggregateFunction::Count => Value::Integer(self.count as i32),
            AggregateFunction::Sum(_) => {
                if let Some(sum) = self.sum {
                    Value::Double(sum)
                } else {
                    Value::Null
                }
            },
            AggregateFunction::Avg(_) => {
                if let Some(sum) = self.sum {
                    if self.count > 0 {
                        Value::Double(sum / self.count as f64)
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            },
            AggregateFunction::Min(_) => {
                self.min.clone().unwrap_or(Value::Null)
            },
            AggregateFunction::Max(_) => {
                self.max.clone().unwrap_or(Value::Null)
            },
        }
    }
}

/// GROUP BY executor with aggregation
pub struct GroupByExecutor {
    input: Box<dyn Executor>,
    group_expressions: Vec<Expression>,
    aggregate_functions: Vec<AggregateFunction>,
    groups: HashMap<Vec<Value>, Vec<AggregateAccumulator>>,
    result_iterator: std::vec::IntoIter<Tuple>,
    schema: Schema,
    materialized: bool,
}

impl GroupByExecutor {
    pub fn new(
        input: Box<dyn Executor>,
        group_expressions: Vec<Expression>,
        aggregate_functions: Vec<AggregateFunction>,
    ) -> Self {
        // Create schema: group columns + aggregate columns
        let mut columns = Vec::new();
        
        // Add group by columns
        for (i, _) in group_expressions.iter().enumerate() {
            columns.push(ColumnDefinition {
                name: format!("group_{}", i),
                data_type: DataType::Varchar(255), // Simplified for now
                nullable: false,
                default: None,
            });
        }
        
        // Add aggregate columns
        for func in &aggregate_functions {
            let col_name = match func {
                AggregateFunction::Count => "count".to_string(),
                AggregateFunction::Sum(col) => format!("sum_{}", col),
                AggregateFunction::Avg(col) => format!("avg_{}", col),
                AggregateFunction::Min(col) => format!("min_{}", col),
                AggregateFunction::Max(col) => format!("max_{}", col),
            };
            
            columns.push(ColumnDefinition {
                name: col_name,
                data_type: DataType::Double, // Use double for aggregates
                nullable: true,
                default: None,
            });
        }
        
        let schema = Schema { columns, primary_key: None };
        
        Self {
            input,
            group_expressions,
            aggregate_functions,
            groups: HashMap::new(),
            result_iterator: Vec::new().into_iter(),
            schema,
            materialized: false,
        }
    }

    fn materialize(&mut self) -> Result<(), ExecutorError> {
        if self.materialized {
            return Ok(());
        }

        // Process all input tuples
        while let Some(tuple) = self.input.next()? {
            // Evaluate group by expressions
            let group_key: Vec<Value> = self.group_expressions
                .iter()
                .map(|expr| self.evaluate_expression(expr, &tuple))
                .collect::<Result<_, _>>()?;

            // Get or create accumulators for this group
            let accumulators = self.groups.entry(group_key).or_insert_with(|| {
                self.aggregate_functions
                    .iter()
                    .map(|_| AggregateAccumulator::new())
                    .collect()
            });

            // Update accumulators
            for (i, func) in self.aggregate_functions.iter().enumerate() {
                match func {
                    AggregateFunction::Count => {
                        accumulators[i].count += 1;
                    },
                    AggregateFunction::Sum(col_name) |
                    AggregateFunction::Avg(col_name) |
                    AggregateFunction::Min(col_name) |
                    AggregateFunction::Max(col_name) => {
                        if let Some(col_idx) = self.input.schema().columns.iter().position(|c| &c.name == col_name) {
                            accumulators[i].update(&tuple.values[col_idx])?;
                        }
                    },
                }
            }
        }

        // Convert groups to result tuples
        let mut results = Vec::new();
        for (group_key, accumulators) in &self.groups {
            let mut values = group_key.clone();
            
            // Add aggregate results
            for (i, func) in self.aggregate_functions.iter().enumerate() {
                values.push(accumulators[i].get_result(func));
            }
            
            results.push(Tuple { values });
        }

        self.result_iterator = results.into_iter();
        self.materialized = true;
        Ok(())
    }

    fn evaluate_expression(&self, expr: &Expression, tuple: &Tuple) -> Result<Value, ExecutorError> {
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Column(name) => {
                if let Some(index) = self.input.schema().columns.iter().position(|c| &c.name == name) {
                    Ok(tuple.values[index].clone())
                } else {
                    Err(ExecutorError::EvaluationError {
                        message: format!("Column not found: {}", name),
                    })
                }
            },
            Expression::QualifiedColumn { table: _, column } => {
                if let Some(index) = self.input.schema().columns.iter().position(|c| &c.name == column) {
                    Ok(tuple.values[index].clone())
                } else {
                    Err(ExecutorError::EvaluationError {
                        message: format!("Column not found: {}", column),
                    })
                }
            },
            _ => {
                // Simplified expression evaluation
                Ok(Value::Null)
            }
        }
    }
}

impl Executor for GroupByExecutor {
    fn next(&mut self) -> Result<Option<Tuple>, ExecutorError> {
        if !self.materialized {
            self.materialize()?;
        }
        
        Ok(self.result_iterator.next())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn reset(&mut self) -> Result<(), ExecutorError> {
        self.input.reset()?;
        self.groups.clear();
        self.result_iterator = Vec::new().into_iter();
        self.materialized = false;
        Ok(())
    }
}
