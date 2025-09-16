//! 索引性能对比测试
//!
//! 通过创建和删除索引来比较查询性能差异

use minidb::engine::database::Database;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniDB 索引性能对比测试 ===\n");

    let mut db = Database::new("index_performance_comparison")?;

    // 1. 创建测试表和数据
    setup_test_data(&mut db)?;

    // 2. 无索引性能测试
    println!("\n📊 第一轮：无索引查询性能测试");
    let no_index_results = run_performance_tests(&mut db, "无索引")?;

    // 3. 创建索引
    println!("\n🔧 创建索引...");
    create_indexes(&mut db)?;

    // 4. 有索引性能测试
    println!("\n📊 第二轮：有索引查询性能测试");
    let with_index_results = run_performance_tests(&mut db, "有索引")?;

    // 5. 删除索引
    println!("\n🗑️ 删除索引...");
    drop_indexes(&mut db)?;

    // 6. 再次无索引性能测试
    println!("\n📊 第三轮：删除索引后查询性能测试");
    let after_drop_results = run_performance_tests(&mut db, "删除索引后")?;

    // 7. 性能对比分析
    println!("\n📈 性能对比分析:");
    compare_performance_results(&no_index_results, &with_index_results, &after_drop_results)?;

    println!("\n✅ 索引性能对比测试完成");
    Ok(())
}

fn setup_test_data(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️ 设置测试环境...");
    
    // 创建表
    execute_sql(db, r#"
        CREATE TABLE products (
            id INTEGER NOT NULL,
            category TEXT NOT NULL,
            name TEXT NOT NULL,
            price INTEGER,
            stock INTEGER,
            supplier_id INTEGER
        )
    "#)?;
    
    println!("  ✓ 表创建完成");

    // 插入测试数据
    println!("  📝 插入测试数据...");
    for i in 1..=2000 {
        let category = match i % 5 {
            0 => "Electronics",
            1 => "Clothing",
            2 => "Books",
            3 => "Home",
            _ => "Sports",
        };
        
        execute_sql(db, &format!(
            "INSERT INTO products (id, category, name, price, stock, supplier_id) VALUES ({}, '{}', 'Product {}', {}, {}, {})",
            i, category, i, (i * 10) % 1000 + 50, i % 100 + 1, (i % 20) + 1
        ))?;
        
        if i % 500 == 0 {
            println!("    已插入 {} 条记录", i);
        }
    }
    
    println!("  ✓ 测试数据准备完成 (2000 条记录)");
    Ok(())
}

fn create_indexes(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    let indexes = vec![
        "CREATE INDEX idx_products_id ON products (id)",
        "CREATE INDEX idx_products_category ON products (category)",
        "CREATE INDEX idx_products_price ON products (price)",
        "CREATE INDEX idx_products_supplier ON products (supplier_id)",
        "CREATE INDEX idx_products_cat_price ON products (category, price)",
    ];

    for (i, sql) in indexes.iter().enumerate() {
        execute_sql(db, sql)?;
        println!("  ✓ 索引 {} 创建完成", i + 1);
    }
    
    Ok(())
}

fn drop_indexes(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    let drop_sqls = vec![
        "DROP INDEX idx_products_id ON products",
        "DROP INDEX idx_products_category ON products",
        "DROP INDEX idx_products_price ON products", 
        "DROP INDEX idx_products_supplier ON products",
        "DROP INDEX idx_products_cat_price ON products",
    ];

    for (i, sql) in drop_sqls.iter().enumerate() {
        execute_sql(db, sql)?;
        println!("  ✓ 索引 {} 删除完成", i + 1);
    }
    
    Ok(())
}

#[derive(Debug)]
struct QueryResult {
    query: String,
    duration_ms: f64,
    rows_returned: usize,
}

fn run_performance_tests(db: &mut Database, test_name: &str) -> Result<Vec<QueryResult>, Box<dyn std::error::Error>> {
    let test_queries = vec![
        ("单值查询", "SELECT * FROM products WHERE id = 1000"),
        ("范围查询", "SELECT * FROM products WHERE price BETWEEN 200 AND 300"),
        ("分类查询", "SELECT * FROM products WHERE category = 'Electronics'"),
        ("复合条件", "SELECT * FROM products WHERE category = 'Books' AND price > 100"),
        ("供应商查询", "SELECT * FROM products WHERE supplier_id = 5"),
        ("库存查询", "SELECT * FROM products WHERE stock > 50"),
        ("价格排序", "SELECT * FROM products WHERE price > 500 ORDER BY price"),
    ];

    let mut results = Vec::new();
    
    println!("  执行 {} 查询测试:", test_name);
    
    for (test_desc, query) in test_queries {
        // 预热查询
        let _ = db.execute(query);
        
        // 执行多次取平均值
        let mut durations = Vec::new();
        let mut rows_count = 0;
        
        for _ in 0..5 {
            let start = Instant::now();
            let result = db.execute(query)?;
            let duration = start.elapsed();
            
            durations.push(duration.as_secs_f64() * 1000.0); // 转换为毫秒
            rows_count = result.rows.len();
        }
        
        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
        
        println!("    {} - {:.3}ms (返回 {} 行)", test_desc, avg_duration, rows_count);
        
        results.push(QueryResult {
            query: query.to_string(),
            duration_ms: avg_duration,
            rows_returned: rows_count,
        });
    }
    
    Ok(results)
}

fn compare_performance_results(
    no_index: &[QueryResult],
    with_index: &[QueryResult], 
    after_drop: &[QueryResult]
) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────────────────────┐");
    println!("│                                    性能对比结果                                            │");
    println!("├─────────────────────────┬─────────────┬─────────────┬─────────────┬─────────────┬─────────────┤");
    println!("│ 查询类型                │ 无索引(ms)  │ 有索引(ms)  │ 删除后(ms)  │ 索引加速比  │ 一致性验证  │");
    println!("├─────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤");
    
    for i in 0..no_index.len().min(with_index.len()).min(after_drop.len()) {
        let query_name = match i {
            0 => "单值查询",
            1 => "范围查询", 
            2 => "分类查询",
            3 => "复合条件",
            4 => "供应商查询",
            5 => "库存查询",
            6 => "价格排序",
            _ => "其他查询",
        };
        
        let no_idx_time = no_index[i].duration_ms;
        let with_idx_time = with_index[i].duration_ms;
        let after_drop_time = after_drop[i].duration_ms;
        
        let speedup = if with_idx_time > 0.0 {
            no_idx_time / with_idx_time
        } else {
            1.0
        };
        
        // 验证删除索引后性能是否恢复到原始水平
        let consistency = if (no_idx_time - after_drop_time).abs() / no_idx_time < 0.3 {
            "✓ 一致"
        } else {
            "⚠ 差异"
        };
        
        println!("│ {:23} │ {:>9.3} │ {:>9.3} │ {:>9.3} │ {:>9.2}x │ {:>9} │",
                 query_name, no_idx_time, with_idx_time, after_drop_time, speedup, consistency);
    }
    
    println!("└─────────────────────────┴─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘");
    
    // 计算总体统计
    let avg_no_index: f64 = no_index.iter().map(|r| r.duration_ms).sum::<f64>() / no_index.len() as f64;
    let avg_with_index: f64 = with_index.iter().map(|r| r.duration_ms).sum::<f64>() / with_index.len() as f64;
    let avg_after_drop: f64 = after_drop.iter().map(|r| r.duration_ms).sum::<f64>() / after_drop.len() as f64;
    
    println!();
    println!("📊 总体性能统计:");
    println!("  • 平均无索引查询时间: {:.3}ms", avg_no_index);
    println!("  • 平均有索引查询时间: {:.3}ms", avg_with_index);
    println!("  • 删除索引后平均时间: {:.3}ms", avg_after_drop);
    println!("  • 索引整体加速比: {:.2}x", avg_no_index / avg_with_index);
    
    let consistency_ratio = if (avg_no_index - avg_after_drop).abs() / avg_no_index < 0.2 {
        "✓ 高度一致"
    } else if (avg_no_index - avg_after_drop).abs() / avg_no_index < 0.5 {
        "~ 基本一致"
    } else {
        "⚠ 存在差异"
    };
    println!("  • 性能一致性验证: {}", consistency_ratio);
    
    Ok(())
}

fn execute_sql(db: &mut Database, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _result = db.execute(sql.trim())?;
    Ok(())
}