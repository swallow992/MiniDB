//! 测试CREATE INDEX功能
//!
//! 测试索引创建、删除和EXPLAIN功能

use minidb::engine::database::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniDB 索引功能测试 ===\n");

    let mut db = Database::new("test_index_db")?;

    // 1. 创建测试表
    println!("1. 创建测试表...");
    let create_table_sql = r#"
        CREATE TABLE users (
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            email TEXT,
            age INTEGER
        )
    "#;
    
    execute_sql(&mut db, create_table_sql)?;
    println!("✓ 表 users 创建成功");

    // 2. 插入测试数据
    println!("\n2. 插入测试数据...");
    let insert_sqls = vec![
        "INSERT INTO users (id, name, email, age) VALUES (1, 'Alice', 'alice@example.com', 25)",
        "INSERT INTO users (id, name, email, age) VALUES (2, 'Bob', 'bob@example.com', 30)",
        "INSERT INTO users (id, name, email, age) VALUES (3, 'Charlie', 'charlie@example.com', 28)",
    ];

    for sql in insert_sqls {
        execute_sql(&mut db, sql)?;
    }
    println!("✓ 测试数据插入成功");

    // 3. 测试CREATE INDEX
    println!("\n3. 测试CREATE INDEX...");
    let index_sqls = vec![
        "CREATE INDEX idx_users_id ON users (id)",
        "CREATE UNIQUE INDEX idx_users_email ON users (email)",
        "CREATE INDEX idx_users_name_age ON users (name, age)",
    ];

    for sql in index_sqls {
        println!("执行: {}", sql);
        execute_sql(&mut db, sql)?;
    }
    println!("✓ 索引创建成功");

    // 4. 测试EXPLAIN
    println!("\n4. 测试EXPLAIN...");
    let explain_sql = "EXPLAIN SELECT * FROM users WHERE id = 1";
    println!("执行: {}", explain_sql);
    execute_sql(&mut db, explain_sql)?;

    // 5. 测试DROP INDEX
    println!("\n5. 测试DROP INDEX...");
    let drop_index_sql = "DROP INDEX idx_users_name_age ON users";
    println!("执行: {}", drop_index_sql);
    execute_sql(&mut db, drop_index_sql)?;
    println!("✓ 索引删除成功");

    // 6. 测试带IF EXISTS的DROP INDEX
    println!("\n6. 测试DROP INDEX IF EXISTS...");
    let drop_index_if_exists_sql = "DROP INDEX IF EXISTS idx_nonexistent ON users";
    println!("执行: {}", drop_index_if_exists_sql);
    execute_sql(&mut db, drop_index_if_exists_sql)?;
    println!("✓ 条件删除成功");

    println!("\n=== 索引功能测试完成 ===");
    Ok(())
}

fn execute_sql(db: &mut Database, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 执行SQL
    let result = db.execute(sql)?;
    
    // 显示结果
    if !result.message.is_empty() {
        println!("  → {}", result.message);
    }
    
    if !result.rows.is_empty() {
        println!("  → 返回 {} 行数据", result.rows.len());
        
        // 如果有schema，打印列标题
        if let Some(schema) = &result.schema {
            print!("    ");
            for col in &schema.columns {
                print!("{:<15} ", col.name);
            }
            println!();
        }
        
        // 打印数据行
        for row in result.rows.iter().take(3) { // 只显示前3行
            print!("    ");
            for value in &row.values {
                print!("{:<15} ", format!("{:?}", value));
            }
            println!();
        }
        
        if result.rows.len() > 3 {
            println!("    ... 还有 {} 行", result.rows.len() - 3);
        }
    }
    
    if result.affected_rows > 0 && result.rows.is_empty() {
        println!("  → 影响了 {} 行", result.affected_rows);
    }

    Ok(())
}