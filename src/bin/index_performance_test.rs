//! 索引性能测试
//!
//! 比较有索引和无索引时的查询性能差异

use minidb::engine::database::Database;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniDB 索引性能测试 ===\n");

    let mut db = Database::new("performance_test_db")?;

    // 1. 创建测试表
    println!("1. 创建测试表...");
    let create_table_sql = r#"
        CREATE TABLE performance_test (
            id INTEGER NOT NULL,
            category TEXT NOT NULL,
            value INTEGER,
            description TEXT
        )
    "#;
    
    execute_sql(&mut db, create_table_sql)?;
    println!("✓ 表创建成功");

    // 2. 插入大量测试数据
    println!("\n2. 插入测试数据...");
    for i in 1..=1000 {
        let insert_sql = format!(
            "INSERT INTO performance_test (id, category, value, description) VALUES ({}, 'cat{}', {}, 'desc{}')",
            i, i % 10, i * 10, i
        );
        execute_sql(&mut db, &insert_sql)?;
        
        if i % 200 == 0 {
            println!("  已插入 {} 条记录", i);
        }
    }
    println!("✓ 测试数据插入完成 (1000条记录)");

    // 3. 无索引查询性能测试
    println!("\n3. 无索引查询性能测试...");
    let search_queries = vec![
        "SELECT * FROM performance_test WHERE id = 500",
        "SELECT * FROM performance_test WHERE category = 'cat5'",
        "SELECT * FROM performance_test WHERE value > 5000",
    ];

    let mut no_index_times = Vec::new();
    for query in &search_queries {
        let start = Instant::now();
        execute_sql(&mut db, query)?;
        let duration = start.elapsed();
        no_index_times.push(duration);
        println!("  查询: {} - 用时: {:?}", query, duration);
    }

    // 4. 创建索引
    println!("\n4. 创建索引...");
    let index_sqls = vec![
        "CREATE INDEX idx_id ON performance_test (id)",
        "CREATE INDEX idx_category ON performance_test (category)",
        "CREATE INDEX idx_value ON performance_test (value)",
    ];

    for sql in index_sqls {
        execute_sql(&mut db, sql)?;
    }
    println!("✓ 索引创建完成");

    // 5. 有索引查询性能测试
    println!("\n5. 有索引查询性能测试...");
    let mut with_index_times = Vec::new();
    for query in &search_queries {
        let start = Instant::now();
        execute_sql(&mut db, query)?;
        let duration = start.elapsed();
        with_index_times.push(duration);
        println!("  查询: {} - 用时: {:?}", query, duration);
    }

    // 6. 性能对比
    println!("\n6. 性能对比结果:");
    println!("┌─────────────────────────────────────────────────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ 查询类型                                                │ 无索引耗时   │ 有索引耗时   │ 性能提升    │");
    println!("├─────────────────────────────────────────────────────────┼─────────────┼─────────────┼─────────────┤");
    
    for i in 0..search_queries.len() {
        let no_idx_ms = no_index_times[i].as_millis();
        let with_idx_ms = with_index_times[i].as_millis();
        let improvement = if with_idx_ms > 0 {
            format!("{:.1}x", no_idx_ms as f64 / with_idx_ms as f64)
        } else {
            "N/A".to_string()
        };
        
        println!("│ {:55} │ {:>9}ms │ {:>9}ms │ {:>9} │", 
                 &search_queries[i][0..55.min(search_queries[i].len())], 
                 no_idx_ms, 
                 with_idx_ms, 
                 improvement);
    }
    println!("└─────────────────────────────────────────────────────────┴─────────────┴─────────────┴─────────────┘");

    // 7. 测试EXPLAIN查看执行计划
    println!("\n7. 查看执行计划...");
    let explain_queries = vec![
        "EXPLAIN SELECT * FROM performance_test WHERE id = 500",
        "EXPLAIN SELECT * FROM performance_test WHERE category = 'cat5'",
    ];

    for query in explain_queries {
        println!("\n查询: {}", query);
        execute_sql(&mut db, query)?;
    }

    println!("\n=== 索引性能测试完成 ===");
    Ok(())
}

fn execute_sql(db: &mut Database, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = db.execute(sql)?;
    
    // 只在非测试查询时显示详细信息
    if !sql.contains("performance_test WHERE") || sql.starts_with("EXPLAIN") {
        if !result.message.is_empty() {
            println!("  → {}", result.message);
        }
        
        if sql.starts_with("EXPLAIN") && !result.rows.is_empty() {
            for row in &result.rows {
                for value in &row.values {
                    if let minidb::types::Value::Varchar(plan) = value {
                        println!("  {}", plan);
                    }
                }
            }
        }
    }
    
    Ok(())
}