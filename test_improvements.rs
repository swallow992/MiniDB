use minidb::sql::analyzer::SemanticError;
use minidb::sql::lexer::{Lexer, TokenInfo};

fn test_lexer_position() {
    println!("=== 测试词法分析器位置跟踪 ===");

    let sql = "SELECT name,\n    age\nFROM users\nWHERE id = 123";
    println!("SQL输入:\n{}\n", sql);

    let mut lexer = Lexer::new(sql);
    loop {
        match lexer.next_token_info() {
            Ok(token_info) => {
                println!("{}", token_info.format_output());
                if matches!(token_info.token, minidb::sql::lexer::Token::EOF) {
                    break;
                }
            }
            Err(e) => {
                println!("词法错误: {:?}", e);
                break;
            }
        }
    }
}

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
        println!("{}", error.format_output());
    }
}

fn main() {
    test_lexer_position();
    test_semantic_error_format();
}
