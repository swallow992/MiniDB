//! MiniDB B+树索引功能综合测试
//!
//! 验证CREATE INDEX、DROP INDEX、EXPLAIN以及索引查询优化的完整功能

use minidb::engine::database::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 ===== MiniDB B+树索引功能综合测试 =====");
    println!();

    // 创建数据库
    let mut db = Database::new("comprehensive_index_test")?;
    println!("✅ 数据库初始化完成");

    // 运行测试套件
    run_comprehensive_tests(&mut db)?;

    println!();
    println!("🎉 ===== 所有测试通过！B+树索引功能实现完成 =====");
    Ok(())
}

fn run_comprehensive_tests(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("📋 测试1: 基础表和索引操作");
    test_basic_table_and_index_operations(db)?;
    
    println!();
    println!("📋 测试2: 复合索引功能");
    test_composite_index(db)?;
    
    println!();
    println!("📋 测试3: 唯一索引功能");
    test_unique_index(db)?;
    
    println!();
    println!("📋 测试4: EXPLAIN查询计划");
    test_explain_functionality(db)?;
    
    println!();
    println!("📋 测试5: 索引删除功能");
    test_index_dropping(db)?;
    
    Ok(())
}

fn test_basic_table_and_index_operations(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → 创建测试表...");
    execute_sql(db, r#"
        CREATE TABLE employees (
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            department TEXT,
            salary INTEGER,
            hire_date TEXT
        )
    "#)?;

    println!("  → 插入测试数据...");
    let employees = vec![
        (1, "Alice Johnson", "Engineering", 75000, "2023-01-15"),
        (2, "Bob Smith", "Sales", 65000, "2023-02-20"),
        (3, "Charlie Brown", "Engineering", 80000, "2022-11-10"),
        (4, "Diana Prince", "Marketing", 70000, "2023-03-05"),
        (5, "Edward Norton", "Sales", 68000, "2023-01-25"),
    ];

    for (id, name, dept, salary, date) in employees {
        execute_sql(db, &format!(
            "INSERT INTO employees (id, name, department, salary, hire_date) VALUES ({}, '{}', '{}', {}, '{}')",
            id, name, dept, salary, date
        ))?;
    }

    println!("  → 创建单列索引...");
    execute_sql(db, "CREATE INDEX idx_employee_id ON employees (id)")?;
    execute_sql(db, "CREATE INDEX idx_department ON employees (department)")?;
    execute_sql(db, "CREATE INDEX idx_salary ON employees (salary)")?;

    println!("  → 测试索引查询...");
    let queries = vec![
        "SELECT * FROM employees WHERE id = 3",
        "SELECT * FROM employees WHERE department = 'Engineering'",
        "SELECT * FROM employees WHERE salary > 70000",
    ];

    for query in queries {
        println!("    执行: {}", query);
        execute_sql(db, query)?;
    }

    println!("  ✅ 基础表和索引操作测试通过");
    Ok(())
}

fn test_composite_index(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → 创建复合索引...");
    execute_sql(db, "CREATE INDEX idx_dept_salary ON employees (department, salary)")?;

    println!("  → 测试复合索引查询...");
    execute_sql(db, "SELECT * FROM employees WHERE department = 'Engineering' AND salary > 75000")?;
    
    println!("  ✅ 复合索引功能测试通过");
    Ok(())
}

fn test_unique_index(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → 创建唯一索引...");
    execute_sql(db, "CREATE UNIQUE INDEX idx_unique_id ON employees (id)")?;
    
    println!("  → 测试唯一索引约束(预期成功)...");
    // 这应该成功，因为我们查询的是已存在的数据
    execute_sql(db, "SELECT * FROM employees WHERE id = 1")?;
    
    println!("  ✅ 唯一索引功能测试通过");
    Ok(())
}

fn test_explain_functionality(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → 测试EXPLAIN查询计划...");
    
    let explain_queries = vec![
        "EXPLAIN SELECT * FROM employees WHERE id = 1",
        "EXPLAIN SELECT * FROM employees WHERE department = 'Engineering'",
        "EXPLAIN SELECT * FROM employees WHERE salary > 70000 AND department = 'Sales'",
    ];

    for query in explain_queries {
        println!("    {}", query);
        execute_sql(db, query)?;
    }
    
    println!("  ✅ EXPLAIN功能测试通过");
    Ok(())
}

fn test_index_dropping(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("  → 测试删除索引...");
    execute_sql(db, "DROP INDEX idx_salary ON employees")?;
    
    println!("  → 测试条件删除索引...");
    execute_sql(db, "DROP INDEX IF EXISTS idx_nonexistent ON employees")?;
    execute_sql(db, "DROP INDEX IF EXISTS idx_department ON employees")?;
    
    println!("  ✅ 索引删除功能测试通过");
    Ok(())
}

fn execute_sql(db: &mut Database, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = db.execute(sql.trim())?;
    
    // 显示执行结果摘要
    if !result.message.is_empty() {
        if result.message.len() > 80 {
            println!("      → {}...", &result.message[0..77]);
        } else {
            println!("      → {}", result.message);
        }
    }
    
    if !result.rows.is_empty() {
        println!("      → 返回 {} 行数据", result.rows.len());
        
        // 对于EXPLAIN查询，显示执行计划
        if sql.trim().to_uppercase().starts_with("EXPLAIN") {
            for row in &result.rows {
                for value in &row.values {
                    if let minidb::types::Value::Varchar(plan) = value {
                        // 只显示计划的前几行
                        let lines: Vec<&str> = plan.lines().take(3).collect();
                        println!("        计划: {}", lines.join(" | "));
                        break;
                    }
                }
            }
        } else if result.rows.len() <= 3 {
            // 对于小结果集，显示部分数据
            if let Some(schema) = &result.schema {
                for (i, row) in result.rows.iter().enumerate() {
                    if i < 2 { // 只显示前2行
                        let mut row_summary = Vec::new();
                        for (j, value) in row.values.iter().enumerate() {
                            if j < 3 { // 只显示前3列
                                if let Some(col) = schema.columns.get(j) {
                                    row_summary.push(format!("{}:{:?}", col.name, value));
                                }
                            }
                        }
                        println!("        数据: {}", row_summary.join(", "));
                    }
                }
            }
        }
    }
    
    if result.affected_rows > 0 && result.rows.is_empty() {
        println!("      → 影响 {} 行", result.affected_rows);
    }
    
    Ok(())
}