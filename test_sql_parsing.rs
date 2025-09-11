//! Test SQL parsing directly

use minidb::sql::parse_sql;

fn main() {
    // Test simple SQL statements
    let sqls = vec![
        "CREATE TABLE users (id INT)",
        "CREATE TABLE users (id INT, name VARCHAR(255))",
        "DROP TABLE users",
        "INSERT INTO users VALUES (1, 'Alice')",
        "SELECT * FROM users",
    ];

    for sql in sqls {
        println!("Testing SQL: {}", sql);
        match parse_sql(sql) {
            Ok(stmt) => println!("  ✅ Parsed: {:?}", stmt),
            Err(e) => println!("  ❌ Error: {:?}", e),
        }
        println!();
    }
}
