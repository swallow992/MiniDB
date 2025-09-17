//! SQL 语义分析器
//!
//! 对解析后的 SQL 语句执行语义分析，包括：
//! - 类型检查
//! - 符号解析
//! - 约束验证
//! - 模式验证

use crate::sql::parser::{BinaryOperator, Expression, Statement, UnaryOperator};
use crate::types::{ColumnDefinition, DataType, Schema, Value};
use std::collections::HashMap;
use thiserror::Error;

/// 已分析的 SQL 语句，包含已解析的类型和符号
#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    /// 原始语句
    pub statement: Statement,
    /// 引用表的模式信息
    pub table_schemas: HashMap<String, Schema>,
    /// 已解析的表达式类型
    pub expression_types: HashMap<String, DataType>,
}

/// 模式查找的目录接口
pub trait SchemaCatalog {
    /// 获取表的模式
    fn get_table_schema(&self, table_name: &str) -> Option<Schema>;
    /// 检查表是否存在
    fn table_exists(&self, table_name: &str) -> bool;
}

/// 用于测试的简单内存目录
#[derive(Debug, Default)]
pub struct MemoryCatalog {
    schemas: HashMap<String, Schema>,
}

impl MemoryCatalog {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, table_name: String, schema: Schema) {
        self.schemas.insert(table_name, schema);
    }
}

impl SchemaCatalog for MemoryCatalog {
    fn get_table_schema(&self, table_name: &str) -> Option<Schema> {
        self.schemas.get(table_name).cloned()
    }

    fn table_exists(&self, table_name: &str) -> bool {
        self.schemas.contains_key(table_name)
    }
}

/// SQL 语义分析器
pub struct SemanticAnalyzer<'a> {
    catalog: &'a dyn SchemaCatalog,
}

/// 语义分析错误
#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("未找到表: {table}")]
    TableNotFound {
        table: String,
        position: Option<(u32, u32)>,
    },

    #[error("在表 {table} 中未找到列: {column}")]
    ColumnNotFound {
        table: String,
        column: String,
        position: Option<(u32, u32)>,
    },

    #[error("列引用不明确: {column}")]
    AmbiguousColumn {
        column: String,
        position: Option<(u32, u32)>,
    },

    #[error("类型不匹配: 期望 {expected:?}, 发现 {found:?}")]
    TypeMismatch {
        expected: DataType,
        found: DataType,
        position: Option<(u32, u32)>,
    },

    #[error("在类型 {left:?} 和 {right:?} 上执行无效操作 {op:?}")]
    InvalidBinaryOperation {
        op: BinaryOperator,
        left: DataType,
        right: DataType,
        position: Option<(u32, u32)>,
    },

    #[error("在类型 {operand:?} 上执行无效一元操作 {op:?}")]
    InvalidUnaryOperation {
        op: UnaryOperator,
        operand: DataType,
        position: Option<(u32, u32)>,
    },

    #[error("重复的列名: {column}")]
    DuplicateColumn {
        column: String,
        position: Option<(u32, u32)>,
    },

    #[error("表已存在: {table}")]
    TableAlreadyExists {
        table: String,
        position: Option<(u32, u32)>,
    },

    #[error("插入列数不匹配: 期望 {expected}, 实际 {actual}")]
    InsertColumnMismatch {
        expected: usize,
        actual: usize,
        position: Option<(u32, u32)>,
    },

    #[error("列 {column} 不能为空")]
    NullConstraintViolation {
        column: String,
        position: Option<(u32, u32)>,
    },
}

impl SemanticError {
    /// 创建带默认位置的 TableNotFound 错误
    pub fn table_not_found(table: String) -> Self {
        SemanticError::TableNotFound {
            table,
            position: None,
        }
    }

    /// 创建带默认位置的 ColumnNotFound 错误
    pub fn column_not_found(table: String, column: String) -> Self {
        SemanticError::ColumnNotFound {
            table,
            column,
            position: None,
        }
    }

    /// 创建带默认位置的 TypeMismatch 错误
    pub fn type_mismatch(expected: DataType, found: DataType) -> Self {
        SemanticError::TypeMismatch {
            expected,
            found,
            position: None,
        }
    }

    /// 创建带默认位置的 DuplicateColumn 错误
    pub fn duplicate_column(column: String) -> Self {
        SemanticError::DuplicateColumn {
            column,
            position: None,
        }
    }

    /// 创建带默认位置的 TableAlreadyExists 错误
    pub fn table_already_exists(table: String) -> Self {
        SemanticError::TableAlreadyExists {
            table,
            position: None,
        }
    }

    /// 创建带默认位置的 InsertColumnMismatch 错误
    pub fn insert_column_mismatch(expected: usize, actual: usize) -> Self {
        SemanticError::InsertColumnMismatch {
            expected,
            actual,
            position: None,
        }
    }

    /// 创建带默认位置的 NullConstraintViolation 错误
    pub fn null_constraint_violation(column: String) -> Self {
        SemanticError::NullConstraintViolation {
            column,
            position: None,
        }
    }

    /// 创建带默认位置的 InvalidBinaryOperation 错误
    pub fn invalid_binary_operation(op: BinaryOperator, left: DataType, right: DataType) -> Self {
        SemanticError::InvalidBinaryOperation {
            op,
            left,
            right,
            position: None,
        }
    }

    /// 创建带默认位置的 InvalidUnaryOperation 错误
    pub fn invalid_unary_operation(op: UnaryOperator, operand: DataType) -> Self {
        SemanticError::InvalidUnaryOperation {
            op,
            operand,
            position: None,
        }
    }

    /// 创建带默认位置的 AmbiguousColumn 错误
    pub fn ambiguous_column(column: String) -> Self {
        SemanticError::AmbiguousColumn {
            column,
            position: None,
        }
    }

    /// 格式化错误输出为 [错误类型，位置，原因说明]
    pub fn format_output(&self) -> String {
        let (category, position, reason) = match self {
            SemanticError::TableNotFound { table, position } => {
                (1, *position, format!("Table '{}' not found", table))
            }
            SemanticError::ColumnNotFound {
                table,
                column,
                position,
            } => (
                1,
                *position,
                format!("Column '{}' not found in table '{}'", column, table),
            ),
            SemanticError::AmbiguousColumn { column, position } => (
                1,
                *position,
                format!("Ambiguous column reference: '{}'", column),
            ),
            SemanticError::TypeMismatch {
                expected,
                found,
                position,
            } => (
                2,
                *position,
                format!("Type mismatch: expected {:?}, found {:?}", expected, found),
            ),
            SemanticError::InvalidBinaryOperation {
                op,
                left,
                right,
                position,
            } => (
                2,
                *position,
                format!(
                    "Invalid operation {:?} on types {:?} and {:?}",
                    op, left, right
                ),
            ),
            SemanticError::InvalidUnaryOperation {
                op,
                operand,
                position,
            } => (
                2,
                *position,
                format!("Invalid unary operation {:?} on type {:?}", op, operand),
            ),
            SemanticError::DuplicateColumn { column, position } => {
                (1, *position, format!("Duplicate column name: '{}'", column))
            }
            SemanticError::TableAlreadyExists { table, position } => {
                (1, *position, format!("Table '{}' already exists", table))
            }
            SemanticError::InsertColumnMismatch {
                expected,
                actual,
                position,
            } => (
                3,
                *position,
                format!(
                    "Insert column count mismatch: expected {}, got {}",
                    expected, actual
                ),
            ),
            SemanticError::NullConstraintViolation { column, position } => {
                (3, *position, format!("Column '{}' cannot be null", column))
            }
        };

        let pos_str = if let Some((line, col)) = position {
            format!("{}:{}", line, col)
        } else {
            "unknown".to_string()
        };

        format!("[{}, {}, {}]", category, pos_str, reason)
    }
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(catalog: &'a dyn SchemaCatalog) -> Self {
        Self { catalog }
    }

    /// 分析 SQL 语句
    pub fn analyze(&self, stmt: Statement) -> Result<AnalyzedStatement, SemanticError> {
        let mut table_schemas = HashMap::new();
        let mut expression_types = HashMap::new();

        match &stmt {
            Statement::CreateTable {
                table_name,
                columns,
                ..
            } => {
                self.analyze_create_table(table_name, columns)?;
            }
            Statement::DropTable { table_name, .. } => {
                self.analyze_drop_table(table_name)?;
            }
            Statement::Select {
                from_clause,
                where_clause,
                select_list,
                ..
            } => {
                self.analyze_select(
                    from_clause,
                    where_clause,
                    select_list,
                    &mut table_schemas,
                    &mut expression_types,
                )?;
            }
            Statement::Insert {
                table_name,
                columns,
                values,
            } => {
                self.analyze_insert(
                    table_name,
                    columns,
                    values,
                    &mut table_schemas,
                    &mut expression_types,
                )?;
            }
            Statement::Update {
                table_name,
                assignments,
                where_clause,
            } => {
                self.analyze_update(
                    table_name,
                    assignments,
                    where_clause,
                    &mut table_schemas,
                    &mut expression_types,
                )?;
            }
            Statement::Delete {
                table_name,
                where_clause,
            } => {
                self.analyze_delete(
                    table_name,
                    where_clause,
                    &mut table_schemas,
                    &mut expression_types,
                )?;
            }
            Statement::CreateIndex { table_name, .. } => {
                // 验证表是否存在
                if !self.catalog.table_exists(table_name) {
                    return Err(SemanticError::TableNotFound {
                        table: table_name.clone(),
                        position: None,
                    });
                }
            }
            Statement::DropIndex { .. } => {
                // 索引删除的语义分析（暂时简单处理）
            }
            Statement::Explain { .. } => {
                // EXPLAIN语句不需要特殊的语义分析
            }
        }

        Ok(AnalyzedStatement {
            statement: stmt,
            table_schemas,
            expression_types,
        })
    }

    /// 分析 CREATE TABLE 语句
    fn analyze_create_table(
        &self,
        table_name: &str,
        columns: &[crate::sql::parser::ColumnDef],
    ) -> Result<(), SemanticError> {
        // Check if table already exists
        if self.catalog.table_exists(table_name) {
            return Err(SemanticError::TableAlreadyExists {
                table: table_name.to_string(),
                position: None,
            });
        }

        // Check for duplicate column names
        let mut column_names = std::collections::HashSet::new();
        for column in columns {
            if !column_names.insert(&column.name) {
                return Err(SemanticError::DuplicateColumn {
                    column: column.name.clone(),
                    position: None,
                });
            }
        }

        Ok(())
    }

    /// 分析 DROP TABLE 语句
    fn analyze_drop_table(&self, table_name: &str) -> Result<(), SemanticError> {
        if !self.catalog.table_exists(table_name) {
            return Err(SemanticError::TableNotFound {
                table: table_name.to_string(),
                position: None,
            });
        }

        Ok(())
    }

    /// 分析 SELECT 语句
    fn analyze_select(
        &self,
        from_clause: &Option<crate::sql::parser::FromClause>,
        where_clause: &Option<Expression>,
        _select_list: &crate::sql::parser::SelectList,
        table_schemas: &mut HashMap<String, Schema>,
        expression_types: &mut HashMap<String, DataType>,
    ) -> Result<(), SemanticError> {
        // Analyze FROM clause
        if let Some(from) = from_clause {
            self.analyze_from_clause(from, table_schemas)?;
        }

        // Analyze WHERE clause
        if let Some(where_expr) = where_clause {
            let expr_type = self.analyze_expression(where_expr, table_schemas, expression_types)?;

            // WHERE clause must be boolean
            if expr_type != DataType::Boolean {
                return Err(SemanticError::TypeMismatch {
                    expected: DataType::Boolean,
                    found: expr_type,
                    position: None,
                });
            }
        }

        Ok(())
    }

    /// 分析 FROM 子句
    fn analyze_from_clause(
        &self,
        from_clause: &crate::sql::parser::FromClause,
        table_schemas: &mut HashMap<String, Schema>,
    ) -> Result<(), SemanticError> {
        match from_clause {
            crate::sql::parser::FromClause::Table(table_name) => {
                let schema = self.catalog.get_table_schema(table_name).ok_or_else(|| {
                    SemanticError::TableNotFound {
                        table: table_name.clone(),
                        position: None,
                    }
                })?;
                table_schemas.insert(table_name.clone(), schema);
            }
            crate::sql::parser::FromClause::Join { left, right, .. } => {
                self.analyze_from_clause(left, table_schemas)?;
                self.analyze_from_clause(right, table_schemas)?;
            }
        }

        Ok(())
    }

    /// 分析 INSERT 语句
    fn analyze_insert(
        &self,
        table_name: &str,
        columns: &Option<Vec<String>>,
        values: &[Vec<Expression>],
        table_schemas: &mut HashMap<String, Schema>,
        expression_types: &mut HashMap<String, DataType>,
    ) -> Result<(), SemanticError> {
        // Get table schema
        let schema = self.catalog.get_table_schema(table_name).ok_or_else(|| {
            SemanticError::TableNotFound {
                table: table_name.to_string(),
                position: None,
            }
        })?;

        table_schemas.insert(table_name.to_string(), schema.clone());

        // Determine target columns
        let target_columns: Vec<&ColumnDefinition> = if let Some(column_names) = columns {
            // Explicit column list
            let mut target_cols = Vec::new();
            for col_name in column_names {
                let col_def = schema
                    .columns
                    .iter()
                    .find(|c| c.name == *col_name)
                    .ok_or_else(|| SemanticError::ColumnNotFound {
                        table: table_name.to_string(),
                        column: col_name.clone(),
                        position: None,
                    })?;
                target_cols.push(col_def);
            }
            target_cols
        } else {
            // All columns in order
            schema.columns.iter().collect()
        };

        // Analyze each value row
        for value_row in values {
            if value_row.len() != target_columns.len() {
                return Err(SemanticError::InsertColumnMismatch {
                    expected: target_columns.len(),
                    actual: value_row.len(),
                    position: None,
                });
            }

            // Check type compatibility for each value
            for (value_expr, target_column) in value_row.iter().zip(target_columns.iter()) {
                let value_type =
                    self.analyze_expression(value_expr, table_schemas, expression_types)?;

                // Check if value type is compatible with column type
                if !value_type.is_compatible_with(&target_column.data_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: target_column.data_type.clone(),
                        found: value_type,
                        position: None,
                    });
                }

                // Check null constraint
                if matches!(value_expr, Expression::Literal(Value::Null)) && !target_column.nullable
                {
                    return Err(SemanticError::NullConstraintViolation {
                        column: target_column.name.clone(),
                        position: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// 分析 UPDATE 语句
    fn analyze_update(
        &self,
        table_name: &str,
        assignments: &[crate::sql::parser::Assignment],
        where_clause: &Option<Expression>,
        table_schemas: &mut HashMap<String, Schema>,
        expression_types: &mut HashMap<String, DataType>,
    ) -> Result<(), SemanticError> {
        // Get table schema
        let schema = self.catalog.get_table_schema(table_name).ok_or_else(|| {
            SemanticError::TableNotFound {
                table: table_name.to_string(),
                position: None,
            }
        })?;

        table_schemas.insert(table_name.to_string(), schema.clone());

        // Analyze assignments
        for assignment in assignments {
            // Check if column exists
            let column_def = schema
                .columns
                .iter()
                .find(|c| c.name == assignment.column)
                .ok_or_else(|| SemanticError::ColumnNotFound {
                    table: table_name.to_string(),
                    column: assignment.column.clone(),
                    position: None,
                })?;

            // Analyze assignment value
            let value_type =
                self.analyze_expression(&assignment.value, table_schemas, expression_types)?;

            // Check type compatibility
            if !value_type.is_compatible_with(&column_def.data_type) {
                return Err(SemanticError::TypeMismatch {
                    expected: column_def.data_type.clone(),
                    found: value_type,
                    position: None,
                });
            }
        }

        // Analyze WHERE clause
        if let Some(where_expr) = where_clause {
            let expr_type = self.analyze_expression(where_expr, table_schemas, expression_types)?;

            if expr_type != DataType::Boolean {
                return Err(SemanticError::TypeMismatch {
                    expected: DataType::Boolean,
                    found: expr_type,
                    position: None,
                });
            }
        }

        Ok(())
    }

    /// 分析 DELETE 语句
    fn analyze_delete(
        &self,
        table_name: &str,
        where_clause: &Option<Expression>,
        table_schemas: &mut HashMap<String, Schema>,
        expression_types: &mut HashMap<String, DataType>,
    ) -> Result<(), SemanticError> {
        // Get table schema
        let schema = self.catalog.get_table_schema(table_name).ok_or_else(|| {
            SemanticError::TableNotFound {
                table: table_name.to_string(),
                position: None,
            }
        })?;

        table_schemas.insert(table_name.to_string(), schema);

        // Analyze WHERE clause
        if let Some(where_expr) = where_clause {
            let expr_type = self.analyze_expression(where_expr, table_schemas, expression_types)?;

            if expr_type != DataType::Boolean {
                return Err(SemanticError::TypeMismatch {
                    expected: DataType::Boolean,
                    found: expr_type,
                    position: None,
                });
            }
        }

        Ok(())
    }

    /// 分析表达式并返回其类型
    fn analyze_expression(
        &self,
        expr: &Expression,
        table_schemas: &HashMap<String, Schema>,
        expression_types: &mut HashMap<String, DataType>,
    ) -> Result<DataType, SemanticError> {
        let expr_type = match expr {
            Expression::Literal(value) => value.data_type(),

            Expression::Column(column_name) => {
                self.resolve_column_type(column_name, table_schemas)?
            }

            Expression::QualifiedColumn { table, column } => {
                let schema =
                    table_schemas
                        .get(table)
                        .ok_or_else(|| SemanticError::TableNotFound {
                            table: table.clone(),
                            position: None,
                        })?;

                let column_def = schema
                    .columns
                    .iter()
                    .find(|c| c.name == *column)
                    .ok_or_else(|| SemanticError::ColumnNotFound {
                        table: table.clone(),
                        column: column.clone(),
                        position: None,
                    })?;

                column_def.data_type.clone()
            }

            Expression::BinaryOp { left, op, right } => {
                let left_type = self.analyze_expression(left, table_schemas, expression_types)?;
                let right_type = self.analyze_expression(right, table_schemas, expression_types)?;

                self.analyze_binary_operation(op, &left_type, &right_type)?
            }

            Expression::UnaryOp { op, expr: operand } => {
                let operand_type =
                    self.analyze_expression(operand, table_schemas, expression_types)?;

                self.analyze_unary_operation(op, &operand_type)?
            }

            Expression::FunctionCall { .. } => {
                // For now, assume function calls return VARCHAR
                // TODO: Implement proper function signature checking
                DataType::Varchar(255)
            }

            Expression::In {
                expr: operand,
                list,
            } => {
                let operand_type =
                    self.analyze_expression(operand, table_schemas, expression_types)?;

                // Check that all list items are compatible with operand type
                for item in list {
                    let item_type =
                        self.analyze_expression(item, table_schemas, expression_types)?;
                    if !item_type.is_compatible_with(&operand_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: operand_type,
                            found: item_type,
                            position: None,
                        });
                    }
                }

                DataType::Boolean
            }

            Expression::Between {
                expr: operand,
                low,
                high,
            } => {
                let operand_type =
                    self.analyze_expression(operand, table_schemas, expression_types)?;
                let low_type = self.analyze_expression(low, table_schemas, expression_types)?;
                let high_type = self.analyze_expression(high, table_schemas, expression_types)?;

                if !low_type.is_compatible_with(&operand_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: operand_type,
                        found: low_type,
                        position: None,
                    });
                }
                if !high_type.is_compatible_with(&operand_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: operand_type,
                        found: high_type,
                        position: None,
                    });
                }

                DataType::Boolean
            }

            Expression::Like { .. } => DataType::Boolean,
            Expression::IsNull(_) => DataType::Boolean,
            Expression::IsNotNull(_) => DataType::Boolean,
        };

        // Store expression type for later use
        let expr_key = format!("{:?}", expr);
        expression_types.insert(expr_key, expr_type.clone());

        Ok(expr_type)
    }

    /// 从可用模式中解析列类型
    fn resolve_column_type(
        &self,
        column_name: &str,
        table_schemas: &HashMap<String, Schema>,
    ) -> Result<DataType, SemanticError> {
        let mut matches = Vec::new();

        for (table_name, schema) in table_schemas {
            for column in &schema.columns {
                if column.name == column_name {
                    matches.push((table_name.clone(), column.data_type.clone()));
                }
            }
        }

        match matches.len() {
            0 => Err(SemanticError::ColumnNotFound {
                table: "unknown".to_string(),
                column: column_name.to_string(),
                position: None,
            }),
            1 => Ok(matches[0].1.clone()),
            _ => Err(SemanticError::AmbiguousColumn {
                column: column_name.to_string(),
                position: None,
            }),
        }
    }

    /// 分析二元操作并返回结果类型
    fn analyze_binary_operation(
        &self,
        op: &BinaryOperator,
        left_type: &DataType,
        right_type: &DataType,
    ) -> Result<DataType, SemanticError> {
        use BinaryOperator::*;

        match op {
            // Arithmetic operations
            Add | Subtract | Multiply | Divide | Modulo => {
                if self.is_numeric_type(left_type) && self.is_numeric_type(right_type) {
                    // Return the "wider" type
                    if matches!(left_type, DataType::Double)
                        || matches!(right_type, DataType::Double)
                    {
                        Ok(DataType::Double)
                    } else if matches!(left_type, DataType::Float)
                        || matches!(right_type, DataType::Float)
                    {
                        Ok(DataType::Float)
                    } else if matches!(left_type, DataType::BigInt)
                        || matches!(right_type, DataType::BigInt)
                    {
                        Ok(DataType::BigInt)
                    } else {
                        Ok(DataType::Integer)
                    }
                } else {
                    Err(SemanticError::InvalidBinaryOperation {
                        op: op.clone(),
                        left: left_type.clone(),
                        right: right_type.clone(),
                        position: None,
                    })
                }
            }

            // Comparison operations
            Equal | NotEqual | LessThan | LessEqual | GreaterThan | GreaterEqual => {
                if left_type.is_compatible_with(right_type)
                    || right_type.is_compatible_with(left_type)
                {
                    Ok(DataType::Boolean)
                } else {
                    Err(SemanticError::InvalidBinaryOperation {
                        op: op.clone(),
                        left: left_type.clone(),
                        right: right_type.clone(),
                        position: None,
                    })
                }
            }

            // Logical operations
            And | Or => {
                if *left_type == DataType::Boolean && *right_type == DataType::Boolean {
                    Ok(DataType::Boolean)
                } else {
                    Err(SemanticError::InvalidBinaryOperation {
                        op: op.clone(),
                        left: left_type.clone(),
                        right: right_type.clone(),
                        position: None,
                    })
                }
            }
        }
    }

    /// 分析一元操作并返回结果类型
    fn analyze_unary_operation(
        &self,
        op: &UnaryOperator,
        operand_type: &DataType,
    ) -> Result<DataType, SemanticError> {
        use UnaryOperator::*;

        match op {
            Not => {
                if *operand_type == DataType::Boolean {
                    Ok(DataType::Boolean)
                } else {
                    Err(SemanticError::InvalidUnaryOperation {
                        op: op.clone(),
                        operand: operand_type.clone(),
                        position: None,
                    })
                }
            }

            Minus | Plus => {
                if self.is_numeric_type(operand_type) {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidUnaryOperation {
                        op: op.clone(),
                        operand: operand_type.clone(),
                        position: None,
                    })
                }
            }
        }
    }

    /// 检查类型是否为数值类型
    fn is_numeric_type(&self, data_type: &DataType) -> bool {
        matches!(
            data_type,
            DataType::Integer | DataType::BigInt | DataType::Float | DataType::Double
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parse_sql;
    use crate::sql::parser::{BinaryOperator, Expression, Statement};
    use crate::types::{ColumnDefinition, DataType, Schema, Value};

    fn create_test_catalog() -> MemoryCatalog {
        let mut catalog = MemoryCatalog::new();

        // Create users table schema
        let users_schema = Schema {
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Varchar(255),
                    nullable: false,
                    default: None,
                },
                ColumnDefinition {
                    name: "age".to_string(),
                    data_type: DataType::Integer,
                    nullable: true,
                    default: None,
                },
                ColumnDefinition {
                    name: "email".to_string(),
                    data_type: DataType::Varchar(255),
                    nullable: true,
                    default: None,
                },
            ],
            primary_key: Some(vec![0]), // id column is primary key
        };

        catalog.add_table("users".to_string(), users_schema);
        catalog
    }

    #[test]
    fn test_analyze_create_table() {
        let catalog = MemoryCatalog::new();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt =
            parse_sql("CREATE TABLE test (id INT PRIMARY KEY, name VARCHAR NOT NULL)").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_duplicate_table() {
        let mut catalog = MemoryCatalog::new();
        catalog.add_table("test".to_string(), Schema { columns: vec![], primary_key: None });

        let analyzer = SemanticAnalyzer::new(&catalog);
        let stmt = parse_sql("CREATE TABLE test (id INT)").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(
            result,
            Err(SemanticError::TableAlreadyExists { .. })
        ));
    }

    #[test]
    fn test_analyze_select_valid() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("SELECT * FROM users WHERE age > 18").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_select_invalid_table() {
        let catalog = MemoryCatalog::new();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("SELECT * FROM nonexistent").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(result, Err(SemanticError::TableNotFound { .. })));
    }

    #[test]
    fn test_analyze_select_invalid_column() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("SELECT * FROM users WHERE nonexistent > 18").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(result, Err(SemanticError::ColumnNotFound { .. })));
    }

    #[test]
    fn test_analyze_select_type_mismatch() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        // WHERE clause should be boolean, but age > 'string' is invalid
        let stmt = parse_sql("SELECT * FROM users WHERE age > 'not_a_number'").unwrap();
        let result = analyzer.analyze(stmt);

        // This should fail during expression type analysis
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_insert_valid() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("INSERT INTO users (name, age) VALUES ('Alice', 25)").unwrap();
        let result = analyzer.analyze(stmt);

        match result {
            Ok(_) => {}
            Err(e) => panic!("Expected success, got error: {:?}", e),
        }
    }

    #[test]
    fn test_analyze_insert_column_mismatch() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        // Providing 3 values for 2 columns
        let stmt =
            parse_sql("INSERT INTO users (name, age) VALUES ('Alice', 25, 'extra')").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(
            result,
            Err(SemanticError::InsertColumnMismatch { .. })
        ));
    }

    #[test]
    fn test_analyze_insert_invalid_column() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("INSERT INTO users (nonexistent) VALUES ('Alice')").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(result, Err(SemanticError::ColumnNotFound { .. })));
    }

    #[test]
    fn test_analyze_update_valid() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("UPDATE users SET age = 26 WHERE name = 'Alice'").unwrap();
        let result = analyzer.analyze(stmt);

        match result {
            Ok(_) => {}
            Err(e) => panic!("Expected success, got error: {:?}", e),
        }
    }

    #[test]
    fn test_analyze_update_invalid_column() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("UPDATE users SET nonexistent = 26").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(matches!(result, Err(SemanticError::ColumnNotFound { .. })));
    }

    #[test]
    fn test_analyze_delete_valid() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("DELETE FROM users WHERE age < 18").unwrap();
        let result = analyzer.analyze(stmt);

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_binary_operations() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        // Arithmetic operation (should work with integers)
        let stmt = parse_sql("SELECT * FROM users WHERE age + 1 > 18").unwrap();
        let result = analyzer.analyze(stmt);
        match result {
            Ok(_) => {}
            Err(e) => panic!(
                "Expected success for arithmetic operation, got error: {:?}",
                e
            ),
        }

        // String comparison (should work)
        let stmt = parse_sql("SELECT * FROM users WHERE name = 'Alice'").unwrap();
        let result = analyzer.analyze(stmt);
        match result {
            Ok(_) => {}
            Err(e) => panic!("Expected success for string comparison, got error: {:?}", e),
        }

        // Boolean operations (should work)
        let stmt = parse_sql("SELECT * FROM users WHERE age > 18 AND name = 'Alice'").unwrap();
        let result = analyzer.analyze(stmt);
        match result {
            Ok(_) => {}
            Err(e) => panic!("Expected success for boolean operation, got error: {:?}", e),
        }
    }

    #[test]
    fn test_analyze_expression_types() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);

        let stmt = parse_sql("SELECT * FROM users WHERE age > 18").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();

        // Should have stored expression types
        assert!(!analyzed.expression_types.is_empty());
        assert_eq!(analyzed.table_schemas.len(), 1);
        assert!(analyzed.table_schemas.contains_key("users"));
    }
}
