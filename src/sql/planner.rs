//! Query planner
//!
//! Converts analyzed SQL statements into executable query plans.
//! The planner performs query optimization and generates a tree of
//! operators that can be executed by the query executor.

use crate::sql::analyzer::AnalyzedStatement;
use crate::sql::parser::{Expression, FromClause, SelectList, Statement};
use crate::types::{DataType, Schema};
use std::collections::HashMap;
use thiserror::Error;

/// Execution plan representing a tree of operators
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPlan {
    /// Scan a table sequentially
    TableScan {
        table_name: String,
        schema: Schema,
        filter: Option<Expression>,
    },

    /// Project specific columns
    Project {
        input: Box<ExecutionPlan>,
        columns: Vec<ProjectColumn>,
    },

    /// Filter rows based on a condition
    Filter {
        input: Box<ExecutionPlan>,
        condition: Expression,
    },

    /// Insert data into a table
    Insert {
        table_name: String,
        schema: Schema,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expression>>,
    },

    /// Update rows in a table
    Update {
        table_name: String,
        schema: Schema,
        assignments: Vec<UpdateAssignment>,
        filter: Option<Expression>,
    },

    /// Delete rows from a table
    Delete {
        table_name: String,
        schema: Schema,
        filter: Option<Expression>,
    },

    /// Create a new table
    CreateTable { table_name: String, schema: Schema },

    /// Drop a table
    DropTable { table_name: String, if_exists: bool },

    /// Join two inputs
    Join {
        left: Box<ExecutionPlan>,
        right: Box<ExecutionPlan>,
        join_type: JoinType,
        condition: Option<Expression>,
    },

    /// Sort the input
    Sort {
        input: Box<ExecutionPlan>,
        sort_keys: Vec<SortKey>,
    },

    /// Limit the number of output rows
    Limit {
        input: Box<ExecutionPlan>,
        count: u64,
        offset: Option<u64>,
    },
}

/// Column projection specification
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectColumn {
    pub expression: Expression,
    pub alias: Option<String>,
    pub data_type: DataType,
}

/// Update assignment in UPDATE plan
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAssignment {
    pub column: String,
    pub expression: Expression,
}

/// Join type
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// Sort key specification
#[derive(Debug, Clone, PartialEq)]
pub struct SortKey {
    pub expression: Expression,
    pub descending: bool,
}

/// Query planner
pub struct QueryPlanner {
    // Future: Could add cost-based optimization state here
}

/// Planning errors
#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Table schema not found: {table}")]
    SchemaNotFound { table: String },

    #[error("Column not found in projection: {column}")]
    ProjectionColumnNotFound { column: String },

    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },

    #[error("Planning error: {message}")]
    PlanningError { message: String },
}

impl QueryPlanner {
    /// Create a new query planner
    pub fn new() -> Self {
        Self {}
    }

    /// Create an execution plan from an analyzed statement
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
                group_by: _, // TODO: Implement GROUP BY
                having: _,   // TODO: Implement HAVING
                order_by: _, // TODO: Implement ORDER BY
                limit: _,    // TODO: Implement LIMIT
                offset: _,   // TODO: Implement OFFSET
            } => self.plan_select(
                select_list,
                from_clause,
                where_clause,
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
        }
    }

    /// Plan a SELECT statement
    fn plan_select(
        &self,
        select_list: SelectList,
        from_clause: Option<FromClause>,
        where_clause: Option<Expression>,
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

        // Add projection
        plan = self.plan_select_list(plan, select_list, table_schemas, expression_types)?;

        Ok(plan)
    }

    /// Plan FROM clause
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

    /// Plan SELECT list (projection)
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

    /// Build schema from column definitions
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

        Ok(Schema {
            columns: column_defs,
        })
    }

    /// Build wildcard projection (SELECT *)
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

    /// Infer the data type of an expression
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
        catalog.add_table("test".to_string(), Schema { columns: vec![] });

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
