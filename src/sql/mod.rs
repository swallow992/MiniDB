//! SQL compilation module
//!
//! This module provides SQL parsing and compilation functionality,
//! transforming SQL statements into executable query plans.

pub mod analyzer;
pub mod lexer;
pub mod parser;
pub mod planner;

// Re-export commonly used types
pub use analyzer::{AnalyzedStatement, SemanticAnalyzer, SemanticError};
pub use lexer::{LexError, Lexer, Token};
pub use parser::{ParseError, Parser, Statement};
pub use planner::{ExecutionPlan, PlanError, QueryPlanner};

/// Parse a SQL string into a Statement
pub fn parse_sql(input: &str) -> Result<Statement, ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer)?;
    parser.parse_statement()
}

/// Analyze a parsed statement for semantic correctness
pub fn analyze_statement(stmt: Statement) -> Result<AnalyzedStatement, SemanticError> {
    let analyzer = SemanticAnalyzer::new();
    analyzer.analyze(stmt)
}

/// Create an execution plan from an analyzed statement
pub fn create_plan(stmt: AnalyzedStatement) -> Result<ExecutionPlan, PlanError> {
    let planner = QueryPlanner::new();
    planner.create_plan(stmt)
}
