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
        let mut lexer = Lexer::new("SELECT -- this is a comment\nFROM /* block comment */ table");
        assert_eq!(lexer.next_token().unwrap(), Token::Select);
        assert_eq!(lexer.next_token().unwrap(), Token::From);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("table".to_string())
        );
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }
}
