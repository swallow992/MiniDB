use crate::sql::lexer::Lexer;

#[test]
fn debug_position_tracking() {
    let mut lexer = Lexer::new("SELECT\nid");
    println!("Testing: 'SELECT\\nid'");
    
    let mut count = 0;
    loop {
        match lexer.next_token_info() {
            Ok(token_info) => {
                println!("Token {}: {:?} at line {}, col {}", count, token_info.token, token_info.line, token_info.column);
                count += 1;
                if matches!(token_info.token, crate::sql::lexer::Token::EOF) {
                    break;
                }
            },
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }
}
