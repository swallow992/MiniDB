//! Data types and value system
//!
//! This module defines the type system used throughout MiniDB,
//! including data types, values, and schema definitions.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// SQL data types supported by MiniDB
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// 32-bit signed integer
    Integer,
    /// 64-bit signed integer
    BigInt,
    /// 32-bit floating point
    Float,
    /// 64-bit floating point
    Double,
    /// Variable-length string with maximum length
    Varchar(usize),
    /// Boolean true/false
    Boolean,
    /// Date without time
    Date,
    /// Date and time
    Timestamp,
}

/// Runtime values that can be stored in the database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// NULL value
    Null,
    /// Integer value
    Integer(i32),
    /// Big integer value
    BigInt(i64),
    /// Float value
    Float(f32),
    /// Double precision value
    Double(f64),
    /// String value
    Varchar(String),
    /// Boolean value
    Boolean(bool),
    /// Date value
    Date(NaiveDate),
    /// Timestamp value
    Timestamp(NaiveDateTime),
}

/// Database tuple (row) containing multiple values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuple {
    pub values: Vec<Value>,
}

/// Column definition in a table schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
}

/// Table schema containing column definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<ColumnDefinition>,
}

/// Errors related to type operations
#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch { expected: DataType, found: DataType },

    #[error("Invalid cast from {from:?} to {to:?}")]
    InvalidCast { from: DataType, to: DataType },

    #[error("Null value in non-nullable column")]
    NullConstraintViolation,

    #[error("String too long: max length {max}, got {actual}")]
    StringTooLong { max: usize, actual: usize },
}

impl DataType {
    /// Get the size in bytes for fixed-size types
    pub fn size(&self) -> Option<usize> {
        match self {
            DataType::Integer => Some(4),
            DataType::BigInt => Some(8),
            DataType::Float => Some(4),
            DataType::Double => Some(8),
            DataType::Boolean => Some(1),
            DataType::Date => Some(4),      // Days since epoch
            DataType::Timestamp => Some(8), // Microseconds since epoch
            DataType::Varchar(_) => None,   // Variable size
        }
    }

    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            (DataType::Integer, DataType::BigInt) => true,
            (DataType::BigInt, DataType::Integer) => true,
            (DataType::Float, DataType::Double) => true,
            (DataType::Double, DataType::Float) => true,
            (DataType::Integer, DataType::Float) => true,
            (DataType::Integer, DataType::Double) => true,
            _ => false,
        }
    }
}

impl Value {
    /// Get the data type of this value
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Null => DataType::Varchar(0), // Null can be any type
            Value::Integer(_) => DataType::Integer,
            Value::BigInt(_) => DataType::BigInt,
            Value::Float(_) => DataType::Float,
            Value::Double(_) => DataType::Double,
            Value::Varchar(s) => DataType::Varchar(s.len()),
            Value::Boolean(_) => DataType::Boolean,
            Value::Date(_) => DataType::Date,
            Value::Timestamp(_) => DataType::Timestamp,
        }
    }

    /// Check if this value is compatible with a data type
    pub fn is_compatible_with(&self, data_type: &DataType) -> bool {
        match self {
            Value::Null => true, // Null is compatible with any type
            _ => self.data_type().is_compatible_with(data_type),
        }
    }

    /// Attempt to cast this value to another type
    pub fn cast_to(&self, target_type: &DataType) -> Result<Value, TypeError> {
        match (self, target_type) {
            (Value::Null, _) => Ok(Value::Null),
            (value, target) if value.data_type() == *target => Ok(value.clone()),

            // Integer conversions
            (Value::Integer(i), DataType::BigInt) => Ok(Value::BigInt(*i as i64)),
            (Value::Integer(i), DataType::Float) => Ok(Value::Float(*i as f32)),
            (Value::Integer(i), DataType::Double) => Ok(Value::Double(*i as f64)),
            (Value::Integer(i), DataType::Varchar(_)) => Ok(Value::Varchar(i.to_string())),

            // String conversions
            (Value::Varchar(s), DataType::Integer) => {
                s.parse::<i32>()
                    .map(Value::Integer)
                    .map_err(|_| TypeError::InvalidCast {
                        from: DataType::Varchar(s.len()),
                        to: target_type.clone(),
                    })
            }

            _ => Err(TypeError::InvalidCast {
                from: self.data_type(),
                to: target_type.clone(),
            }),
        }
    }

    /// Get the serialized size of this value in bytes
    pub fn serialized_size(&self) -> usize {
        match self {
            Value::Null => 1, // Null marker
            Value::Integer(_) => 4,
            Value::BigInt(_) => 8,
            Value::Float(_) => 4,
            Value::Double(_) => 8,
            Value::Varchar(s) => 4 + s.len(), // Length prefix + string data
            Value::Boolean(_) => 1,
            Value::Date(_) => 4,
            Value::Timestamp(_) => 8,
        }
    }
}

impl Tuple {
    /// Create a new tuple with the given values
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Get a value by column index
    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Get the total serialized size of this tuple
    pub fn size(&self) -> usize {
        self.values.iter().map(|v| v.serialized_size()).sum()
    }

    /// Check if this tuple conforms to the given schema
    pub fn conforms_to_schema(&self, schema: &Schema) -> Result<(), TypeError> {
        if self.values.len() != schema.columns.len() {
            return Err(TypeError::Mismatch {
                expected: DataType::Varchar(schema.columns.len()),
                found: DataType::Varchar(self.values.len()),
            });
        }

        for (value, column) in self.values.iter().zip(&schema.columns) {
            match value {
                Value::Null if !column.nullable => {
                    return Err(TypeError::NullConstraintViolation);
                }
                Value::Null => continue, // Null is allowed
                _ => {
                    if !value.is_compatible_with(&column.data_type) {
                        return Err(TypeError::Mismatch {
                            expected: column.data_type.clone(),
                            found: value.data_type(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

impl Schema {
    /// Create a new schema with the given column definitions
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self { columns }
    }

    /// Find a column by name
    pub fn find_column(&self, name: &str) -> Option<(usize, &ColumnDefinition)> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, col)| col.name == name)
    }

    /// Get the total number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

impl ColumnDefinition {
    /// Create a new column definition
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            data_type,
            nullable,
            default: None,
        }
    }

    /// Set a default value for this column
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);
        self
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::BigInt(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Double(d) => write!(f, "{}", d),
            Value::Varchar(s) => write!(f, "'{}'", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Date(d) => write!(f, "{}", d),
            Value::Timestamp(ts) => write!(f, "{}", ts),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::BigInt => write!(f, "BIGINT"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Varchar(len) => write!(f, "VARCHAR({})", len),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Date => write!(f, "DATE"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
        }
    }
}
