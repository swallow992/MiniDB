//! 数据类型和值系统
//!
//! 此模块定义了整个 MiniDB 中使用的类型系统，
//! 包括数据类型、值和模式定义。

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// MiniDB 支持的 SQL 数据类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// 32位有符号整数
    Integer,
    /// 64位有符号整数
    BigInt,
    /// 32位浮点数
    Float,
    /// 64位浮点数
    Double,
    /// 可变长度字符串，带最大长度限制
    Varchar(usize),
    /// 布尔值 true/false
    Boolean,
    /// 日期（不含时间）
    Date,
    /// 日期和时间
    Timestamp,
}

/// 可以存储在数据库中的运行时值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// NULL 值
    Null,
    /// 整数值
    Integer(i32),
    /// 大整数值
    BigInt(i64),
    /// 浮点数值
    Float(f32),
    /// 双精度浮点数值
    Double(f64),
    /// 字符串值
    Varchar(String),
    /// 布尔值
    Boolean(bool),
    /// 日期值
    Date(NaiveDate),
    /// 时间戳值
    Timestamp(NaiveDateTime),
}

// 为 Value 自定义实现，用于处理浮点数比较
impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Integer(i) => i.hash(state),
            Value::BigInt(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Double(f) => f.to_bits().hash(state),
            Value::Varchar(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Date(d) => d.hash(state),
            Value::Timestamp(t) => t.hash(state),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (self, other) {
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Null, _) => Some(Ordering::Less),
            (_, Value::Null) => Some(Ordering::Greater),
            
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::BigInt(a), Value::BigInt(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Double(a), Value::Double(b)) => a.partial_cmp(b),
            (Value::Varchar(a), Value::Varchar(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Date(a), Value::Date(b)) => a.partial_cmp(b),
            (Value::Timestamp(a), Value::Timestamp(b)) => a.partial_cmp(b),
            
            // 数值类型的类型提升
            (Value::Integer(a), Value::BigInt(b)) => (*a as i64).partial_cmp(b),
            (Value::BigInt(a), Value::Integer(b)) => a.partial_cmp(&(*b as i64)),
            (Value::Integer(a), Value::Float(b)) => (*a as f32).partial_cmp(b),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f32)),
            (Value::Integer(a), Value::Double(b)) => (*a as f64).partial_cmp(b),
            (Value::Double(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Float(a), Value::Double(b)) => (*a as f64).partial_cmp(b),
            (Value::Double(a), Value::Float(b)) => a.partial_cmp(&(*b as f64)),
            
            // 不同类型不可比较
            _ => None,
        }
    }
}

/// 数据库元组（行），包含多个值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuple {
    pub values: Vec<Value>,
}

/// 表模式中的列定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
}

/// 包含列定义的表模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub columns: Vec<ColumnDefinition>,
    pub primary_key: Option<Vec<usize>>, // 构成主键的列索引
}

/// 与类型操作相关的错误
#[derive(Error, Debug)]
pub enum TypeError {
    #[error("类型不匹配：期望 {expected:?}，实际 {found:?}")]
    Mismatch { expected: DataType, found: DataType },

    #[error("无效的类型转换：从 {from:?} 到 {to:?}")]
    InvalidCast { from: DataType, to: DataType },

    #[error("非空列中存在 NULL 值")]
    NullConstraintViolation,

    #[error("字符串过长：最大长度 {max}，实际长度 {actual}")]
    StringTooLong { max: usize, actual: usize },
}

impl DataType {
    /// 获取固定大小类型的字节大小
    pub fn size(&self) -> Option<usize> {
        match self {
            DataType::Integer => Some(4),
            DataType::BigInt => Some(8),
            DataType::Float => Some(4),
            DataType::Double => Some(8),
            DataType::Boolean => Some(1),
            DataType::Date => Some(4),      // 自纪元以来的天数
            DataType::Timestamp => Some(8), // 自纪元以来的微秒数
            DataType::Varchar(_) => None,   // 可变大小
        }
    }

    /// 检查此类型是否与另一个类型兼容
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        match (self, other) {
            (a, b) if a == b => true,
            (DataType::Integer, DataType::BigInt) => true,
            (DataType::BigInt, DataType::Integer) => true,
            (DataType::Float, DataType::Double) => true,
            (DataType::Double, DataType::Float) => true,
            (DataType::Integer, DataType::Float) => true,
            (DataType::Integer, DataType::Double) => true,
            // Varchar 兼容性：较小的字符串可以适配较大的 varchar 列
            (DataType::Varchar(len1), DataType::Varchar(len2)) => len1 <= len2,
            _ => false,
        }
    }
}

impl Value {
    /// 获取此值的数据类型
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Null => DataType::Varchar(0), // Null 可以是任何类型
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

    /// 检查此值是否与数据类型兼容
    pub fn is_compatible_with(&self, data_type: &DataType) -> bool {
        match self {
            Value::Null => true, // Null 与任何类型兼容
            _ => self.data_type().is_compatible_with(data_type),
        }
    }

    /// 尝试将此值转换为另一种类型
    pub fn cast_to(&self, target_type: &DataType) -> Result<Value, TypeError> {
        match (self, target_type) {
            (Value::Null, _) => Ok(Value::Null),
            (value, target) if value.data_type() == *target => Ok(value.clone()),

            // 整数转换
            (Value::Integer(i), DataType::BigInt) => Ok(Value::BigInt(*i as i64)),
            (Value::Integer(i), DataType::Float) => Ok(Value::Float(*i as f32)),
            (Value::Integer(i), DataType::Double) => Ok(Value::Double(*i as f64)),
            (Value::Integer(i), DataType::Varchar(_)) => Ok(Value::Varchar(i.to_string())),

            // 字符串转换
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

    /// 获取此值的序列化字节大小
    pub fn serialized_size(&self) -> usize {
        match self {
            Value::Null => 1, // Null 标记
            Value::Integer(_) => 4,
            Value::BigInt(_) => 8,
            Value::Float(_) => 4,
            Value::Double(_) => 8,
            Value::Varchar(s) => 4 + s.len(), // 长度前缀 + 字符串数据
            Value::Boolean(_) => 1,
            Value::Date(_) => 4,
            Value::Timestamp(_) => 8,
        }
    }
}

impl Tuple {
    /// 使用给定值创建新元组
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// 根据列索引获取值
    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// 获取此元组的总序列化大小
    pub fn size(&self) -> usize {
        self.values.iter().map(|v| v.serialized_size()).sum()
    }

    /// 检查此元组是否符合给定模式
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
                Value::Null => continue, // 允许 Null 值
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
    /// 使用给定的列定义创建新模式
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self { 
            columns,
            primary_key: None,
        }
    }
    
    /// 创建带有主键的新模式
    pub fn new_with_primary_key(columns: Vec<ColumnDefinition>, primary_key: Vec<usize>) -> Self {
        Self {
            columns,
            primary_key: Some(primary_key),
        }
    }

    /// 根据名称查找列
    pub fn find_column(&self, name: &str) -> Option<(usize, &ColumnDefinition)> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, col)| col.name == name)
    }

    /// 获取列的总数
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

impl ColumnDefinition {
    /// 创建新的列定义
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            data_type,
            nullable,
            default: None,
        }
    }

    /// 为此列设置默认值
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
