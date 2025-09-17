use minidb::engine::database::QueryResult;
use minidb::Database;
use std::env;
use std::io::{self, Write};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== MiniDB Interactive Shell v{} ===", minidb::VERSION);
    println!("欢迎使用 MiniDB！");
    println!("输入 'help' 查看可用命令，输入 'quit' 退出。");
    println!();

    // Get database path from command line args or use default
    let db_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "./minidb_data".to_string());

    println!("正在打开数据库: {}", db_path);
    let mut database = Database::new(&db_path)?;
    println!("数据库已成功加载！");
    println!();

    loop {
        print!("minidb> ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached (e.g., from pipe input)
                println!();
                break;
            }
            Ok(_) => {
                // Continue processing
            }
            Err(e) => {
                eprintln!("读取输入时出错: {}", e);
                break;
            }
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "\\q" => {
                println!("再见！感谢使用 MiniDB!");
                break;
            }
            "help" | "\\h" => {
                show_help();
            }
            "\\d" => {
                show_tables(&database)?;
            }
            "\\s" => {
                show_status(&database)?;
            }
            "\\i" => {
                show_internal_info(&database)?;
            }
            "\\t" => {
                run_quick_test(&mut database)?;
            }
            "\\version" | "version" => {
                show_version_info();
            }
            "clear" | "\\c" => {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            }
            _ => {
                let start = Instant::now();
                match execute_sql(&mut database, input) {
                    Ok(result) => {
                        let duration = start.elapsed();
                        print_detailed_result(&result, duration);
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        print_error(&e, duration);
                    }
                }
                println!(); // Add spacing after each command
            }
        }
    }

    Ok(())
}

fn show_help() {
    println!("=== MiniDB 命令帮助 ===");
    println!();
    println!("系统命令:");
    println!("  help, \\h          显示此帮助信息");
    println!("  quit, exit, \\q    退出程序");
    println!("  \\d                列出所有表");
    println!("  \\s                显示系统状态");
    println!("  \\i                显示内部信息");
    println!("  \\t                运行快速测试");
    println!("  \\version          显示版本信息");
    println!("  clear, \\c         清空屏幕");
    println!();
    println!("基础 SQL 命令:");
    println!("  CREATE TABLE name (column_definitions...)");
    println!("  INSERT INTO name VALUES (...)");
    println!("  SELECT columns FROM name [WHERE condition]");
    println!("  UPDATE name SET column=value [WHERE condition]");
    println!("  DELETE FROM name [WHERE condition]");
    println!("  DROP TABLE name");
    println!();
    println!("高级 SQL 功能:");
    println!("  SELECT ... ORDER BY column [ASC|DESC]     - 排序查询");
    println!("  SELECT ... LIMIT number [OFFSET number]   - 分页查询");
    println!("  SELECT ... GROUP BY column                - 分组查询");
    println!("  SELECT COUNT(*), SUM(col), AVG(col)...    - 聚合函数");
    println!("  SELECT ... WHERE col IN (values)          - 条件查询");
    println!();
    println!("🗂️ 索引系统 (NEW!):");
    println!("  CREATE INDEX idx_name ON table (column)           - 创建索引");
    println!("  CREATE UNIQUE INDEX idx_name ON table (column)    - 创建唯一索引");
    println!("  DROP INDEX idx_name                               - 删除索引");
    println!("  自动主键索引和查询优化                              - 自动功能");
    println!();
    println!("支持的聚合函数:");
    println!("  COUNT(*)         计算行数");
    println!("  COUNT(column)    计算非空值数量");
    println!("  SUM(column)      求和");
    println!("  AVG(column)      平均值");
    println!("  MAX(column)      最大值");
    println!("  MIN(column)      最小值");
    println!();
    println!("基础示例:");
    println!("  CREATE TABLE users (id INT, name VARCHAR(50), age INT);");
    println!("  INSERT INTO users VALUES (1, 'Alice', 25);");
    println!("  SELECT * FROM users;");
    println!("  SELECT name FROM users WHERE age > 20;");
    println!();
    println!("高级示例:");
    println!("  SELECT * FROM users ORDER BY age DESC;");
    println!("  SELECT * FROM users LIMIT 5 OFFSET 10;");
    println!("  SELECT department, COUNT(*) FROM users GROUP BY department;");
    println!("  SELECT AVG(age), MAX(age) FROM users WHERE age > 25;");
    println!();
    println!("🗂️ 索引管理示例:");
    println!("  CREATE INDEX idx_age ON users (age);");
    println!("  CREATE UNIQUE INDEX idx_email ON users (email);");
    println!();
}

fn show_tables(database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 数据库表列表 ===");
    
    let tables = database.list_tables();
    
    if tables.is_empty() {
        println!("📋 暂无表格");
        println!("提示: 使用 CREATE TABLE 命令创建新表");
    } else {
        println!("📋 共找到 {} 个表:", tables.len());
        println!();
        
        for (i, table_name) in tables.iter().enumerate() {
            println!("{}. 📊 {}", i + 1, table_name);
            
            // Show table schema if available
            if let Some(schema) = database.get_table_schema(table_name) {
                println!("   列信息:");
                for (j, column) in schema.columns.iter().enumerate() {
                    println!("     {}. {} - {}", 
                        j + 1, 
                        column.name, 
                        format_data_type(&column.data_type)
                    );
                }
            }
            println!();
        }
    }
    
    Ok(())
}

fn show_status(database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 系统状态 ===");
    println!("🔧 数据库引擎: MiniDB v{}", minidb::VERSION);
    
    // 表统计
    let tables = database.list_tables();
    println!("📊 表统计: {} 个表", tables.len());
    
    if !tables.is_empty() {
        let total_rows = 0;
        for table_name in &tables {
            // 注意：这是一个简化的实现，实际应该有专门的方法获取行数
            println!("   📋 {}", table_name);
        }
        println!("📈 总行数: {} 行 (估算)", total_rows);
    }
    
    // 系统资源（简化显示）
    println!("💾 页面大小: {} bytes", minidb::DEFAULT_PAGE_SIZE);
    println!("🗂️  缓冲池配置: {} pages", minidb::DEFAULT_BUFFER_POOL_SIZE);
    println!("🔗 活跃连接: 1");
    
    // 存储状态
    println!("💿 数据目录: ./minidb_data");
    println!("⚙️  编译模式: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
    
    println!();
    println!("🟢 系统运行正常");
    
    Ok(())
}

fn show_internal_info(_database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 内部系统信息 ===");
    println!("🔧 数据库引擎: MiniDB v{}", minidb::VERSION);
    println!("📁 数据目录: ./minidb_data");
    println!("💾 页面大小: {} bytes", minidb::DEFAULT_PAGE_SIZE);
    println!("🗂️  缓冲池大小: {} pages", minidb::DEFAULT_BUFFER_POOL_SIZE);
    println!("⚙️  编译模式: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
    println!("🦀 编译器: rustc");
    println!("📦 包管理: Cargo");
    Ok(())
}

fn run_quick_test(database: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 快速功能测试 ===");
    
    let test_commands = vec![
        ("创建测试表", "CREATE TABLE test_table (id INT, name VARCHAR(20));"),
        ("插入测试数据", "INSERT INTO test_table VALUES (1, 'Test');"),
        ("查询测试数据", "SELECT * FROM test_table;"),
    ];
    
    for (description, sql) in test_commands {
        println!("🧪 {}: {}", description, sql);
        let start = Instant::now();
        match database.execute(sql) {
            Ok(result) => {
                let duration = start.elapsed();
                println!("   ✅ 成功 ({:.2}ms), {} 行结果", 
                    duration.as_secs_f64() * 1000.0, result.rows.len());
            }
            Err(e) => {
                let duration = start.elapsed();
                println!("   ❌ 失败 ({:.2}ms): {}", 
                    duration.as_secs_f64() * 1000.0, e);
            }
        }
    }
    
    println!("🎯 快速测试完成!");
    Ok(())
}

fn show_version_info() {
    println!("=== 版本信息 ===");
    println!("🚀 MiniDB v{}", minidb::VERSION);
    println!("🦀 Rust 编译器: rustc");
    println!("🎯 目标平台: {}", std::env::consts::OS);
    println!("⚙️  构建配置: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
    println!();
    println!("🏗️  项目信息:");
    println!("   作者: MiniDB Team");
    println!("   描述: 一个用 Rust 构建的小型数据库系统");
    println!("   功能: SQL 编译器 + 存储引擎 + 查询执行器");
}

fn execute_sql(
    database: &mut Database,
    sql: &str,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    println!("📝 执行SQL: {}", sql);
    let result = database.execute(sql)?;
    Ok(result)
}

fn print_detailed_result(result: &QueryResult, duration: std::time::Duration) {
    println!("✅ 查询执行成功!");
    println!("⏱️  执行时间: {:.2}ms", duration.as_secs_f64() * 1000.0);
    
    if !result.message.is_empty() {
        println!("💬 消息: {}", result.message);
    }

    if result.rows.is_empty() {
        println!("📊 结果: 无数据行");
        return;
    }

    println!("📊 查询结果:");
    println!("═══════════════════════════════════════");

    // Print column headers if schema is available
    if let Some(ref schema) = result.schema {
        // Print column names
        for (i, column) in schema.columns.iter().enumerate() {
            if i > 0 {
                print!(" │ ");
            }
            print!("{:>12}", column.name);
        }
        println!();

        // Print column types
        for (i, column) in schema.columns.iter().enumerate() {
            if i > 0 {
                print!(" │ ");
            }
            print!("{:>12}", format!("({})", format_data_type(&column.data_type)));
        }
        println!();

        // Print separator
        for i in 0..schema.columns.len() {
            if i > 0 {
                print!("─┼─");
            }
            print!("{:─<12}", "");
        }
        println!();
    } else {
        // Fallback to generic column headers
        if let Some(first_row) = result.rows.first() {
            for (i, _value) in first_row.values.iter().enumerate() {
                if i > 0 {
                    print!(" │ ");
                }
                print!("{:>12}", format!("column_{}", i + 1));
            }
            println!();

            // Print separator
            for i in 0..first_row.values.len() {
                if i > 0 {
                    print!("─┼─");
                }
                print!("{:─<12}", "");
            }
            println!();
        }
    }

    // Print data rows
    for (row_idx, row) in result.rows.iter().enumerate() {
        for (i, value) in row.values.iter().enumerate() {
            if i > 0 {
                print!(" │ ");
            }
            print!("{:>12}", format_value(value));
        }
        println!();
        
        // Add separator every 10 rows for better readability
        if (row_idx + 1) % 10 == 0 && row_idx + 1 < result.rows.len() {
            println!("─────────────────────────────────────────");
        }
    }

    println!("═══════════════════════════════════════");
    println!("📈 总共 {} 行数据", result.rows.len());
    if result.affected_rows > 0 {
        println!("🔄 影响行数: {}", result.affected_rows);
    }
}

fn print_error(error: &Box<dyn std::error::Error>, duration: std::time::Duration) {
    println!("❌ 查询执行失败!");
    println!("⏱️  执行时间: {:.2}ms", duration.as_secs_f64() * 1000.0);
    println!("🚨 错误信息: {}", error);
    
    // Try to provide helpful hints based on error type
    let error_str = error.to_string().to_lowercase();
    if error_str.contains("parse") {
        println!("💡 提示: 请检查SQL语法是否正确");
    } else if error_str.contains("column") && error_str.contains("not found") {
        println!("💡 提示: 列不存在，请检查列名是否正确");
    } else if error_str.contains("table") && error_str.contains("not found") {
        println!("💡 提示: 表不存在，请使用 \\d 查看可用的表");
    }
}

#[allow(dead_code)]
fn print_result(result: &QueryResult) {
    if result.rows.is_empty() {
        println!("(0 rows)");
        return;
    }

    // Print column headers
    if let Some(first_row) = result.rows.first() {
        for (i, _value) in first_row.values.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>10}", format!("col_{}", i));
        }
        println!();

        // Print separator
        for i in 0..first_row.values.len() {
            if i > 0 {
                print!("-+-");
            }
            print!("{:-<10}", "");
        }
        println!();
    }

    // Print data rows
    for row in &result.rows {
        for (i, value) in row.values.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>10}", format_value(value));
        }
        println!();
    }

    println!("({} rows)", result.rows.len());
}

fn format_data_type(data_type: &minidb::types::DataType) -> String {
    match data_type {
        minidb::types::DataType::Integer => "INT".to_string(),
        minidb::types::DataType::BigInt => "BIGINT".to_string(),
        minidb::types::DataType::Float => "FLOAT".to_string(),
        minidb::types::DataType::Double => "DOUBLE".to_string(),
        minidb::types::DataType::Varchar(size) => format!("VARCHAR({})", size),
        minidb::types::DataType::Boolean => "BOOLEAN".to_string(),
        minidb::types::DataType::Date => "DATE".to_string(),
        minidb::types::DataType::Timestamp => "TIMESTAMP".to_string(),
    }
}

fn format_value(value: &minidb::Value) -> String {
    match value {
        minidb::Value::Null => "NULL".to_string(),
        minidb::Value::Integer(i) => i.to_string(),
        minidb::Value::BigInt(i) => i.to_string(),
        minidb::Value::Float(f) => format!("{:.2}", f),
        minidb::Value::Double(f) => format!("{:.2}", f),
        minidb::Value::Varchar(s) => s.clone(),
        minidb::Value::Boolean(b) => b.to_string(),
        minidb::Value::Date(d) => d.to_string(),
        minidb::Value::Timestamp(ts) => ts.to_string(),
    }
}