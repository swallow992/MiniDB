//! MiniDB 查询优化器
//!
//! 此模块实现各种查询优化技术：
//! - 谓词下推
//! - 投影下推
//! - 连接重排序
//! - 常量折叠

use crate::sql::parser::{Expression, BinaryOperator};
use crate::sql::planner::{ExecutionPlan, PlanError, ProjectColumn};
use crate::types::Value;
use std::collections::HashSet;

/// 查询优化器配置
pub struct QueryOptimizer {
    /// 启用谓词下推优化
    enable_predicate_pushdown: bool,
    /// 启用投影下推优化
    enable_projection_pushdown: bool,
    /// 启用常量折叠优化
    enable_constant_folding: bool,
}

/// 优化统计信息
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// 下推的谓词数量
    pub predicates_pushed: usize,
    /// 下推的投影数量
    pub projections_pushed: usize,
    /// 折叠的常量数量
    pub constants_folded: usize,
    /// 重排序的连接数量
    pub joins_reordered: usize,
}

/// 带统计信息的优化执行计划
#[derive(Debug, Clone)]
pub struct OptimizedPlan {
    /// 优化后的执行计划
    pub plan: ExecutionPlan,
    /// 优化统计信息
    pub stats: OptimizationStats,
}

impl QueryOptimizer {
    /// 使用默认设置创建新的查询优化器
    pub fn new() -> Self {
        Self {
            enable_predicate_pushdown: true,
            enable_projection_pushdown: true,
            enable_constant_folding: true,
        }
    }

    /// 使用自定义设置创建新的查询优化器
    pub fn with_settings(
        predicate_pushdown: bool,
        projection_pushdown: bool,
        constant_folding: bool,
    ) -> Self {
        Self {
            enable_predicate_pushdown: predicate_pushdown,
            enable_projection_pushdown: projection_pushdown,
            enable_constant_folding: constant_folding,
        }
    }

    /// 优化执行计划
    pub fn optimize(&self, plan: ExecutionPlan) -> Result<OptimizedPlan, PlanError> {
        let mut optimized_plan = plan;
        let mut stats = OptimizationStats::default();

        // Apply optimizations in order
        if self.enable_constant_folding {
            optimized_plan = self.apply_constant_folding(optimized_plan, &mut stats)?;
        }
        
        if self.enable_predicate_pushdown {
            optimized_plan = self.apply_predicate_pushdown(optimized_plan, &mut stats)?;
        }
        
        if self.enable_projection_pushdown {
            optimized_plan = self.apply_projection_pushdown(optimized_plan, &mut stats)?;
        }

        Ok(OptimizedPlan {
            plan: optimized_plan,
            stats,
        })
    }

    /// 应用常量折叠优化
    fn apply_constant_folding(
        &self,
        plan: ExecutionPlan,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        let mut plan = plan;
        match &mut plan {
            ExecutionPlan::Filter { condition, input } => {
                // Fold constants in filter condition
                let folded_condition = self.fold_constants_in_expression(condition.clone())?;
                if !self.expressions_equal(condition, &folded_condition) {
                    *condition = folded_condition;
                    stats.constants_folded += 1;
                }
                *input = Box::new(self.apply_constant_folding(*input.clone(), stats)?);
            }
            ExecutionPlan::Project { columns, input } => {
                for proj_col in columns {
                    let folded_expr = self.fold_constants_in_expression(proj_col.expression.clone())?;
                    if !self.expressions_equal(&proj_col.expression, &folded_expr) {
                        proj_col.expression = folded_expr;
                        stats.constants_folded += 1;
                    }
                }
                *input = Box::new(self.apply_constant_folding(*input.clone(), stats)?);
            }
            ExecutionPlan::Join { left, right, .. } => {
                *left = Box::new(self.apply_constant_folding(*left.clone(), stats)?);
                *right = Box::new(self.apply_constant_folding(*right.clone(), stats)?);
            }
            _ => {} // Other plans don't need constant folding
        }
        
        Ok(plan)
    }

    /// 应用谓词下推优化
    fn apply_predicate_pushdown(
        &self,
        plan: ExecutionPlan,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        match plan {
            ExecutionPlan::Filter { condition, input } => {
                match *input {
                    ExecutionPlan::Join { left, right, condition: join_condition, join_type } => {
                        // Analyze which predicates can be pushed down
                        let pushable_predicates = self.analyze_pushable_predicates(&condition)?;
                        
                        let mut left_predicates = Vec::new();
                        let mut right_predicates = Vec::new();
                        let mut remaining_predicates = Vec::new();
                        
                        for predicate in pushable_predicates {
                            let predicate_tables = self.get_referenced_tables(&predicate);
                            let left_tables = self.get_plan_tables(&left);
                            let right_tables = self.get_plan_tables(&right);
                            
                            if predicate_tables.iter().all(|t| left_tables.contains(t)) {
                                left_predicates.push(predicate);
                                stats.predicates_pushed += 1;
                            } else if predicate_tables.iter().all(|t| right_tables.contains(t)) {
                                right_predicates.push(predicate);
                                stats.predicates_pushed += 1;
                            } else {
                                remaining_predicates.push(predicate);
                            }
                        }
                        
                        // Apply pushed predicates to left and right sides
                        let mut new_left = left;
                        let mut new_right = right;
                        
                        if !left_predicates.is_empty() {
                            let left_condition = self.combine_predicates(left_predicates)?;
                            new_left = Box::new(ExecutionPlan::Filter {
                                condition: left_condition,
                                input: new_left,
                            });
                        }
                        
                        if !right_predicates.is_empty() {
                            let right_condition = self.combine_predicates(right_predicates)?;
                            new_right = Box::new(ExecutionPlan::Filter {
                                condition: right_condition,
                                input: new_right,
                            });
                        }
                        
                        let join_plan = ExecutionPlan::Join {
                            left: new_left,
                            right: new_right,
                            condition: join_condition,
                            join_type,
                        };
                        
                        if remaining_predicates.is_empty() {
                            Ok(join_plan)
                        } else {
                            let remaining_condition = self.combine_predicates(remaining_predicates)?;
                            Ok(ExecutionPlan::Filter {
                                condition: remaining_condition,
                                input: Box::new(join_plan),
                            })
                        }
                    }
                    ExecutionPlan::TableScan { table_name, schema, .. } => {
                        // Push filter condition into table scan
                        Ok(ExecutionPlan::TableScan {
                            table_name,
                            schema,
                            filter: Some(condition),
                        })
                    }
                    _ => {
                        // Can't push down further, apply recursively to input
                        let optimized_input = self.apply_predicate_pushdown(*input, stats)?;
                        Ok(ExecutionPlan::Filter {
                            condition,
                            input: Box::new(optimized_input),
                        })
                    }
                }
            }
            _ => {
                // Recursively apply to child plans
                self.apply_predicate_pushdown_recursive(plan, stats)
            }
        }
    }

    /// 应用投影下推优化
    fn apply_projection_pushdown(
        &self,
        plan: ExecutionPlan,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        match plan {
            ExecutionPlan::Project { columns, input } => {
                let required_columns = self.get_required_columns_from_projections(&columns);
                
                // Try to push projection into the input
                let optimized_input = self.push_projection_into_plan(*input, &required_columns, stats)?;
                
                Ok(ExecutionPlan::Project {
                    columns,
                    input: Box::new(optimized_input),
                })
            }
            _ => {
                // Recursively apply to child plans
                self.apply_projection_pushdown_recursive(plan, stats)
            }
        }
    }

    /// 在表达式中折叠常量
    fn fold_constants_in_expression(&self, expr: Expression) -> Result<Expression, PlanError> {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left_folded = self.fold_constants_in_expression(*left)?;
                let right_folded = self.fold_constants_in_expression(*right)?;
                
                // Check if both operands are literals
                if let (Expression::Literal(left_val), Expression::Literal(right_val)) = (&left_folded, &right_folded) {
                    // Evaluate the operation
                    match self.evaluate_binary_op(left_val, &op, right_val) {
                        Ok(result) => return Ok(Expression::Literal(result)),
                        Err(_) => {} // Fall back to original expression
                    }
                }
                
                Ok(Expression::BinaryOp {
                    left: Box::new(left_folded),
                    op,
                    right: Box::new(right_folded),
                })
            }
            Expression::UnaryOp { op, expr } => {
                let folded_expr = self.fold_constants_in_expression(*expr)?;
                
                // Try to fold if operand is a literal
                if let Expression::Literal(val) = &folded_expr {
                    match self.evaluate_unary_op(&op, val) {
                        Ok(result) => return Ok(Expression::Literal(result)),
                        Err(_) => {} // Keep original expression if evaluation fails
                    }
                }
                
                Ok(Expression::UnaryOp {
                    op,
                    expr: Box::new(folded_expr),
                })
            }
            Expression::FunctionCall { name, args } => {
                let folded_args = args.into_iter()
                    .map(|arg| self.fold_constants_in_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                Ok(Expression::FunctionCall {
                    name,
                    args: folded_args,
                })
            }
            _ => Ok(expr), // Other expressions cannot be folded
        }
    }

    /// 对常量值执行二元运算
    fn evaluate_binary_op(
        &self,
        left: &Value,
        operator: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, PlanError> {
        match (left, operator, right) {
            (Value::Integer(a), BinaryOperator::Add, Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Integer(a), BinaryOperator::Subtract, Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Integer(a), BinaryOperator::Multiply, Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Integer(a), BinaryOperator::Divide, Value::Integer(b)) if *b != 0 => Ok(Value::Integer(a / b)),
            (Value::Integer(a), BinaryOperator::Equal, Value::Integer(b)) => Ok(Value::Boolean(a == b)),
            (Value::Integer(a), BinaryOperator::LessThan, Value::Integer(b)) => Ok(Value::Boolean(a < b)),
            (Value::Integer(a), BinaryOperator::GreaterThan, Value::Integer(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(PlanError::UnsupportedOperation { operation: "Unsupported binary operation for constant folding".to_string() }),
        }
    }

    /// 对常量值执行一元运算
    fn evaluate_unary_op(
        &self,
        operator: &crate::sql::parser::UnaryOperator,
        value: &Value,
    ) -> Result<Value, PlanError> {
        use crate::sql::parser::UnaryOperator;
        match (operator, value) {
            (UnaryOperator::Minus, Value::Integer(n)) => Ok(Value::Integer(-n)),
            (UnaryOperator::Not, Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ => Err(PlanError::UnsupportedOperation { operation: "Unsupported unary operation for constant folding".to_string() }),
        }
    }

    /// 检查两个表达式是否相等
    fn expressions_equal(&self, expr1: &Expression, expr2: &Expression) -> bool {
        match (expr1, expr2) {
            (Expression::Literal(v1), Expression::Literal(v2)) => v1 == v2,
            (Expression::Column(c1), Expression::Column(c2)) => c1 == c2,
            (
                Expression::BinaryOp { left: l1, op: op1, right: r1 },
                Expression::BinaryOp { left: l2, op: op2, right: r2 },
            ) => {
                op1 == op2 
                    && self.expressions_equal(l1, l2) 
                    && self.expressions_equal(r1, r2)
            }
            _ => false,
        }
    }

    /// 从投影列中获取所需列
    fn get_required_columns_from_projections(&self, columns: &[ProjectColumn]) -> HashSet<String> {
        let mut required = HashSet::new();
        for col in columns {
            let column_names = self.get_column_references(&col.expression);
            required.extend(column_names);
        }
        required
    }

    /// 获取表达式中的列引用
    fn get_column_references(&self, expr: &Expression) -> HashSet<String> {
        let mut columns = HashSet::new();
        match expr {
            Expression::Column(name) => {
                columns.insert(name.clone());
            }
            Expression::QualifiedColumn { column, .. } => {
                columns.insert(column.clone());
            }
            Expression::BinaryOp { left, right, .. } => {
                columns.extend(self.get_column_references(left));
                columns.extend(self.get_column_references(right));
            }
            Expression::UnaryOp { expr, .. } => {
                columns.extend(self.get_column_references(expr));
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    columns.extend(self.get_column_references(arg));
                }
            }
            _ => {} // Other expressions don't reference columns
        }
        columns
    }



    /// 获取执行计划引用的表
    fn get_plan_tables(&self, plan: &ExecutionPlan) -> HashSet<String> {
        match plan {
            ExecutionPlan::TableScan { table_name, .. } => {
                let mut tables = HashSet::new();
                tables.insert(table_name.clone());
                tables
            }
            ExecutionPlan::Join { left, right, .. } => {
                let mut tables = self.get_plan_tables(left);
                tables.extend(self.get_plan_tables(right));
                tables
            }
            ExecutionPlan::Project { input, .. } |
            ExecutionPlan::Filter { input, .. } => {
                self.get_plan_tables(input)
            }
            _ => HashSet::new(),
        }
    }

    /// 获取表达式中引用的表
    fn get_referenced_tables(&self, _expr: &Expression) -> HashSet<String> {
        // Simplified implementation - would need table resolution context
        HashSet::new()
    }

    /// 分析哪些谓词可以下推
    fn analyze_pushable_predicates(&self, condition: &Expression) -> Result<Vec<Expression>, PlanError> {
        // For now, treat the entire condition as one predicate
        // In a full implementation, this would split AND conditions
        Ok(vec![condition.clone()])
    }

    /// 使用 AND 组合多个谓词
    fn combine_predicates(&self, predicates: Vec<Expression>) -> Result<Expression, PlanError> {
        if predicates.is_empty() {
            return Err(PlanError::UnsupportedOperation { operation: "Cannot combine empty predicate list".to_string() });
        }
        
        let mut result = predicates[0].clone();
        for predicate in predicates.into_iter().skip(1) {
            result = Expression::BinaryOp {
                left: Box::new(result),
                op: BinaryOperator::And,
                right: Box::new(predicate),
            };
        }
        
        Ok(result)
    }

    /// 递归地向子计划应用谓词下推
    fn apply_predicate_pushdown_recursive(
        &self,
        plan: ExecutionPlan,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        // Recursively apply to child plans
        match plan {
            ExecutionPlan::Project { columns, input } => {
                let optimized_input = self.apply_predicate_pushdown(*input, stats)?;
                Ok(ExecutionPlan::Project {
                    columns,
                    input: Box::new(optimized_input),
                })
            }
            ExecutionPlan::Join { left, right, condition, join_type } => {
                let optimized_left = self.apply_predicate_pushdown(*left, stats)?;
                let optimized_right = self.apply_predicate_pushdown(*right, stats)?;
                Ok(ExecutionPlan::Join {
                    left: Box::new(optimized_left),
                    right: Box::new(optimized_right),
                    condition,
                    join_type,
                })
            }
            _ => Ok(plan),
        }
    }

    /// 递归地向子计划应用投影下推
    fn apply_projection_pushdown_recursive(
        &self,
        plan: ExecutionPlan,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        match plan {
            ExecutionPlan::Filter { condition, input } => {
                let optimized_input = self.apply_projection_pushdown(*input, stats)?;
                Ok(ExecutionPlan::Filter {
                    condition,
                    input: Box::new(optimized_input),
                })
            }
            _ => Ok(plan),
        }
    }

    /// 将投影需求推入计划
    fn push_projection_into_plan(
        &self,
        plan: ExecutionPlan,
        _required_columns: &HashSet<String>,
        stats: &mut OptimizationStats,
    ) -> Result<ExecutionPlan, PlanError> {
        // Simplified implementation - would need more sophisticated column tracking
        self.apply_projection_pushdown(plan, stats)
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::parser::BinaryOperator;
    
    #[test]
    fn test_constant_folding() {
        let optimizer = QueryOptimizer::new();
        
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Integer(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Integer(2))),
        };
        
        let folded = optimizer.fold_constants_in_expression(expr).unwrap();
        assert_eq!(folded, Expression::Literal(Value::Integer(3)));
    }
    
    #[test]
    fn test_unary_constant_folding() {
        let optimizer = QueryOptimizer::new();
        
        let expr = Expression::UnaryOp {
            op: crate::sql::parser::UnaryOperator::Minus,
            expr: Box::new(Expression::Literal(Value::Integer(5))),
        };
        
        let folded = optimizer.fold_constants_in_expression(expr).unwrap();
        assert_eq!(folded, Expression::Literal(Value::Integer(-5)));
    }
    
    #[test]
    fn test_predicate_combination() {
        let optimizer = QueryOptimizer::new();
        
        let pred1 = Expression::Column("a".to_string());
        let pred2 = Expression::Column("b".to_string());
        
        let combined = optimizer.combine_predicates(vec![pred1, pred2]).unwrap();
        
        match combined {
            Expression::BinaryOp { op: BinaryOperator::And, .. } => {
                // Success - predicates combined with AND
            }
            _ => panic!("Expected AND combination of predicates"),
        }
    }
}