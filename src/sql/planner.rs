//! Query planner

use crate::sql::analyzer::AnalyzedStatement;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    // TODO: Add execution plan data
}

pub struct QueryPlanner {
    // TODO: Add planner state
}

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Not implemented")]
    NotImplemented,
}

impl QueryPlanner {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn create_plan(&self, _stmt: AnalyzedStatement) -> Result<ExecutionPlan, PlanError> {
        Err(PlanError::NotImplemented)
    }
}
