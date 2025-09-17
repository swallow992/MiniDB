//! 查询规划器
//!
//! 将已分析的 SQL 语句转换为可执行的查询计划。
//! 规划器执行查询优化并生成可由查询执行器执行的操作符树。

use crate::engine::executor::AggregateFunction;
use crate::sql::analyzer::AnalyzedStatement;
use crate::sql::parser::{Expression, FromClause, SelectList, Statement};
use crate::types::{DataType, Schema};
use std::collections::HashMap;
use thiserror::Error;

/// 表示操作符树的执行计划
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPlan {
    /// 顺序扫描表
    TableScan {
        table_name: String,
        schema: Schema,
        filter: Option<Expression>,
    },

    /// 使用索引扫描表
    IndexScan {
        table_name: String,
        index_name: String,
        condition: Option<Expression>,
    },

    /// 投影特定列
    Project {
        input: Box<ExecutionPlan>,
        columns: Vec<ProjectColumn>,
    },

    /// 基于条件过滤行
    Filter {
        input: Box<ExecutionPlan>,
        condition: Expression,
    },

    /// 向表中插入数据
    Insert {
        table_name: String,
        schema: Schema,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expression>>,
    },

    /// 更新表中的行
    Update {
        table_name: String,
        schema: Schema,
        assignments: Vec<UpdateAssignment>,
        filter: Option<Expression>,
    },

    /// 从表中删除行
    Delete {
        table_name: String,
        schema: Schema,
        filter: Option<Expression>,
    },

    /// 创建新表
    CreateTable { table_name: String, schema: Schema },

    /// 删除表
    DropTable { table_name: String, if_exists: bool },

    /// 连接两个输入
    Join {
        left: Box<ExecutionPlan>,
        right: Box<ExecutionPlan>,
        join_type: JoinType,
        condition: Option<Expression>,
    },

    /// 排序输入
    Sort {
        input: Box<ExecutionPlan>,
        sort_keys: Vec<SortKey>,
    },

    /// 限制输出行数
    Limit {
        input: Box<ExecutionPlan>,
        count: u64,
        offset: Option<u64>,
    },

    /// 分组和聚合
    GroupBy {
        input: Box<ExecutionPlan>,
        group_expressions: Vec<Expression>,
        aggregate_functions: Vec<AggregateFunction>,
    },

    /// 创建索引
    CreateIndex {
        index_name: String,
        table_name: String,
        columns: Vec<String>,
        is_unique: bool,
    },

    /// 删除索引
    DropIndex {
        index_name: String,
        table_name: String,
        if_exists: bool,
    },

    /// 解释查询计划
    Explain {
        statement: Box<Statement>,
    },
}

/// 列投影规格
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub data_type: DataType,
}

/// UPDATE 计划中的更新赋值
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAssignment {
    pub column: String,
    pub expression: Expression,
}

/// 连接类型
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// 排序键规格
#[derive(Debug, Clone, PartialEq)]
pub struct SortKey {
    pub expression: Expression,
    pub descending: bool,
}

/// 查询规划器
pub struct QueryPlanner {
    // 未来：这里可以添加基于成本的优化状态
}

/// 规划错误
#[derive(Error, Debug)]
pub enum PlanError {
    #[error("未找到表模式: {table}")]
    SchemaNotFound { table: String },

    #[error("投影中未找到列: {column}")]
    ProjectionColumnNotFound { column: String },

    #[error("不支持的操作: {operation}")]
    UnsupportedOperation { operation: String },

    #[error("规划错误: {message}")]
    PlanningError { message: String },
}

impl QueryPlanner {
    /// 创建新的查询规划器
    pub fn new() -> Self {
        Self {}
    }

    /// 从已分析的语句创建执行计划
    pub fn create_plan(&self, analyzed: AnalyzedStatement) -> Result<ExecutionPlan, PlanError> {
        match analyzed.statement {
            Statement::CreateTable {
                table_name,
                columns,
                ..
            } => {
                let schema = self.build_schema_from_columns(&columns)?;
                Ok(ExecutionPlan::CreateTable { table_name, schema })
            }

            Statement::DropTable {
                table_name,
                if_exists,
            } => Ok(ExecutionPlan::DropTable {
                table_name,
                if_exists,
            }),

            Statement::Select {
                select_list,
                from_clause,
                where_clause,
                group_by,
                having,
                order_by,
                limit,
                offset,
            } => self.plan_select_complete(
                select_list,
                from_clause,
                where_clause,
                group_by,
                having,
                order_by,
                limit,
                offset,
                &analyzed.table_schemas,
                &analyzed.expression_types,
            ),

            Statement::Insert {
                table_name,
                columns,
                values,
            } => {
                let schema = analyzed.table_schemas.get(&table_name).ok_or_else(|| {
                    PlanError::SchemaNotFound {
                        table: table_name.clone(),
                    }
                })?;

                Ok(ExecutionPlan::Insert {
                    table_name,
                    schema: schema.clone(),
                    columns,
                    values,
                })
            }

            Statement::Update {
                table_name,
                assignments,
                where_clause,
            } => {
                let schema = analyzed.table_schemas.get(&table_name).ok_or_else(|| {
                    PlanError::SchemaNotFound {
                        table: table_name.clone(),
                    }
                })?;

                let plan_assignments = assignments
                    .into_iter()
                    .map(|a| UpdateAssignment {
                        column: a.column,
                        expression: a.value,
                    })
                    .collect();

                Ok(ExecutionPlan::Update {
                    table_name,
                    schema: schema.clone(),
                    assignments: plan_assignments,
                    filter: where_clause,
                })
            }

            Statement::Delete {
                table_name,
                where_clause,
            } => {
                let schema = analyzed.table_schemas.get(&table_name).ok_or_else(|| {
                    PlanError::SchemaNotFound {
                        table: table_name.clone(),
                    }
                })?;

                Ok(ExecutionPlan::Delete {
                    table_name,
                    schema: schema.clone(),
                    filter: where_clause,
                })
            }

            Statement::CreateIndex {
                index_name,
                table_name,
                columns,
                is_unique,
            } => Ok(ExecutionPlan::CreateIndex {
                index_name,
                table_name,
                columns,
                is_unique,
            }),

            Statement::DropIndex {
                index_name,
                table_name,
                if_exists,
            } => Ok(ExecutionPlan::DropIndex {
                index_name,
                table_name,
                if_exists,
            }),

            Statement::Explain { statement } => Ok(ExecutionPlan::Explain {
                statement: Box::new(*statement),
            }),
        }
    }

    /// 规划完整的 SELECT 语句及其所有子句
    fn plan_select_complete(
        &self,
        select_list: SelectList,
        from_clause: Option<FromClause>,
        where_clause: Option<Expression>,
        group_by: Option<Vec<Expression>>,
        having: Option<Expression>,
        order_by: Option<Vec<crate::sql::parser::OrderByExpr>>,
        limit: Option<u64>,
        offset: Option<u64>,
        table_schemas: &HashMap<String, Schema>,
        expression_types: &HashMap<String, DataType>,
    ) -> Result<ExecutionPlan, PlanError> {
        // Start with the FROM clause
        let mut plan = if let Some(from) = from_clause {
            self.plan_from_clause(from, table_schemas)?
        } else {
            // SELECT without FROM - not commonly supported, but we could add a dummy scan
            return Err(PlanError::UnsupportedOperation {
                operation: "SELECT without FROM clause".to_string(),
            });
        };

        // Add WHERE filter if present
        if let Some(condition) = where_clause {
            plan = ExecutionPlan::Filter {
                input: Box::new(plan),
                condition,
            };
        }

        // Add GROUP BY if present, or if SELECT list contains aggregate functions
        if let Some(group_exprs) = group_by {
            // Explicit GROUP BY clause
            plan = self.plan_group_by(plan, group_exprs, having)?;
        } else if self.contains_aggregate_functions(&select_list) {
            // No GROUP BY but SELECT contains aggregate functions - create implicit GROUP BY
            let aggregate_functions = self.extract_aggregate_functions(&select_list);
            plan = ExecutionPlan::GroupBy {
                input: Box::new(plan),
                group_expressions: Vec::new(), // Empty group - aggregate over all rows
                aggregate_functions,
            };
        }

        // Add projection
        plan = self.plan_select_list(plan, select_list, table_schemas, expression_types)?;

        // Add ORDER BY if present
        if let Some(order_exprs) = order_by {
            let sort_keys = order_exprs
                .into_iter()
                .map(|order_expr| SortKey {
                    expression: order_expr.expr,
                    descending: order_expr.desc,
                })
                .collect();

            plan = ExecutionPlan::Sort {
                input: Box::new(plan),
                sort_keys,
            };
        }

        // Add LIMIT/OFFSET if present
        if let Some(limit_count) = limit {
            plan = ExecutionPlan::Limit {
                input: Box::new(plan),
                count: limit_count,
                offset,
            };
        }

        Ok(plan)
    }

    /// 规划 GROUP BY 子句
    fn plan_group_by(
        &self,
        input: ExecutionPlan,
        group_exprs: Vec<Expression>,
        _having: Option<Expression>,
    ) -> Result<ExecutionPlan, PlanError> {
        // For now, create basic aggregate functions
        // TODO: Parse actual aggregate functions from SELECT list
        let aggregate_functions = vec![
            AggregateFunction::Count,
        ];

        Ok(ExecutionPlan::GroupBy {
            input: Box::new(input),
            group_expressions: group_exprs,
            aggregate_functions,
        })
    }

    /// 检查 SELECT 列表是否包含聚合函数
    fn contains_aggregate_functions(&self, select_list: &SelectList) -> bool {
        match select_list {
            SelectList::Wildcard => false,
            SelectList::Expressions(expressions) => {
                expressions.iter().any(|select_expr| {
                    self.expression_contains_aggregate(&select_expr.expr)
                })
            }
        }
    }

    /// 检查表达式是否包含聚合函数（递归检查）
    fn expression_contains_aggregate(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall { name, .. } => {
                // Check if this is an aggregate function
                matches!(name.to_uppercase().as_str(), "COUNT" | "SUM" | "AVG" | "MIN" | "MAX")
            }
            // For other expression types, we can add recursive checks if needed
            _ => false
        }
    }

    /// 从 SELECT 列表中提取聚合函数
    fn extract_aggregate_functions(&self, select_list: &SelectList) -> Vec<AggregateFunction> {
        let mut functions = Vec::new();
        
        match select_list {
            SelectList::Wildcard => {},
            SelectList::Expressions(expressions) => {
                for select_expr in expressions {
                    if let Expression::FunctionCall { name, args } = &select_expr.expr {
                        match name.to_uppercase().as_str() {
                            "COUNT" => functions.push(AggregateFunction::Count),
                            "SUM" => {
                                if let Some(Expression::Column(col)) = args.get(0) {
                                    functions.push(AggregateFunction::Sum(col.clone()));
                                }
                            }
                            "AVG" => {
                                if let Some(Expression::Column(col)) = args.get(0) {
                                    functions.push(AggregateFunction::Avg(col.clone()));
                                }
                            }
                            "MIN" => {
                                if let Some(Expression::Column(col)) = args.get(0) {
                                    functions.push(AggregateFunction::Min(col.clone()));
                                }
                            }
                            "MAX" => {
                                if let Some(Expression::Column(col)) = args.get(0) {
                                    functions.push(AggregateFunction::Max(col.clone()));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        functions
    }

    /// 规划 SELECT 语句（为兼容性保留的旧方法）
    fn plan_select(
        &self,
        select_list: SelectList,
        from_clause: Option<FromClause>,
        where_clause: Option<Expression>,
        table_schemas: &HashMap<String, Schema>,
        expression_types: &HashMap<String, DataType>,
    ) -> Result<ExecutionPlan, PlanError> {
        self.plan_select_complete(
            select_list,
            from_clause,
            where_clause,
            None,    // group_by
            None,    // having
            None,    // order_by
            None,    // limit
            None,    // offset
            table_schemas,
            expression_types,
        )
    }

    /// 规划 FROM 子句
    fn plan_from_clause(
        &self,
        from_clause: FromClause,
        table_schemas: &HashMap<String, Schema>,
    ) -> Result<ExecutionPlan, PlanError> {
        match from_clause {
            FromClause::Table(table_name) => {
                let schema =
                    table_schemas
                        .get(&table_name)
                        .ok_or_else(|| PlanError::SchemaNotFound {
                            table: table_name.clone(),
                        })?;

                Ok(ExecutionPlan::TableScan {
                    table_name,
                    schema: schema.clone(),
                    filter: None,
                })
            }

            FromClause::Join {
                left,
                join_type,
                right,
                condition,
            } => {
                let left_plan = self.plan_from_clause(*left, table_schemas)?;
                let right_plan = self.plan_from_clause(*right, table_schemas)?;

                let plan_join_type = match join_type {
                    crate::sql::parser::JoinType::Inner => JoinType::Inner,
                    crate::sql::parser::JoinType::Left => JoinType::Left,
                    crate::sql::parser::JoinType::Right => JoinType::Right,
                    crate::sql::parser::JoinType::Full => JoinType::Full,
                };

                Ok(ExecutionPlan::Join {
                    left: Box::new(left_plan),
                    right: Box::new(right_plan),
                    join_type: plan_join_type,
                    condition,
                })
            }
        }
    }

    /// 规划 SELECT 列表（投影）
    fn plan_select_list(
        &self,
        input: ExecutionPlan,
        select_list: SelectList,
        table_schemas: &HashMap<String, Schema>,
        expression_types: &HashMap<String, DataType>,
    ) -> Result<ExecutionPlan, PlanError> {
        match select_list {
            SelectList::Wildcard => {
                // SELECT * - include all columns from all tables
                let columns = self.build_wildcard_projection(table_schemas)?;

                Ok(ExecutionPlan::Project {
                    input: Box::new(input),
                    columns,
                })
            }

            SelectList::Expressions(expressions) => {
                let mut columns = Vec::new();

                for select_expr in expressions {
                    let data_type =
                        self.infer_expression_type(&select_expr.expr, expression_types)?;

                    columns.push(ProjectColumn {
                        expression: select_expr.expr,
                        alias: select_expr.alias,
                        data_type,
                    });
                }

                Ok(ExecutionPlan::Project {
                    input: Box::new(input),
                    columns,
                })
            }
        }
    }

    /// 从列定义构建模式
    fn build_schema_from_columns(
        &self,
        columns: &[crate::sql::parser::ColumnDef],
    ) -> Result<Schema, PlanError> {
        let column_defs = columns
            .iter()
            .map(|col| crate::types::ColumnDefinition {
                name: col.name.clone(),
                data_type: col.data_type.clone(),
                nullable: col.nullable,
                default: None, // TODO: Evaluate default expressions to values
            })
            .collect();

        // Extract primary key column indices
        let mut primary_key_columns = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            if col.primary_key {
                primary_key_columns.push(i);
            }
        }

        let primary_key = if primary_key_columns.is_empty() {
            None
        } else {
            Some(primary_key_columns)
        };

        Ok(Schema {
            columns: column_defs,
            primary_key,
        })
    }

    /// 构建通配符投影（SELECT *）
    fn build_wildcard_projection(
        &self,
        table_schemas: &HashMap<String, Schema>,
    ) -> Result<Vec<ProjectColumn>, PlanError> {
        let mut columns = Vec::new();

        for (table_name, schema) in table_schemas {
            for column_def in &schema.columns {
                columns.push(ProjectColumn {
                    expression: Expression::QualifiedColumn {
                        table: table_name.clone(),
                        column: column_def.name.clone(),
                    },
                    alias: None,
                    data_type: column_def.data_type.clone(),
                });
            }
        }

        Ok(columns)
    }

    /// 推断表达式的数据类型
    fn infer_expression_type(
        &self,
        expression: &Expression,
        expression_types: &HashMap<String, DataType>,
    ) -> Result<DataType, PlanError> {
        // Look up type from analyzer
        let expr_key = format!("{:?}", expression);
        expression_types
            .get(&expr_key)
            .cloned()
            .or_else(|| {
                // Fallback: basic type inference
                match expression {
                    Expression::Literal(value) => Some(value.data_type()),
                    Expression::Column(_) => Some(DataType::Varchar(255)), // Default assumption
                    Expression::QualifiedColumn { .. } => Some(DataType::Varchar(255)), // Default assumption
                    _ => None,
                }
            })
            .ok_or_else(|| PlanError::PlanningError {
                message: format!("Could not infer type for expression: {:?}", expression),
            })
    }
}

impl Default for QueryPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::analyzer::{MemoryCatalog, SemanticAnalyzer};
    use crate::sql::parse_sql;
    use crate::types::{ColumnDefinition, DataType, Schema};

    fn create_test_catalog() -> MemoryCatalog {
        let mut catalog = MemoryCatalog::new();

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
            ],
            primary_key: None, // Test schema without primary key
        };

        catalog.add_table("users".to_string(), users_schema);
        catalog
    }

    #[test]
    fn test_plan_create_table() {
        let catalog = MemoryCatalog::new();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt =
            parse_sql("CREATE TABLE test (id INT PRIMARY KEY, name VARCHAR NOT NULL)").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::CreateTable { table_name, schema } => {
                assert_eq!(table_name, "test");
                assert_eq!(schema.columns.len(), 2);
                assert_eq!(schema.columns[0].name, "id");
                assert_eq!(schema.columns[1].name, "name");
            }
            _ => panic!("Expected CreateTable plan"),
        }
    }

    #[test]
    fn test_plan_drop_table() {
        let mut catalog = MemoryCatalog::new();
        catalog.add_table("test".to_string(), Schema { columns: vec![], primary_key: None });

        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("DROP TABLE test").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::DropTable {
                table_name,
                if_exists,
            } => {
                assert_eq!(table_name, "test");
                assert_eq!(if_exists, false);
            }
            _ => panic!("Expected DropTable plan"),
        }
    }

    #[test]
    fn test_plan_select_wildcard() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("SELECT * FROM users").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::Project { input, columns } => {
                // Should have projection with all columns
                assert_eq!(columns.len(), 3);

                // Should have table scan as input
                match input.as_ref() {
                    ExecutionPlan::TableScan { table_name, .. } => {
                        assert_eq!(table_name, "users");
                    }
                    _ => panic!("Expected TableScan as input to projection"),
                }
            }
            _ => panic!("Expected Project plan, got: {:?}", plan),
        }
    }

    #[test]
    fn test_plan_select_with_where() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("SELECT * FROM users WHERE age > 18").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::Project { input, .. } => match input.as_ref() {
                ExecutionPlan::Filter {
                    input: scan_input,
                    condition: _,
                } => match scan_input.as_ref() {
                    ExecutionPlan::TableScan { table_name, .. } => {
                        assert_eq!(table_name, "users");
                    }
                    _ => panic!("Expected TableScan as input to filter"),
                },
                _ => panic!("Expected Filter as input to projection"),
            },
            _ => panic!("Expected Project plan"),
        }
    }

    #[test]
    fn test_plan_insert() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("INSERT INTO users (name, age) VALUES ('Alice', 25)").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::Insert {
                table_name,
                columns,
                values,
                ..
            } => {
                assert_eq!(table_name, "users");
                assert_eq!(columns.as_ref().unwrap().len(), 2);
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].len(), 2);
            }
            _ => panic!("Expected Insert plan"),
        }
    }

    #[test]
    fn test_plan_update() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("UPDATE users SET age = 26 WHERE name = 'Alice'").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::Update {
                table_name,
                assignments,
                filter,
                ..
            } => {
                assert_eq!(table_name, "users");
                assert_eq!(assignments.len(), 1);
                assert_eq!(assignments[0].column, "age");
                assert!(filter.is_some());
            }
            _ => panic!("Expected Update plan"),
        }
    }

    #[test]
    fn test_plan_delete() {
        let catalog = create_test_catalog();
        let analyzer = SemanticAnalyzer::new(&catalog);
        let planner = QueryPlanner::new();

        let stmt = parse_sql("DELETE FROM users WHERE age < 18").unwrap();
        let analyzed = analyzer.analyze(stmt).unwrap();
        let plan = planner.create_plan(analyzed).unwrap();

        match plan {
            ExecutionPlan::Delete {
                table_name, filter, ..
            } => {
                assert_eq!(table_name, "users");
                assert!(filter.is_some());
            }
            _ => panic!("Expected Delete plan"),
        }
    }
}
