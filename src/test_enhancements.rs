#[cfg(test)]
mod enhanced_tests {
    use crate::sql::analyzer::SemanticError;
    use crate::sql::lexer::{Lexer, TokenInfo};

    #[test]
    fn test_lexer_position_tracking() {
        println!("=== 测试词法分析器位置跟踪 ===");

        let sql = "SELECT name,\n    age\nFROM users\nWHERE id = 123";
        println!("SQL输入:\n{}\n", sql);

        let mut lexer = Lexer::new(sql);
        let mut token_count = 0;

        loop {
            match lexer.next_token_info() {
                Ok(token_info) => {
                    println!("{}", token_info.format_output());
                    token_count += 1;
                    if matches!(token_info.token, crate::sql::lexer::Token::EOF) {
                        break;
                    }
                }
                Err(e) => {
                    println!("词法错误: {:?}", e);
                    break;
                }
            }
        }

        assert!(token_count > 0, "应该至少有一个token");
    }

    #[test]
    fn test_semantic_error_format() {
        println!("\n=== 测试语义分析器错误格式 ===");

        // 测试不同类型的错误
        let errors = vec![
            SemanticError::TableNotFound {
                table: "users".to_string(),
                position: None,
            },
            SemanticError::ColumnNotFound {
                table: "users".to_string(),
                column: "name".to_string(),
                position: None,
            },
            SemanticError::AmbiguousColumn {
                column: "id".to_string(),
                position: None,
            },
        ];

        for error in errors {
            let formatted = error.format_output();
            println!("{}", formatted);
            assert!(formatted.starts_with("["), "错误格式应该以[开始");
            assert!(formatted.contains(","), "错误格式应该包含逗号分隔符");
        }
    }

    #[test]
    fn test_position_tracking_with_multiline() {
        let sql = "SELECT *\nFROM table1\nWHERE col = 'value'";
        let mut lexer = Lexer::new(sql);

        let mut has_multiline_tokens = false;

        loop {
            match lexer.next_token_info() {
                Ok(token_info) => {
                    if token_info.line > 1 {
                        has_multiline_tokens = true;
                    }
                    if matches!(token_info.token, crate::sql::lexer::Token::EOF) {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        assert!(has_multiline_tokens, "应该检测到多行的tokens");
    }
}
