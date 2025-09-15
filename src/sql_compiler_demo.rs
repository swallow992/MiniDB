use std::collections::HashMap;
use std::io::{self, Write, IsTerminal};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,
    Identifier,
    Number,
    String,
    Operator,
    Punctuation,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum DataType {
    INT,
    VARCHAR(usize),
    DECIMAL(u8, u8),
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Column(String),
    Literal(String),
    BinaryOp {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<Column>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Expression>,
    },
    Select {
        columns: Vec<String>,
        from: String,
        where_clause: Option<Expression>,
    },
}

#[derive(Debug)]
pub enum CompilerError {
    LexicalError { line: usize, column: usize, message: String },
    SyntaxError { line: usize, column: usize, message: String },
    SemanticError { line: usize, column: usize, message: String },
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) {
        if self.position < self.input.len() && self.input[self.position] == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.position += 1;
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        value
    }

    fn read_number(&mut self) -> Result<String, CompilerError> {
        let mut value = String::new();
        let start_line = self.line;
        let start_column = self.column;

        while let Some(ch) = self.peek() {
            if ch.is_numeric() || ch == '.' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if value.chars().filter(|&c| c == '.').count() > 1 {
            return Err(CompilerError::LexicalError {
                line: start_line,
                column: start_column,
                message: "非法数字格式，包含多个小数点".to_string(),
            });
        }

        Ok(value)
    }

    fn read_string(&mut self) -> Result<String, CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        self.advance(); // 跳过开头的单引号

        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\'' {
                self.advance(); // 跳过结尾的单引号
                return Ok(value);
            }
            value.push(ch);
            self.advance();
        }

        Err(CompilerError::LexicalError {
            line: start_line,
            column: start_column,
            message: "未闭合的字符串字面量".to_string(),
        })
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }

            let ch = self.input[self.position];
            let start_line = self.line;
            let start_column = self.column;

            match ch {
                ';' | ',' | '(' | ')' | '=' | '>' | '<' | '*' => {
                    tokens.push(Token {
                        token_type: if "=><*".contains(ch) { TokenType::Operator } else { TokenType::Punctuation },
                        value: ch.to_string(),
                        line: start_line,
                        column: start_column,
                    });
                    self.advance();
                }
                '\'' => {
                    match self.read_string() {
                        Ok(value) => {
                            tokens.push(Token {
                                token_type: TokenType::String,
                                value,
                                line: start_line,
                                column: start_column,
                            });
                        }
                        Err(error) => return Err(error),
                    }
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    let value = self.read_identifier();
                    let token_type = match value.to_uppercase().as_str() {
                        "CREATE" | "TABLE" | "INSERT" | "INTO" | "VALUES" | "SELECT" | "FROM" | "WHERE" |
                        "INT" | "VARCHAR" | "DECIMAL" | "AND" | "OR" | "NOT" | "NULL" => TokenType::Keyword,
                        _ => TokenType::Identifier,
                    };
                    tokens.push(Token {
                        token_type,
                        value,
                        line: start_line,
                        column: start_column,
                    });
                }
                _ if ch.is_numeric() => {
                    match self.read_number() {
                        Ok(value) => {
                            tokens.push(Token {
                                token_type: TokenType::Number,
                                value,
                                line: start_line,
                                column: start_column,
                            });
                        }
                        Err(error) => return Err(error),
                    }
                }
                _ => {
                    // 检查是否是不可见字符或UTF-8 BOM
                    if ch.is_control() || ch == '\u{FEFF}' {
                        // 跳过控制字符和BOM
                        self.advance();
                        continue;
                    }
                    
                    return Err(CompilerError::LexicalError {
                        line: start_line,
                        column: start_column,
                        message: format!("未识别的字符: '{}' (Unicode: U+{:04X})", ch, ch as u32),
                    });
                }
            }
        }

        tokens.push(Token {
            token_type: TokenType::EOF,
            value: String::new(),
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.position).cloned().unwrap_or(Token {
            token_type: TokenType::EOF,
            value: String::new(),
            line: 0,
            column: 0,
        })
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: &str) -> Result<Token, CompilerError> {
        let token = self.current_token();
        if token.value.to_uppercase() == expected.to_uppercase() {
            self.advance();
            Ok(token)
        } else {
            Err(CompilerError::SyntaxError {
                line: token.line,
                column: token.column,
                message: format!("期望 '{}', 但找到 '{}'", expected, token.value),
            })
        }
    }

    fn parse_data_type(&mut self) -> Result<DataType, CompilerError> {
        let token = self.current_token();
        match token.value.to_uppercase().as_str() {
            "INT" => {
                self.advance();
                Ok(DataType::INT)
            }
            "VARCHAR" => {
                self.advance();
                self.expect("(")?;
                let size_token = self.current_token();
                if size_token.token_type != TokenType::Number {
                    return Err(CompilerError::SyntaxError {
                        line: size_token.line,
                        column: size_token.column,
                        message: "VARCHAR大小必须是数字".to_string(),
                    });
                }
                self.advance();
                self.expect(")")?;
                let size: usize = size_token.value.parse().map_err(|_| CompilerError::SyntaxError {
                    line: size_token.line,
                    column: size_token.column,
                    message: "无效的VARCHAR大小".to_string(),
                })?;
                Ok(DataType::VARCHAR(size))
            }
            "DECIMAL" => {
                self.advance();
                self.expect("(")?;
                let precision_token = self.current_token();
                if precision_token.token_type != TokenType::Number {
                    return Err(CompilerError::SyntaxError {
                        line: precision_token.line,
                        column: precision_token.column,
                        message: "DECIMAL精度必须是数字".to_string(),
                    });
                }
                self.advance();
                self.expect(",")?;
                let scale_token = self.current_token();
                if scale_token.token_type != TokenType::Number {
                    return Err(CompilerError::SyntaxError {
                        line: scale_token.line,
                        column: scale_token.column,
                        message: "DECIMAL刻度必须是数字".to_string(),
                    });
                }
                self.advance();
                self.expect(")")?;
                
                let precision: u8 = precision_token.value.parse().map_err(|_| CompilerError::SyntaxError {
                    line: precision_token.line,
                    column: precision_token.column,
                    message: "无效的DECIMAL精度".to_string(),
                })?;
                let scale: u8 = scale_token.value.parse().map_err(|_| CompilerError::SyntaxError {
                    line: scale_token.line,
                    column: scale_token.column,
                    message: "无效的DECIMAL刻度".to_string(),
                })?;
                Ok(DataType::DECIMAL(precision, scale))
            }
            _ => Err(CompilerError::SyntaxError {
                line: token.line,
                column: token.column,
                message: format!("不支持的数据类型: {}", token.value),
            }),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement, CompilerError> {
        self.expect("CREATE")?;
        self.expect("TABLE")?;
        
        let table_name = self.current_token();
        if table_name.token_type != TokenType::Identifier {
            return Err(CompilerError::SyntaxError {
                line: table_name.line,
                column: table_name.column,
                message: "期望表名".to_string(),
            });
        }
        self.advance();
        
        self.expect("(")?;
        
        let mut columns = Vec::new();
        
        while self.current_token().value != ")" {
            let column_name = self.current_token();
            if column_name.token_type != TokenType::Identifier {
                return Err(CompilerError::SyntaxError {
                    line: column_name.line,
                    column: column_name.column,
                    message: "期望列名".to_string(),
                });
            }
            self.advance();
            
            let data_type = self.parse_data_type()?;
            
            columns.push(Column {
                name: column_name.value,
                data_type,
                nullable: true,
            });
            
            if self.current_token().value == "," {
                self.advance();
            } else if self.current_token().value != ")" {
                return Err(CompilerError::SyntaxError {
                    line: self.current_token().line,
                    column: self.current_token().column,
                    message: "期望 ',' 或 ')'".to_string(),
                });
            }
        }
        
        self.expect(")")?;
        
        Ok(Statement::CreateTable {
            name: table_name.value,
            columns,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, CompilerError> {
        self.expect("INSERT")?;
        self.expect("INTO")?;
        
        let table_name = self.current_token();
        if table_name.token_type != TokenType::Identifier {
            return Err(CompilerError::SyntaxError {
                line: table_name.line,
                column: table_name.column,
                message: "期望表名".to_string(),
            });
        }
        self.advance();
        
        let mut columns = Vec::new();
        if self.current_token().value == "(" {
            self.advance();
            while self.current_token().value != ")" {
                let column_name = self.current_token();
                if column_name.token_type != TokenType::Identifier {
                    return Err(CompilerError::SyntaxError {
                        line: column_name.line,
                        column: column_name.column,
                        message: "期望列名".to_string(),
                    });
                }
                columns.push(column_name.value);
                self.advance();
                
                if self.current_token().value == "," {
                    self.advance();
                } else if self.current_token().value != ")" {
                    return Err(CompilerError::SyntaxError {
                        line: self.current_token().line,
                        column: self.current_token().column,
                        message: "期望 ',' 或 ')'".to_string(),
                    });
                }
            }
            self.advance();
        }
        
        self.expect("VALUES")?;
        self.expect("(")?;
        
        let mut values = Vec::new();
        while self.current_token().value != ")" {
            let value_token = self.current_token();
            let expr = match value_token.token_type {
                TokenType::String => Expression::Literal(value_token.value),
                TokenType::Number => Expression::Literal(value_token.value),
                TokenType::Identifier => Expression::Column(value_token.value),
                _ => return Err(CompilerError::SyntaxError {
                    line: value_token.line,
                    column: value_token.column,
                    message: "期望值表达式".to_string(),
                }),
            };
            values.push(expr);
            self.advance();
            
            if self.current_token().value == "," {
                self.advance();
            } else if self.current_token().value != ")" {
                return Err(CompilerError::SyntaxError {
                    line: self.current_token().line,
                    column: self.current_token().column,
                    message: "期望 ',' 或 ')'".to_string(),
                });
            }
        }
        
        self.expect(")")?;
        
        Ok(Statement::Insert {
            table: table_name.value,
            columns,
            values,
        })
    }

    fn parse_select(&mut self) -> Result<Statement, CompilerError> {
        self.expect("SELECT")?;
        
        let mut columns = Vec::new();
        
        // 处理 SELECT *
        if self.current_token().value == "*" {
            columns.push("*".to_string());
            self.advance();
        } else {
            // 处理具体列名
            while self.current_token().token_type == TokenType::Identifier {
                columns.push(self.current_token().value.clone());
                self.advance();
                
                if self.current_token().value == "," {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        self.expect("FROM")?;
        
        let table_name = self.current_token();
        if table_name.token_type != TokenType::Identifier {
            return Err(CompilerError::SyntaxError {
                line: table_name.line,
                column: table_name.column,
                message: "期望表名".to_string(),
            });
        }
        self.advance();
        
        let where_clause = if self.current_token().value.to_uppercase() == "WHERE" {
            self.advance();
            // 简单的 WHERE 条件解析
            let left_column = self.current_token();
            if left_column.token_type != TokenType::Identifier {
                return Err(CompilerError::SyntaxError {
                    line: left_column.line,
                    column: left_column.column,
                    message: "期望列名".to_string(),
                });
            }
            self.advance();
            
            let operator = self.current_token();
            if operator.token_type != TokenType::Operator {
                return Err(CompilerError::SyntaxError {
                    line: operator.line,
                    column: operator.column,
                    message: "期望操作符".to_string(),
                });
            }
            self.advance();
            
            let right_value = self.current_token();
            let right_expr = match right_value.token_type {
                TokenType::String | TokenType::Number => Expression::Literal(right_value.value),
                TokenType::Identifier => Expression::Column(right_value.value),
                _ => return Err(CompilerError::SyntaxError {
                    line: right_value.line,
                    column: right_value.column,
                    message: "期望值表达式".to_string(),
                }),
            };
            self.advance();
            
            Some(Expression::BinaryOp {
                left: Box::new(Expression::Column(left_column.value)),
                operator: operator.value,
                right: Box::new(right_expr),
            })
        } else {
            None
        };
        
        Ok(Statement::Select {
            columns,
            from: table_name.value,
            where_clause,
        })
    }

    pub fn parse(&mut self) -> Result<Statement, CompilerError> {
        let token = self.current_token();
        match token.value.to_uppercase().as_str() {
            "CREATE" => self.parse_create_table(),
            "INSERT" => self.parse_insert(),
            "SELECT" => self.parse_select(),
            _ => Err(CompilerError::SyntaxError {
                line: token.line,
                column: token.column,
                message: format!("不支持的语句类型: {}", token.value),
            }),
        }
    }
}

pub struct SemanticAnalyzer {
    tables: HashMap<String, Vec<Column>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, ast: &Statement) -> Result<(), CompilerError> {
        match ast {
            Statement::CreateTable { name, columns } => {
                if self.tables.contains_key(name) {
                    return Err(CompilerError::SemanticError {
                        line: 1,
                        column: 1,
                        message: format!("表 '{}' 已存在", name),
                    });
                }
                
                for column in columns {
                    if column.name.is_empty() {
                        return Err(CompilerError::SemanticError {
                            line: 1,
                            column: 1,
                            message: "列名不能为空".to_string(),
                        });
                    }
                }
                
                self.tables.insert(name.clone(), columns.clone());
                Ok(())
            }
            Statement::Insert { table, columns, values } => {
                if !self.tables.contains_key(table) {
                    return Err(CompilerError::SemanticError {
                        line: 1,
                        column: 1,
                        message: format!("表 '{}' 不存在", table),
                    });
                }
                
                let table_columns = &self.tables[table];
                
                if !columns.is_empty() && columns.len() != values.len() {
                    return Err(CompilerError::SemanticError {
                        line: 1,
                        column: 1,
                        message: "列数与值数不匹配".to_string(),
                    });
                }
                
                if columns.is_empty() && table_columns.len() != values.len() {
                    return Err(CompilerError::SemanticError {
                        line: 1,
                        column: 1,
                        message: "值数与表列数不匹配".to_string(),
                    });
                }
                
                Ok(())
            }
            Statement::Select { columns, from, where_clause: _ } => {
                if !self.tables.contains_key(from) {
                    return Err(CompilerError::SemanticError {
                        line: 1,
                        column: 1,
                        message: format!("表 '{}' 不存在", from),
                    });
                }
                
                let table_columns = &self.tables[from];
                
                for column in columns {
                    if column != "*" {
                        if !table_columns.iter().any(|c| c.name == *column) {
                            return Err(CompilerError::SemanticError {
                                line: 1,
                                column: 1,
                                message: format!("列 '{}' 在表 '{}' 中不存在", column, from),
                            });
                        }
                    }
                }
                
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct QueryPlan {
    pub operations: Vec<Operation>,
    pub estimated_cost: f64,
}

#[derive(Debug)]
pub enum Operation {
    TableScan { table: String },
    Filter { condition: String },
    Project { columns: Vec<String> },
    Insert { table: String, values: Vec<String> },
    CreateTable { name: String, schema: String },
}

pub struct QueryPlanner;

impl QueryPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_plan(&self, ast: &Statement) -> Result<QueryPlan, CompilerError> {
        let mut operations = Vec::new();
        let estimated_cost = 1.0;

        match ast {
            Statement::CreateTable { name, columns } => {
                let schema = columns.iter()
                    .map(|c| format!("{}: {:?}", c.name, c.data_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                operations.push(Operation::CreateTable {
                    name: name.clone(),
                    schema,
                });
            }
            Statement::Insert { table, values, .. } => {
                let value_strings = values.iter()
                    .map(|v| match v {
                        Expression::Literal(s) => s.clone(),
                        Expression::Column(s) => s.clone(),
                        Expression::BinaryOp { .. } => "复杂表达式".to_string(),
                    })
                    .collect();
                
                operations.push(Operation::Insert {
                    table: table.clone(),
                    values: value_strings,
                });
            }
            Statement::Select { columns, from, where_clause } => {
                operations.push(Operation::TableScan {
                    table: from.clone(),
                });
                
                if let Some(condition) = where_clause {
                    let condition_str = match condition {
                        Expression::BinaryOp { left, operator, right } => {
                            format!("{:?} {} {:?}", left, operator, right)
                        }
                        _ => "复杂条件".to_string(),
                    };
                    operations.push(Operation::Filter { condition: condition_str });
                }
                
                operations.push(Operation::Project {
                    columns: columns.clone(),
                });
            }
        }

        Ok(QueryPlan {
            operations,
            estimated_cost,
        })
    }
}

// 修复：创建编译器会话结构体
pub struct SQLCompilerSession {
    semantic_analyzer: SemanticAnalyzer,
}

impl SQLCompilerSession {
    pub fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::new(),
        }
    }

    pub fn compile_sql(&mut self, sql: &str) -> Result<(Vec<Token>, Statement, QueryPlan), CompilerError> {
        // 1. 词法分析
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize()?;

        // 2. 语法分析
        let mut parser = Parser::new(tokens.clone());
        let ast = parser.parse()?;

        // 3. 语义分析
        self.semantic_analyzer.analyze(&ast)?;

        // 4. 查询计划生成
        let planner = QueryPlanner::new();
        let plan = planner.generate_plan(&ast)?;

        Ok((tokens, ast, plan))
    }
}

fn process_sql_command(session: &mut SQLCompilerSession, sql: &str) {
    match session.compile_sql(sql) {
        Ok((tokens, ast, plan)) => {
            println!("✅ 编译成功！");
            println!();
            
            // 词法分析结果
            println!("【四元式词法分析】");
            for (j, token) in tokens.iter().enumerate() {
                if token.token_type != TokenType::EOF {
                    println!("  ({}: {:?}, '{}', {}:{})", 
                        j + 1, token.token_type, token.value, token.line, token.column);
                }
            }
            println!();
            
            // 语法分析结果
            println!("【抽象语法树】");
            println!("  {:?}", ast);
            println!();
            
            // 查询计划
            println!("【执行计划】");
            for (j, op) in plan.operations.iter().enumerate() {
                println!("  步骤 {}: {:?}", j + 1, op);
            }
            println!("  预估成本: {:.2}", plan.estimated_cost);
        }
        Err(error) => {
            println!("❌ 编译失败:");
            match error {
                CompilerError::LexicalError { line, column, message } => {
                    println!("  [词法错误, 行{}:列{}, {}]", line, column, message);
                }
                CompilerError::SyntaxError { line, column, message } => {
                    println!("  [语法错误, 行{}:列{}, {}]", line, column, message);
                }
                CompilerError::SemanticError { line, column, message } => {
                    println!("  [语义错误, 行{}:列{}, {}]", line, column, message);
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    println!("=== MiniDB SQL编译器演示 ===");
    println!();

    // 自动测试示例
    let test_cases = vec![
        "CREATE TABLE student (id INT, name VARCHAR(50), score DECIMAL(5,2))",
        "INSERT INTO student (id, name, score) VALUES (1, 'Alice', 95.5)",
        "INSERT INTO student VALUES (2, 'Bob', 87.0)",
        "SELECT * FROM student",
        "SELECT name, score FROM student WHERE score > 90",
    ];

    let mut session = SQLCompilerSession::new(); // 会话保持状态

    for (i, sql) in test_cases.iter().enumerate() {
        println!("测试用例 {}: {}", i + 1, sql);
        println!("{}", "-".repeat(60));
        
        match session.compile_sql(sql) {
            Ok((tokens, ast, plan)) => {
                println!("✅ 编译成功！");
                println!();
                
                // 词法分析结果
                println!("【四元式词法分析】");
                for (j, token) in tokens.iter().enumerate() {
                    if token.token_type != TokenType::EOF {
                        println!("  ({}: {:?}, '{}', {}:{})", 
                            j + 1, token.token_type, token.value, token.line, token.column);
                    }
                }
                println!();
                
                // 语法分析结果
                println!("【抽象语法树】");
                println!("  {:?}", ast);
                println!();
                
                // 查询计划
                println!("【执行计划】");
                for (j, op) in plan.operations.iter().enumerate() {
                    println!("  步骤 {}: {:?}", j + 1, op);
                }
                println!("  预估成本: {:.2}", plan.estimated_cost);
            }
            Err(error) => {
                println!("❌ 编译失败:");
                match error {
                    CompilerError::LexicalError { line, column, message } => {
                        println!("  [词法错误, 行{}:列{}, {}]", line, column, message);
                    }
                    CompilerError::SyntaxError { line, column, message } => {
                        println!("  [语法错误, 行{}:列{}, {}]", line, column, message);
                    }
                    CompilerError::SemanticError { line, column, message } => {
                        println!("  [语义错误, 行{}:列{}, {}]", line, column, message);
                    }
                }
            }
        }
        
        println!();
        println!("{}", "=".repeat(60));
        println!();
    }

    // 检查是否是交互式终端
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        // 交互式模式
        println!("=== 交互式模式 ===");
        println!("请输入SQL语句 (输入 'quit' 或 'exit' 退出):");
        
        let mut interactive_session = SQLCompilerSession::new();
        
        loop {
            print!("SQL> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let sql = input.trim();
                    
                    if sql.to_lowercase() == "quit" || sql.to_lowercase() == "exit" {
                        println!("再见!");
                        break;
                    }
                    
                    if sql.is_empty() {
                        continue;
                    }
                    
                    process_sql_command(&mut interactive_session, sql);
                    println!();
                }
                Err(error) => {
                    println!("读取输入时出错: {}", error);
                    break;
                }
            }
        }
    } else {
        // 管道输入模式
        println!("=== 管道输入模式 ===");
        let mut pipe_session = SQLCompilerSession::new();
        
        let mut input = String::new();
        while let Ok(bytes_read) = io::stdin().read_line(&mut input) {
            if bytes_read == 0 {
                break; // EOF
            }
            
            let sql = input.trim();
            if !sql.is_empty() {
                println!("处理命令: {}", sql);
                process_sql_command(&mut pipe_session, sql);
                println!();
            }
            
            input.clear();
        }
    }

    Ok(())
}
