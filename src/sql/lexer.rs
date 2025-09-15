//! SQL lexical analyzer
//!
//! Tokenizes SQL input into a stream of tokens for parsing.

use std::collections::HashMap;
use thiserror::Error;

/// SQL token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,

    // Identifiers and keywords
    Identifier(String),

    // Keywords (SQL reserved words)
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Drop,
    Alter,
    Index,
    Primary,
    Key,
    Foreign,
    References,
    Not,
    And,
    Or,
    In,
    Like,
    Between,
    Is,
    As,
    Distinct,
    Order,
    By,
    Group,
    Having,
    Limit,
    Offset,
    Join,
    Inner,
    Left,
    Right,
    Full,
    Outer,
    On,
    Union,
    All,
    Exists,
    Case,
    When,
    Then,
    Else,
    End,
    If,

    // Data types
    Int,
    BigInt,
    Float32,
    Double,
    Varchar,
    Char,
    Text,
    Bool,
    Date,
    Timestamp,

    // Operators
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    Modulo,       // %
    Equal,        // =
    NotEqual,     // <> or !=
    LessThan,     // <
    LessEqual,    // <=
    GreaterThan,  // >
    GreaterEqual, // >=

    // Punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Semicolon,    // ;
    Dot,          // .

    // Special
    Wildcard, // *
    EOF,
}

/// Token information with position details
#[derive(Debug, Clone, PartialEq)]
pub struct TokenInfo {
    /// Token type and value
    pub token: Token,
    /// Token category code
    pub category: TokenCategory,
    /// Token lexeme (original text)
    pub lexeme: String,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)  
    pub column: u32,
}

/// Token category codes for output format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenCategory {
    Keyword = 1,
    Identifier = 2,
    Integer = 3,
    Float = 4,
    String = 5,
    Operator = 6,
    Delimiter = 7,
    Comment = 8,
    EOF = 9,
}

/// SQL lexer
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    keywords: HashMap<String, Token>,
    line: u32,
    column: u32,
}

/// Lexer errors
#[derive(Error, Debug)]
pub enum LexError {
    #[error("Unexpected character: '{0}' at position {1}")]
    UnexpectedCharacter(char, usize),

    #[error("Unterminated string at position {0}")]
    UnterminatedString(usize),

    #[error("Invalid number format at position {0}")]
    InvalidNumber(usize),
}

impl Lexer {
    /// Create a new lexer
    pub fn new(input: &str) -> Self {
        let input: Vec<char> = input.chars().collect();
        let current_char = input.get(0).copied();

        let mut lexer = Self {
            input,
            position: 0,
            current_char,
            keywords: HashMap::new(),
            line: 1,
            column: 1,
        };

        lexer.init_keywords();
        lexer
    }

    /// Initialize keyword mapping
    fn init_keywords(&mut self) {
        let keywords = [
            ("SELECT", Token::Select),
            ("FROM", Token::From),
            ("WHERE", Token::Where),
            ("INSERT", Token::Insert),
            ("INTO", Token::Into),
            ("VALUES", Token::Values),
            ("UPDATE", Token::Update),
            ("SET", Token::Set),
            ("DELETE", Token::Delete),
            ("CREATE", Token::Create),
            ("TABLE", Token::Table),
            ("DROP", Token::Drop),
            ("ALTER", Token::Alter),
            ("INDEX", Token::Index),
            ("PRIMARY", Token::Primary),
            ("KEY", Token::Key),
            ("FOREIGN", Token::Foreign),
            ("REFERENCES", Token::References),
            ("NOT", Token::Not),
            ("AND", Token::And),
            ("OR", Token::Or),
            ("IN", Token::In),
            ("LIKE", Token::Like),
            ("BETWEEN", Token::Between),
            ("IS", Token::Is),
            ("AS", Token::As),
            ("DISTINCT", Token::Distinct),
            ("ORDER", Token::Order),
            ("BY", Token::By),
            ("GROUP", Token::Group),
            ("HAVING", Token::Having),
            ("LIMIT", Token::Limit),
            ("OFFSET", Token::Offset),
            ("JOIN", Token::Join),
            ("INNER", Token::Inner),
            ("LEFT", Token::Left),
            ("RIGHT", Token::Right),
            ("FULL", Token::Full),
            ("OUTER", Token::Outer),
            ("ON", Token::On),
            ("UNION", Token::Union),
            ("ALL", Token::All),
            ("EXISTS", Token::Exists),
            ("CASE", Token::Case),
            ("WHEN", Token::When),
            ("THEN", Token::Then),
            ("ELSE", Token::Else),
            ("END", Token::End),
            ("IF", Token::If),
            ("INT", Token::Int),
            ("INTEGER", Token::Int), // Support both INT and INTEGER
            ("BIGINT", Token::BigInt),
            ("FLOAT", Token::Float32),
            ("DOUBLE", Token::Double),
            ("VARCHAR", Token::Varchar),
            ("CHAR", Token::Char),
            ("TEXT", Token::Text),
            ("BOOLEAN", Token::Bool),
            ("BOOL", Token::Bool),
            ("DATE", Token::Date),
            ("TIMESTAMP", Token::Timestamp),
            ("NULL", Token::Null),
            ("TRUE", Token::Boolean(true)),
            ("FALSE", Token::Boolean(false)),
        ];

        for (keyword, token) in keywords {
            self.keywords.insert(keyword.to_string(), token);
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        if let Some('\n') = self.current_char {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// Skip whitespace characters and BOM
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() || ch == '\u{feff}' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip line comments starting with --
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    /// Skip block comments /* ... */
    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        self.advance(); // skip '/'
        self.advance(); // skip '*'

        while let Some(ch) = self.current_char {
            if ch == '*' && self.peek() == Some('/') {
                self.advance(); // skip '*'
                self.advance(); // skip '/'
                return Ok(());
            }
            self.advance();
        }

        Err(LexError::UnterminatedString(self.position))
    }

    /// Read a number (integer or float)
    fn read_number(&mut self) -> Result<Token, LexError> {
        let start_pos = self.position;
        let mut number_str = String::new();
        let mut is_float = false;

        // Read digits and optional decimal point
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek().map_or(false, |c| c.is_ascii_digit()) {
                is_float = true;
                number_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            number_str
                .parse::<f64>()
                .map(Token::Float)
                .map_err(|_| LexError::InvalidNumber(start_pos))
        } else {
            number_str
                .parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| LexError::InvalidNumber(start_pos))
        }
    }

    /// Read a string literal
    fn read_string(&mut self) -> Result<Token, LexError> {
        let start_pos = self.position;
        self.advance(); // skip opening quote

        let mut string_value = String::new();

        while let Some(ch) = self.current_char {
            if ch == '\'' {
                self.advance(); // skip closing quote
                return Ok(Token::String(string_value));
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance();
                match self.current_char {
                    Some('n') => string_value.push('\n'),
                    Some('t') => string_value.push('\t'),
                    Some('r') => string_value.push('\r'),
                    Some('\\') => string_value.push('\\'),
                    Some('\'') => string_value.push('\''),
                    Some(escaped) => {
                        string_value.push('\\');
                        string_value.push(escaped);
                    }
                    None => return Err(LexError::UnterminatedString(start_pos)),
                }
                self.advance();
            } else {
                string_value.push(ch);
                self.advance();
            }
        }

        Err(LexError::UnterminatedString(start_pos))
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword
        let upper_identifier = identifier.to_uppercase();
        self.keywords
            .get(&upper_identifier)
            .cloned()
            .unwrap_or_else(|| Token::Identifier(identifier))
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        loop {
            self.skip_whitespace();

            match self.current_char {
                None => return Ok(Token::EOF),

                Some(ch) => match ch {
                    // Comments
                    '-' if self.peek() == Some('-') => {
                        self.skip_line_comment();
                        continue;
                    }
                    '/' if self.peek() == Some('*') => {
                        self.skip_block_comment()?;
                        continue;
                    }

                    // Numbers
                    '0'..='9' => return self.read_number(),

                    // String literals
                    '\'' => return self.read_string(),

                    // Identifiers and keywords
                    'a'..='z' | 'A'..='Z' | '_' => return Ok(self.read_identifier()),

                    // Operators and punctuation
                    '+' => {
                        self.advance();
                        return Ok(Token::Plus);
                    }
                    '-' => {
                        self.advance();
                        return Ok(Token::Minus);
                    }
                    '*' => {
                        self.advance();
                        return Ok(Token::Multiply);
                    }
                    '/' => {
                        self.advance();
                        return Ok(Token::Divide);
                    }
                    '%' => {
                        self.advance();
                        return Ok(Token::Modulo);
                    }
                    '=' => {
                        self.advance();
                        return Ok(Token::Equal);
                    }
                    '!' if self.peek() == Some('=') => {
                        self.advance();
                        self.advance();
                        return Ok(Token::NotEqual);
                    }
                    '<' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            return Ok(Token::LessEqual);
                        } else if self.current_char == Some('>') {
                            self.advance();
                            return Ok(Token::NotEqual);
                        } else {
                            return Ok(Token::LessThan);
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            return Ok(Token::GreaterEqual);
                        } else {
                            return Ok(Token::GreaterThan);
                        }
                    }
                    '(' => {
                        self.advance();
                        return Ok(Token::LeftParen);
                    }
                    ')' => {
                        self.advance();
                        return Ok(Token::RightParen);
                    }
                    '[' => {
                        self.advance();
                        return Ok(Token::LeftBracket);
                    }
                    ']' => {
                        self.advance();
                        return Ok(Token::RightBracket);
                    }
                    ',' => {
                        self.advance();
                        return Ok(Token::Comma);
                    }
                    ';' => {
                        self.advance();
                        return Ok(Token::Semicolon);
                    }
                    '.' => {
                        self.advance();
                        return Ok(Token::Dot);
                    }

                    _ => return Err(LexError::UnexpectedCharacter(ch, self.position)),
                },
            }
        }
    }

    /// Get all tokens (useful for debugging)
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token == Token::EOF;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get the next token with position information
    pub fn next_token_info(&mut self) -> Result<TokenInfo, LexError> {
        // Skip whitespace and comments first
        loop {
            self.skip_whitespace();

            match self.current_char {
                None => {
                    return Ok(TokenInfo {
                        token: Token::EOF,
                        category: TokenCategory::EOF,
                        lexeme: String::new(),
                        line: self.line,
                        column: self.column,
                    })
                }

                Some(ch) => match ch {
                    // Comments
                    '-' if self.peek() == Some('-') => {
                        self.skip_line_comment();
                        continue;
                    }
                    '/' if self.peek() == Some('*') => {
                        self.skip_block_comment()?;
                        continue;
                    }
                    _ => break, // Found actual token
                },
            }
        }

        // Record position at start of actual token
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;

        let token = self.next_token()?;
        let end_pos = self.position;

        // Get lexeme from original input
        let lexeme = if start_pos < self.input.len() {
            self.input[start_pos..end_pos.min(self.input.len())]
                .iter()
                .collect()
        } else {
            String::new()
        };

        let category = self.get_token_category(&token);

        Ok(TokenInfo {
            token,
            category,
            lexeme,
            line: start_line,
            column: start_column,
        })
    }

    /// Get token category for output format
    fn get_token_category(&self, token: &Token) -> TokenCategory {
        match token {
            Token::Select
            | Token::From
            | Token::Where
            | Token::Insert
            | Token::Into
            | Token::Values
            | Token::Update
            | Token::Set
            | Token::Delete
            | Token::Create
            | Token::Table
            | Token::Drop
            | Token::Alter
            | Token::Index
            | Token::Primary
            | Token::Key
            | Token::Foreign
            | Token::References
            | Token::Not
            | Token::And
            | Token::Or
            | Token::In
            | Token::Like
            | Token::Between
            | Token::Is
            | Token::As
            | Token::Distinct
            | Token::Order
            | Token::By
            | Token::Group
            | Token::Having
            | Token::Limit
            | Token::Offset
            | Token::Join
            | Token::Inner
            | Token::Left
            | Token::Right
            | Token::Full
            | Token::Outer
            | Token::On
            | Token::Union
            | Token::All
            | Token::Exists
            | Token::Case
            | Token::When
            | Token::Then
            | Token::Else
            | Token::End
            | Token::If
            | Token::Int
            | Token::BigInt
            | Token::Float32
            | Token::Double
            | Token::Varchar
            | Token::Char
            | Token::Text
            | Token::Bool
            | Token::Date
            | Token::Timestamp => TokenCategory::Keyword,

            Token::Identifier(_) => TokenCategory::Identifier,
            Token::Integer(_) => TokenCategory::Integer,
            Token::Float(_) => TokenCategory::Float,
            Token::String(_) => TokenCategory::String,
            Token::Boolean(_) | Token::Null => TokenCategory::Keyword,

            Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::Modulo
            | Token::Equal
            | Token::NotEqual
            | Token::LessThan
            | Token::LessEqual
            | Token::GreaterThan
            | Token::GreaterEqual => TokenCategory::Operator,

            Token::LeftParen
            | Token::RightParen
            | Token::LeftBracket
            | Token::RightBracket
            | Token::Comma
            | Token::Semicolon
            | Token::Dot => TokenCategory::Delimiter,

            Token::Wildcard => TokenCategory::Operator,
            Token::EOF => TokenCategory::EOF,
        }
    }
}

impl TokenInfo {
    /// Format token info as [种别码，词素值，行号，列号]
    pub fn format_output(&self) -> String {
        format!(
            "[{}, {}, {}, {}]",
            self.category as u8, self.lexeme, self.line, self.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("SELECT FROM WHERE");
        assert_eq!(lexer.next_token().unwrap(), Token::Select);
        assert_eq!(lexer.next_token().unwrap(), Token::From);
        assert_eq!(lexer.next_token().unwrap(), Token::Where);
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("table_name column1 _private");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("table_name".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("column1".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("_private".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("123 45.67 0 999");
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Float(45.67));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(0));
        assert_eq!(lexer.next_token().unwrap(), Token::Integer(999));
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("'hello world' 'test\\nstring'");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("hello world".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::String("test\nstring".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / = <> != < <= > >=");
        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Minus);
        assert_eq!(lexer.next_token().unwrap(), Token::Multiply);
        assert_eq!(lexer.next_token().unwrap(), Token::Divide);
        assert_eq!(lexer.next_token().unwrap(), Token::Equal);
        assert_eq!(lexer.next_token().unwrap(), Token::NotEqual);
        assert_eq!(lexer.next_token().unwrap(), Token::NotEqual);
        assert_eq!(lexer.next_token().unwrap(), Token::LessThan);
        assert_eq!(lexer.next_token().unwrap(), Token::LessEqual);
        assert_eq!(lexer.next_token().unwrap(), Token::GreaterThan);
        assert_eq!(lexer.next_token().unwrap(), Token::GreaterEqual);
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_punctuation() {
        let mut lexer = Lexer::new("( ) [ ] , ; .");
        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Comma);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Dot);
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_sql_statement() {
        let mut lexer = Lexer::new("SELECT id, name FROM users WHERE age > 18;");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0], Token::Select);
        assert_eq!(tokens[1], Token::Identifier("id".to_string()));
        assert_eq!(tokens[2], Token::Comma);
        assert_eq!(tokens[3], Token::Identifier("name".to_string()));
        assert_eq!(tokens[4], Token::From);
        assert_eq!(tokens[5], Token::Identifier("users".to_string()));
        assert_eq!(tokens[6], Token::Where);
        assert_eq!(tokens[7], Token::Identifier("age".to_string()));
        assert_eq!(tokens[8], Token::GreaterThan);
        assert_eq!(tokens[9], Token::Integer(18));
        assert_eq!(tokens[10], Token::Semicolon);
        assert_eq!(tokens[11], Token::EOF);
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("SELECT -- this is a comment\nFROM /* block comment */ users");
        assert_eq!(lexer.next_token().unwrap(), Token::Select);
        assert_eq!(lexer.next_token().unwrap(), Token::From);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("users".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_token_info_format() {
        let mut lexer = Lexer::new("SELECT id FROM users");

        // Test SELECT keyword
        let token_info = lexer.next_token_info().unwrap();
        assert_eq!(token_info.token, Token::Select);
        assert_eq!(token_info.category, TokenCategory::Keyword);
        assert_eq!(token_info.line, 1);
        assert_eq!(token_info.column, 1);

        // Test identifier
        let token_info = lexer.next_token_info().unwrap();
        assert_eq!(token_info.token, Token::Identifier("id".to_string()));
        assert_eq!(token_info.category, TokenCategory::Identifier);

        // Test formatted output
        let formatted = token_info.format_output();
        assert!(formatted.starts_with("[2,"));
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("SELECT\nid");

        // SELECT on line 1
        let token_info = lexer.next_token_info().unwrap();
        assert_eq!(token_info.line, 1);
        assert_eq!(token_info.column, 1);

        // id on line 2
        let token_info = lexer.next_token_info().unwrap();
        assert_eq!(token_info.line, 2);
        assert_eq!(token_info.column, 1);
    }
}
