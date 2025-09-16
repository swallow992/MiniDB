//! SQL parser
//!
//! Recursive descent parser for SQL statements.

use crate::sql::lexer::{LexError, Lexer, Token};
use crate::types::{DataType, Value};
use thiserror::Error;

/// Abstract syntax tree nodes for SQL statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// CREATE TABLE statement
    CreateTable {
        table_name: String,
        columns: Vec<ColumnDef>,
        constraints: Vec<TableConstraint>,
    },
    
    /// DROP TABLE statement
    DropTable {
        table_name: String,
        if_exists: bool,
    },
    
    /// INSERT statement
    Insert {
        table_name: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expression>>,
    },
    
    /// SELECT statement
    Select {
        select_list: SelectList,
        from_clause: Option<FromClause>,
        where_clause: Option<Expression>,
        group_by: Option<Vec<Expression>>,
        having: Option<Expression>,
        order_by: Option<Vec<OrderByExpr>>,
        limit: Option<u64>,
        offset: Option<u64>,
    },
    
    /// UPDATE statement
    Update {
        table_name: String,
        assignments: Vec<Assignment>,
        where_clause: Option<Expression>,
    },
    
    /// DELETE statement
    Delete {
        table_name: String,
        where_clause: Option<Expression>,
    },
}

/// Column definition in CREATE TABLE
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Expression>,
    pub primary_key: bool,
}

/// Table constraints
#[derive(Debug, Clone, PartialEq)]
pub enum TableConstraint {
    PrimaryKey(Vec<String>),
    ForeignKey {
        columns: Vec<String>,
        referenced_table: String,
        referenced_columns: Vec<String>,
    },
}

/// SELECT list
#[derive(Debug, Clone, PartialEq)]
pub enum SelectList {
    Wildcard,
    Expressions(Vec<SelectExpr>),
}

/// SELECT expression with optional alias
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpr {
    pub expr: Expression,
    pub alias: Option<String>,
}

/// FROM clause
#[derive(Debug, Clone, PartialEq)]
pub enum FromClause {
    Table(String),
    Join {
        left: Box<FromClause>,
        join_type: JoinType,
        right: Box<FromClause>,
        condition: Option<Expression>,
    },
}

/// Join types
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// ORDER BY expression
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByExpr {
    pub expr: Expression,
    pub desc: bool,
}

/// UPDATE assignment
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub column: String,
    pub value: Expression,
}

/// Expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal values
    Literal(Value),
    
    /// Column reference
    Column(String),
    
    /// Qualified column reference (table.column)
    QualifiedColumn {
        table: String,
        column: String,
    },
    
    /// Binary operations
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    
    /// Unary operations
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    
    /// Function calls
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    
    /// IN expression
    In {
        expr: Box<Expression>,
        list: Vec<Expression>,
    },
    
    /// BETWEEN expression
    Between {
        expr: Box<Expression>,
        low: Box<Expression>,
        high: Box<Expression>,
    },
    
    /// LIKE expression
    Like {
        expr: Box<Expression>,
        pattern: Box<Expression>,
    },
    
    /// IS NULL expression
    IsNull(Box<Expression>),
    
    /// IS NOT NULL expression
    IsNotNull(Box<Expression>),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    
    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}

/// SQL parser
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

/// Parser errors
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Lexical error: {0}")]
    LexError(#[from] LexError),
    
    #[error("Unexpected token: expected {expected}, found {found:?}")]
    UnexpectedToken { expected: String, found: Token },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid expression")]
    InvalidExpression,
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

impl Parser {
    /// Create a new parser
    pub fn new(mut lexer: Lexer) -> Result<Self, ParseError> {
        let current_token = lexer.next_token()?;
        Ok(Self {
            lexer,
            current_token,
        })
    }
    
    /// Advance to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
    
    /// Check if current token matches expected and advance
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token.clone(),
            })
        }
    }
    
    /// Parse a complete SQL statement
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.current_token {
            Token::Create => self.parse_create_statement(),
            Token::Drop => self.parse_drop_statement(),
            Token::Select => self.parse_select_statement(),
            Token::Insert => self.parse_insert_statement(),
            Token::Update => self.parse_update_statement(),
            Token::Delete => self.parse_delete_statement(),
            Token::EOF => Err(ParseError::UnexpectedEof),
            _ => Err(ParseError::UnexpectedToken {
                expected: "SQL statement".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }
    
    /// Parse CREATE statement
    fn parse_create_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Create)?;
        
        match &self.current_token {
            Token::Table => self.parse_create_table(),
            _ => Err(ParseError::UnexpectedToken {
                expected: "TABLE".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }
    
    /// Parse CREATE TABLE statement
    fn parse_create_table(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Table)?;
        
        let table_name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        self.expect(Token::LeftParen)?;
        
        let mut columns = Vec::new();
        let mut constraints = Vec::new();
        
        // Parse column definitions and constraints
        loop {
            if self.current_token == Token::RightParen {
                break;
            }
            
            // Try to parse column definition
            if let Token::Identifier(_) = &self.current_token {
                columns.push(self.parse_column_def()?);
            } else if self.current_token == Token::Primary {
                constraints.push(self.parse_primary_key_constraint()?);
            } else if self.current_token == Token::Foreign {
                constraints.push(self.parse_foreign_key_constraint()?);
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "column definition or constraint".to_string(),
                    found: self.current_token.clone(),
                });
            }
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else if self.current_token == Token::RightParen {
                break;
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "comma or closing parenthesis".to_string(),
                    found: self.current_token.clone(),
                });
            }
        }
        
        self.expect(Token::RightParen)?;
        
        Ok(Statement::CreateTable {
            table_name,
            columns,
            constraints,
        })
    }
    
    /// Parse column definition
    fn parse_column_def(&mut self) -> Result<ColumnDef, ParseError> {
        let name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "column name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        let data_type = self.parse_data_type()?;
        let mut nullable = true;
        let default = None;
        let mut primary_key = false;
        
        // Parse column constraints
        loop {
            match &self.current_token {
                Token::Not => {
                    self.advance()?;
                    if self.current_token == Token::Null {
                        self.advance()?;
                        nullable = false;
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            expected: "NULL".to_string(),
                            found: self.current_token.clone(),
                        });
                    }
                }
                Token::Primary => {
                    self.advance()?;
                    self.expect(Token::Key)?;
                    primary_key = true;
                }
                _ => break,
            }
        }
        
        Ok(ColumnDef {
            name,
            data_type,
            nullable,
            default,
            primary_key,
        })
    }
    
    /// Parse data type
    fn parse_data_type(&mut self) -> Result<DataType, ParseError> {
        let data_type = match &self.current_token {
            Token::Int => {
                self.advance()?;
                DataType::Integer
            }
            Token::BigInt => {
                self.advance()?;
                DataType::BigInt
            }
            Token::Float32 => {
                self.advance()?;
                DataType::Float
            }
            Token::Double => {
                self.advance()?;
                DataType::Double
            }
            Token::Varchar => {
                self.advance()?;
                // Parse size parameter if present
                if self.current_token == Token::LeftParen {
                    self.advance()?; // consume '('
                    
                    let size = match &self.current_token {
                        Token::Integer(n) => {
                            let size = *n as usize;
                            self.advance()?; // consume number
                            size
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "size number".to_string(),
                                found: self.current_token.clone(),
                            })
                        }
                    };
                    
                    self.expect(Token::RightParen)?; // consume ')'
                    DataType::Varchar(size)
                } else {
                    DataType::Varchar(255) // default size
                }
            }
            Token::Char => {
                self.advance()?;
                // TODO: Parse size parameter
                DataType::Varchar(1)
            }
            Token::Text => {
                self.advance()?;
                DataType::Varchar(65535)
            }
            Token::Bool => {
                self.advance()?;
                DataType::Boolean
            }
            Token::Date => {
                self.advance()?;
                DataType::Date
            }
            Token::Timestamp => {
                self.advance()?;
                DataType::Timestamp
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "data type".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        Ok(data_type)
    }
    
    /// Parse PRIMARY KEY constraint
    fn parse_primary_key_constraint(&mut self) -> Result<TableConstraint, ParseError> {
        self.expect(Token::Primary)?;
        self.expect(Token::Key)?;
        self.expect(Token::LeftParen)?;
        
        let mut columns = Vec::new();
        loop {
            if let Token::Identifier(name) = &self.current_token {
                columns.push(name.clone());
                self.advance()?;
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "column name".to_string(),
                    found: self.current_token.clone(),
                });
            }
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        self.expect(Token::RightParen)?;
        Ok(TableConstraint::PrimaryKey(columns))
    }
    
    /// Parse FOREIGN KEY constraint
    fn parse_foreign_key_constraint(&mut self) -> Result<TableConstraint, ParseError> {
        self.expect(Token::Foreign)?;
        self.expect(Token::Key)?;
        self.expect(Token::LeftParen)?;
        
        let mut columns = Vec::new();
        loop {
            if let Token::Identifier(name) = &self.current_token {
                columns.push(name.clone());
                self.advance()?;
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "column name".to_string(),
                    found: self.current_token.clone(),
                });
            }
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        self.expect(Token::RightParen)?;
        self.expect(Token::References)?;
        
        let referenced_table = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "referenced table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        self.expect(Token::LeftParen)?;
        
        let mut referenced_columns = Vec::new();
        loop {
            if let Token::Identifier(name) = &self.current_token {
                referenced_columns.push(name.clone());
                self.advance()?;
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "referenced column name".to_string(),
                    found: self.current_token.clone(),
                });
            }
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        self.expect(Token::RightParen)?;
        
        Ok(TableConstraint::ForeignKey {
            columns,
            referenced_table,
            referenced_columns,
        })
    }
    
    /// Parse DROP statement
    fn parse_drop_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Drop)?;
        self.expect(Token::Table)?;
        
        let if_exists = if self.current_token == Token::If {
            self.advance()?;
            self.expect(Token::Exists)?;
            true
        } else {
            false
        };
        
        let table_name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        Ok(Statement::DropTable {
            table_name,
            if_exists,
        })
    }
    
    /// Parse SELECT statement
    fn parse_select_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Select)?;
        
        let select_list = self.parse_select_list()?;
        
        let from_clause = if self.current_token == Token::From {
            self.advance()?;
            Some(self.parse_from_clause()?)
        } else {
            None
        };
        
        let where_clause = if self.current_token == Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        // Parse GROUP BY clause
        let group_by = if self.current_token == Token::Group {
            self.advance()?;
            self.expect(Token::By)?;
            Some(self.parse_group_by_list()?)
        } else {
            None
        };
        
        // TODO: Parse HAVING
        let having = None;
        
        // Parse ORDER BY clause
        let order_by = if self.current_token == Token::Order {
            self.advance()?;
            self.expect(Token::By)?;
            Some(self.parse_order_by_list()?)
        } else {
            None
        };
        
        // Parse LIMIT clause
        let limit = if self.current_token == Token::Limit {
            self.advance()?;
            match &self.current_token {
                Token::Integer(n) => {
                    let limit_value = *n as u64;
                    self.advance()?;
                    Some(limit_value)
                }
                _ => return Err(ParseError::UnexpectedToken {
                    expected: "integer".to_string(),
                    found: self.current_token.clone(),
                })
            }
        } else {
            None
        };
        
        // Parse OFFSET clause  
        let offset = if self.current_token == Token::Offset {
            self.advance()?;
            match &self.current_token {
                Token::Integer(n) => {
                    let offset_value = *n as u64;
                    self.advance()?;
                    Some(offset_value)
                }
                _ => return Err(ParseError::UnexpectedToken {
                    expected: "integer".to_string(),
                    found: self.current_token.clone(),
                })
            }
        } else {
            None
        };
        
        Ok(Statement::Select {
            select_list,
            from_clause,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        })
    }
    
    /// Parse SELECT list
    fn parse_select_list(&mut self) -> Result<SelectList, ParseError> {
        if self.current_token == Token::Multiply {
            self.advance()?;
            Ok(SelectList::Wildcard)
        } else {
            let mut expressions = Vec::new();
            
            loop {
                let expr = self.parse_expression()?;
                let alias = if self.current_token == Token::As {
                    self.advance()?;
                    match &self.current_token {
                        Token::Identifier(name) => {
                            let name = Some(name.clone());
                            self.advance()?;
                            name
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                
                expressions.push(SelectExpr { expr, alias });
                
                if self.current_token == Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }
            
            Ok(SelectList::Expressions(expressions))
        }
    }
    
    /// Parse FROM clause
    fn parse_from_clause(&mut self) -> Result<FromClause, ParseError> {
        match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                Ok(FromClause::Table(name))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "table name".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }
    
    /// Parse INSERT statement
    fn parse_insert_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Insert)?;
        self.expect(Token::Into)?;
        
        let table_name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        // Optional column list
        let columns = if self.current_token == Token::LeftParen {
            self.advance()?;
            let mut cols = Vec::new();
            
            loop {
                if let Token::Identifier(name) = &self.current_token {
                    cols.push(name.clone());
                    self.advance()?;
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "column name".to_string(),
                        found: self.current_token.clone(),
                    });
                }
                
                if self.current_token == Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }
            
            self.expect(Token::RightParen)?;
            Some(cols)
        } else {
            None
        };
        
        self.expect(Token::Values)?;
        
        let mut values = Vec::new();
        loop {
            self.expect(Token::LeftParen)?;
            
            let mut row_values = Vec::new();
            loop {
                row_values.push(self.parse_expression()?);
                
                if self.current_token == Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }
            
            self.expect(Token::RightParen)?;
            values.push(row_values);
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        Ok(Statement::Insert {
            table_name,
            columns,
            values,
        })
    }
    
    /// Parse UPDATE statement
    fn parse_update_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Update)?;
        
        let table_name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        self.expect(Token::Set)?;
        
        let mut assignments = Vec::new();
        loop {
            let column = match &self.current_token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance()?;
                    name
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "column name".to_string(),
                        found: self.current_token.clone(),
                    })
                }
            };
            
            self.expect(Token::Equal)?;
            let value = self.parse_expression()?;
            
            assignments.push(Assignment { column, value });
            
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        let where_clause = if self.current_token == Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::Update {
            table_name,
            assignments,
            where_clause,
        })
    }
    
    /// Parse DELETE statement
    fn parse_delete_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::Delete)?;
        self.expect(Token::From)?;
        
        let table_name = match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "table name".to_string(),
                    found: self.current_token.clone(),
                })
            }
        };
        
        let where_clause = if self.current_token == Token::Where {
            self.advance()?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::Delete {
            table_name,
            where_clause,
        })
    }
    
    /// Parse expression (simplified version)
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or_expression()
    }
    
    /// Parse OR expression
    fn parse_or_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and_expression()?;
        
        while self.current_token == Token::Or {
            self.advance()?;
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse AND expression
    fn parse_and_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality_expression()?;
        
        while self.current_token == Token::And {
            self.advance()?;
            let right = self.parse_equality_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse equality expression
    fn parse_equality_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison_expression()?;
        
        while matches!(
            self.current_token,
            Token::Equal | Token::NotEqual
        ) {
            let op = match self.current_token {
                Token::Equal => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse comparison expression
    fn parse_comparison_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive_expression()?;
        
        while matches!(
            self.current_token,
            Token::LessThan | Token::LessEqual | Token::GreaterThan | Token::GreaterEqual
        ) {
            let op = match self.current_token {
                Token::LessThan => BinaryOperator::LessThan,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::GreaterThan => BinaryOperator::GreaterThan,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_additive_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse additive expression
    fn parse_additive_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let op = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_multiplicative_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse multiplicative expression
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary_expression()?;
        
        while matches!(
            self.current_token,
            Token::Multiply | Token::Divide | Token::Modulo
        ) {
            let op = match self.current_token {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                Token::Modulo => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_unary_expression()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token {
            Token::Not => {
                self.advance()?;
                let expr = self.parse_unary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_unary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Minus,
                    expr: Box::new(expr),
                })
            }
            Token::Plus => {
                self.advance()?;
                let expr = self.parse_unary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Plus,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary_expression(),
        }
    }
    
    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token.clone() {
            Token::Integer(n) => {
                let value = Value::Integer(*n as i32);
                self.advance()?;
                Ok(Expression::Literal(value))
            }
            Token::Float(f) => {
                let value = Value::Double(*f);
                self.advance()?;
                Ok(Expression::Literal(value))
            }
            Token::String(s) => {
                let value = Value::Varchar(s.clone());
                self.advance()?;
                Ok(Expression::Literal(value))
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(*b);
                self.advance()?;
                Ok(Expression::Literal(value))
            }
            Token::Null => {
                self.advance()?;
                Ok(Expression::Literal(Value::Null))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                
                // Check for function call (name followed by left paren)
                if self.current_token == Token::LeftParen {
                    self.advance()?;
                    let mut args = Vec::new();
                    
                    // Handle empty argument list
                    if self.current_token != Token::RightParen {
                        loop {
                            // Handle special case for COUNT(*)
                            if self.current_token == Token::Multiply {
                                self.advance()?;
                                args.push(Expression::Literal(Value::Varchar("*".to_string())));
                            } else {
                                args.push(self.parse_expression()?);
                            }
                            
                            if self.current_token == Token::Comma {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }
                    
                    self.expect(Token::RightParen)?;
                    Ok(Expression::FunctionCall { name, args })
                } 
                // Check for qualified column (table.column)
                else if self.current_token == Token::Dot {
                    self.advance()?;
                    if let Token::Identifier(column) = &self.current_token {
                        let column = column.clone();
                        self.advance()?;
                        Ok(Expression::QualifiedColumn {
                            table: name,
                            column,
                        })
                    } else {
                        Err(ParseError::UnexpectedToken {
                            expected: "column name".to_string(),
                            found: self.current_token.clone(),
                        })
                    }
                } else {
                    Ok(Expression::Column(name))
                }
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }

    /// Parse ORDER BY clause list
    fn parse_order_by_list(&mut self) -> Result<Vec<OrderByExpr>, ParseError> {
        let mut order_exprs = Vec::new();
        
        loop {
            // Parse the expression (usually a column name)
            let expr = self.parse_expression()?;
            
            // Check for ASC/DESC
            let desc = match &self.current_token {
                Token::Desc => {
                    self.advance()?;
                    true
                }
                Token::Asc => {
                    self.advance()?;
                    false
                }
                _ => false, // Default to ASC
            };
            
            order_exprs.push(OrderByExpr { expr, desc });
            
            // Check if there's a comma for multiple order expressions
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        Ok(order_exprs)
    }

    /// Parse GROUP BY clause list
    fn parse_group_by_list(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut group_exprs = Vec::new();
        
        loop {
            // Parse the expression (usually a column name)
            let expr = self.parse_expression()?;
            group_exprs.push(expr);
            
            // Check if there's a comma for multiple group expressions
            if self.current_token == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        
        Ok(group_exprs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::lexer::Lexer;
    use crate::types::{DataType, Value};

    fn parse_sql(input: &str) -> Result<Statement, ParseError> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer)?;
        parser.parse_statement()
    }

    #[test]
    fn test_create_table() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR NOT NULL, age INT)";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::CreateTable { table_name, columns, .. } => {
                assert_eq!(table_name, "users");
                assert_eq!(columns.len(), 3);
                
                assert_eq!(columns[0].name, "id");
                assert_eq!(columns[0].data_type, DataType::Integer);
                assert_eq!(columns[0].primary_key, true);
                
                assert_eq!(columns[1].name, "name");
                assert_eq!(columns[1].data_type, DataType::Varchar(255));
                assert_eq!(columns[1].nullable, false);
                
                assert_eq!(columns[2].name, "age");
                assert_eq!(columns[2].data_type, DataType::Integer);
                assert_eq!(columns[2].nullable, true);
            }
            _ => panic!("Expected CreateTable statement"),
        }
    }

    #[test]
    fn test_select_simple() {
        let sql = "SELECT * FROM users";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Select { select_list, from_clause, .. } => {
                assert_eq!(select_list, SelectList::Wildcard);
                assert!(from_clause.is_some());
                
                if let Some(FromClause::Table(table_name)) = from_clause {
                    assert_eq!(table_name, "users");
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_select_with_columns() {
        let sql = "SELECT id, name FROM users";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Select { select_list, .. } => {
                match select_list {
                    SelectList::Expressions(expressions) => {
                        assert_eq!(expressions.len(), 2);
                        
                        match &expressions[0].expr {
                            Expression::Column(name) => assert_eq!(name, "id"),
                            _ => panic!("Expected column expression"),
                        }
                        
                        match &expressions[1].expr {
                            Expression::Column(name) => assert_eq!(name, "name"),
                            _ => panic!("Expected column expression"),
                        }
                    }
                    _ => panic!("Expected column expressions"),
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_select_with_where() {
        let sql = "SELECT * FROM users WHERE age > 18";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Select { where_clause, .. } => {
                assert!(where_clause.is_some());
                
                if let Some(Expression::BinaryOp { left, op, right }) = where_clause {
                    match left.as_ref() {
                        Expression::Column(name) => assert_eq!(name, "age"),
                        _ => panic!("Expected column expression"),
                    }
                    
                    assert_eq!(op, BinaryOperator::GreaterThan);
                    
                    match right.as_ref() {
                        Expression::Literal(Value::Integer(18)) => {},
                        _ => panic!("Expected integer literal 18"),
                    }
                }
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_insert() {
        let sql = "INSERT INTO users (name, age) VALUES ('Alice', 25), ('Bob', 30)";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Insert { table_name, columns, values } => {
                assert_eq!(table_name, "users");
                
                let columns = columns.unwrap();
                assert_eq!(columns, vec!["name", "age"]);
                
                assert_eq!(values.len(), 2);
                
                // First row
                assert_eq!(values[0].len(), 2);
                match &values[0][0] {
                    Expression::Literal(Value::Varchar(s)) => assert_eq!(s, "Alice"),
                    _ => panic!("Expected string literal 'Alice'"),
                }
                match &values[0][1] {
                    Expression::Literal(Value::Integer(25)) => {},
                    _ => panic!("Expected integer literal 25"),
                }
                
                // Second row  
                assert_eq!(values[1].len(), 2);
                match &values[1][0] {
                    Expression::Literal(Value::Varchar(s)) => assert_eq!(s, "Bob"),
                    _ => panic!("Expected string literal 'Bob'"),
                }
                match &values[1][1] {
                    Expression::Literal(Value::Integer(30)) => {},
                    _ => panic!("Expected integer literal 30"),
                }
            }
            _ => panic!("Expected Insert statement"),
        }
    }

    #[test]
    fn test_update() {
        let sql = "UPDATE users SET age = 26 WHERE name = 'Alice'";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Update { table_name, assignments, where_clause } => {
                assert_eq!(table_name, "users");
                
                assert_eq!(assignments.len(), 1);
                assert_eq!(assignments[0].column, "age");
                match &assignments[0].value {
                    Expression::Literal(Value::Integer(26)) => {},
                    _ => panic!("Expected integer literal 26"),
                }
                
                assert!(where_clause.is_some());
                // Could add more detailed where clause checking here
            }
            _ => panic!("Expected Update statement"),
        }
    }

    #[test]
    fn test_delete() {
        let sql = "DELETE FROM users WHERE age < 18";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::Delete { table_name, where_clause } => {
                assert_eq!(table_name, "users");
                assert!(where_clause.is_some());
            }
            _ => panic!("Expected Delete statement"),
        }
    }

    #[test]
    fn test_drop_table() {
        let sql = "DROP TABLE users";
        let stmt = parse_sql(sql).unwrap();
        
        match stmt {
            Statement::DropTable { table_name, if_exists } => {
                assert_eq!(table_name, "users");
                assert_eq!(if_exists, false);
            }
            _ => panic!("Expected DropTable statement"),
        }
    }

    #[test]
    fn test_complex_expression() {
        let sql = "SELECT * FROM users WHERE (age > 18 AND age < 65) OR name = 'admin'";
        let stmt = parse_sql(sql).unwrap();
        
        // Just verify it parses successfully - detailed expression testing would be extensive
        match stmt {
            Statement::Select { where_clause, .. } => {
                assert!(where_clause.is_some());
            }
            _ => panic!("Expected Select statement"),
        }
    }
}
