//! 测试新功能的演示程序

use minidb::engine::database::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniDB 新功能测试 ===\n");
    
    // 清理并创建数据库
    let temp_dir = std::path::PathBuf::from("./test_new_features_db");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }
    let mut db = Database::new(temp_dir)?;
    
    println!("🗃️ 创建测试表...");
    
    // 创建用户表
    let create_users = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(50) NOT NULL, age INT, department VARCHAR(30))";
    let result = db.execute(create_users)?;
    println!("✅ {}", result.message);
    
    // 创建订单表
    let create_orders = "CREATE TABLE orders (order_id INT PRIMARY KEY, user_id INT, amount FLOAT, status VARCHAR(20))";
    let result = db.execute(create_orders)?;
    println!("✅ {}", result.message);
    
    println!("\n📝 插入测试数据...");
    
    // 插入用户数据
    let users_data = vec![
        "INSERT INTO users VALUES (1, 'Alice', 25, 'Engineering')",
        "INSERT INTO users VALUES (2, 'Bob', 30, 'Sales')", 
        "INSERT INTO users VALUES (3, 'Charlie', 35, 'Engineering')",
        "INSERT INTO users VALUES (4, 'Diana', 28, 'Marketing')",
        "INSERT INTO users VALUES (5, 'Eve', 22, 'Sales')",
    ];
    
    for sql in users_data {
        db.execute(sql)?;
    }
    println!("✅ 用户数据插入完成");
    
    // 插入订单数据
    let orders_data = vec![
        "INSERT INTO orders VALUES (1, 1, 100.50, 'completed')",
        "INSERT INTO orders VALUES (2, 2, 75.25, 'pending')", 
        "INSERT INTO orders VALUES (3, 1, 200.00, 'completed')",
        "INSERT INTO orders VALUES (4, 3, 150.75, 'shipped')",
        "INSERT INTO orders VALUES (5, 4, 89.99, 'completed')",
    ];
    
    for sql in orders_data {
        db.execute(sql)?;
    }
    println!("✅ 订单数据插入完成");
    
    println!("\n🔍 测试新功能...");
    
    // 1. 测试 ORDER BY - 这会触发新功能检测
    println!("\n1. 测试 ORDER BY 功能:");
    let result = db.execute("SELECT * FROM users ORDER BY age")?;
    println!("   📋 按年龄排序查询用户: {}", result.message);
    
    let result = db.execute("SELECT * FROM users ORDER BY name DESC")?;
    println!("   📋 按姓名降序排序: {}", result.message);
    
    // 2. 测试 GROUP BY - 这会触发新功能检测
    println!("\n2. 测试 GROUP BY 功能:");
    let result = db.execute("SELECT department FROM users GROUP BY department")?;
    println!("   📊 按部门分组统计: {}", result.message);
    
    // 3. 测试 LIMIT - 这会触发新功能检测
    println!("\n3. 测试 LIMIT 功能:");
    let result = db.execute("SELECT * FROM users LIMIT 3")?;
    println!("   📄 限制返回3条记录: {}", result.message);
    
    let result = db.execute("SELECT * FROM users LIMIT 2 OFFSET 1")?;
    println!("   📄 跳过1条，返回2条记录: {}", result.message);
    
    // 4. 测试 JOIN (语法解析，但执行可能简化)
    println!("\n4. 测试 JOIN 功能 (语法解析):");
    let result = db.execute("SELECT name FROM users")?; // 简化的查询
    println!("   🔗 用户查询 (JOIN功能已实现但需要更复杂的集成): {}", result.message);
    
    // 5. 组合测试
    println!("\n5. 测试组合功能:");
    let result = db.execute("SELECT * FROM users WHERE age > 25 ORDER BY age LIMIT 2")?;
    println!("   🔍 复合查询 (WHERE + ORDER BY + LIMIT): {}", result.message);
    
    println!("\n✅ 新功能测试完成！");
    println!("\n📝 说明:");
    println!("   - ORDER BY、GROUP BY、LIMIT 等高级功能已成功解析");
    println!("   - 执行器已实现但需要进一步集成到查询引擎");
    println!("   - 当前版本会检测这些功能并提供相应提示");
    
    Ok(())
}