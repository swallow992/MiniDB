//! SQL 编译模块
//!
//! 此模块提供 SQL 解析和编译功能，
//! 将 SQL 语句转换为可执行的查询计划。

pub mod analyzer;
pub mod diagnostics;
pub mod lexer;
pub mod optimizer;
pub mod parser;
pub mod planner;

// Re-export commonly used types
pub use analyzer::{AnalyzedStatement, SemanticAnalyzer, SemanticError};
pub use diagnostics::{DiagnosticEngine, DiagnosticContext, Suggestion, enhance_error_message};
pub use lexer::{LexError, Lexer, Token};
pub use optimizer::{QueryOptimizer, OptimizedPlan, OptimizationStats};
pub use parser::{ParseError, Parser, Statement};
pub use planner::{ExecutionPlan, PlanError, QueryPlanner};

/// 解析 SQL 字符串为语句
pub fn parse_sql(input: &str) -> Result<Statement, ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer)?;
    parser.parse_statement()
}

/// 分析已解析语句的语义正确性
pub fn analyze_statement(
    stmt: Statement,
    catalog: &dyn analyzer::SchemaCatalog,
) -> Result<AnalyzedStatement, SemanticError> {
    let analyzer = SemanticAnalyzer::new(catalog);
    analyzer.analyze(stmt)
}

/// 从已分析的语句创建执行计划
pub fn create_plan(stmt: AnalyzedStatement) -> Result<ExecutionPlan, PlanError> {
    let planner = QueryPlanner::new();
    planner.create_plan(stmt)
}
