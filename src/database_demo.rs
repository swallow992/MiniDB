use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

// 基础数据类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    String,
    Boolean,
    Float,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
    Boolean(bool),
    Float(f64),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "'{}'", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Float(fl) => write!(f, "{:.2}", fl),
            Value::Null => write!(f, "NULL"),
        }
    }
}

// 表模式定义
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    pub fn find_column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }
}

// 元组（记录）定义
#[derive(Debug, Clone)]
pub struct Tuple {
    pub values: Vec<Value>,
}

impl Tuple {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

// 表定义
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub schema: Schema,
    pub tuples: Vec<Tuple>,
}

impl Table {
    pub fn new(name: String, schema: Schema) -> Self {
        Self {
            name,
            schema,
            tuples: Vec::new(),
        }
    }

    pub fn insert(&mut self, tuple: Tuple) -> Result<(), String> {
        if tuple.values.len() != self.schema.columns.len() {
            return Err(format!(
                "Column count mismatch: expected {}, got {}",
                self.schema.columns.len(),
                tuple.values.len()
            ));
        }

        // 类型检查
        for (i, value) in tuple.values.iter().enumerate() {
            let column = &self.schema.columns[i];
            if !self.is_compatible_type(value, &column.data_type) {
                return Err(format!(
                    "Type mismatch for column '{}': expected {:?}, got {:?}",
                    column.name, column.data_type, value
                ));
            }
        }

        self.tuples.push(tuple);
        Ok(())
    }

    fn is_compatible_type(&self, value: &Value, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (Value::Null, _) => true, // NULL可以赋给任何类型
            (Value::Integer(_), DataType::Integer) => true,
            (Value::String(_), DataType::String) => true,
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Float(_), DataType::Float) => true,
            _ => false,
        }
    }
}

// 执行器操作符定义
pub trait Operator {
    fn next(&mut self) -> Option<Tuple>;
    fn reset(&mut self);
}

// CreateTable操作符
pub struct CreateTableOperator {
    pub table_name: String,
    pub schema: Schema,
    executed: bool,
}

impl CreateTableOperator {
    pub fn new(table_name: String, schema: Schema) -> Self {
        Self {
            table_name,
            schema,
            executed: false,
        }
    }

    pub fn execute(&mut self, database: &mut Database) -> Result<(), String> {
        if self.executed {
            return Err("CreateTable already executed".to_string());
        }

        database.create_table(self.table_name.clone(), self.schema.clone())?;
        self.executed = true;
        println!("🗃️ CREATE TABLE: {} 创建成功", self.table_name);
        Ok(())
    }
}

// Insert操作符
pub struct InsertOperator {
    pub table_name: String,
    pub tuple: Tuple,
    executed: bool,
}

impl InsertOperator {
    pub fn new(table_name: String, tuple: Tuple) -> Self {
        Self {
            table_name,
            tuple,
            executed: false,
        }
    }

    pub fn execute(&mut self, database: &mut Database) -> Result<(), String> {
        if self.executed {
            return Err("Insert already executed".to_string());
        }

        database.insert_tuple(&self.table_name, self.tuple.clone())?;
        self.executed = true;
        println!("📝 INSERT: 插入记录到表 {}", self.table_name);
        Ok(())
    }
}

// SeqScan操作符（顺序扫描）
pub struct SeqScanOperator {
    table: Table,
    current_index: usize,
}

impl SeqScanOperator {
    pub fn new(table: Table) -> Self {
        Self {
            table,
            current_index: 0,
        }
    }
}

impl Operator for SeqScanOperator {
    fn next(&mut self) -> Option<Tuple> {
        if self.current_index < self.table.tuples.len() {
            let tuple = self.table.tuples[self.current_index].clone();
            self.current_index += 1;
            Some(tuple)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.current_index = 0;
    }
}

// Filter操作符（过滤）
pub struct FilterOperator {
    child: Box<dyn Operator>,
    predicate: Box<dyn Fn(&Tuple) -> bool>,
}

impl FilterOperator {
    pub fn new(child: Box<dyn Operator>, predicate: Box<dyn Fn(&Tuple) -> bool>) -> Self {
        Self { child, predicate }
    }
}

impl Operator for FilterOperator {
    fn next(&mut self) -> Option<Tuple> {
        while let Some(tuple) = self.child.next() {
            if (self.predicate)(&tuple) {
                return Some(tuple);
            }
        }
        None
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}

// Project操作符（投影）
pub struct ProjectOperator {
    child: Box<dyn Operator>,
    column_indices: Vec<usize>,
}

impl ProjectOperator {
    pub fn new(child: Box<dyn Operator>, column_indices: Vec<usize>) -> Self {
        Self {
            child,
            column_indices,
        }
    }
}

impl Operator for ProjectOperator {
    fn next(&mut self) -> Option<Tuple> {
        if let Some(tuple) = self.child.next() {
            let projected_values: Vec<Value> = self
                .column_indices
                .iter()
                .filter_map(|&index| tuple.get_value(index).cloned())
                .collect();
            Some(Tuple::new(projected_values))
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}

// 数据库系统
pub struct Database {
    tables: HashMap<String, Table>,
    transaction_log: Vec<String>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            transaction_log: Vec::new(),
        }
    }

    pub fn create_table(&mut self, name: String, schema: Schema) -> Result<(), String> {
        if self.tables.contains_key(&name) {
            return Err(format!("Table '{}' already exists", name));
        }

        let table = Table::new(name.clone(), schema);
        self.tables.insert(name.clone(), table);
        self.log_operation(format!("CREATE TABLE {}", name));
        Ok(())
    }

    pub fn insert_tuple(&mut self, table_name: &str, tuple: Tuple) -> Result<(), String> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        table.insert(tuple)?;
        self.log_operation(format!("INSERT INTO {}", table_name));
        Ok(())
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    pub fn delete_from_table(&mut self, table_name: &str) -> Result<usize, String> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        let deleted_count = table.tuples.len();
        table.tuples.clear();
        self.log_operation(format!("DELETE FROM {}", table_name));
        Ok(deleted_count)
    }

    pub fn list_tables(&self) -> Vec<&String> {
        self.tables.keys().collect()
    }

    fn log_operation(&mut self, operation: String) {
        self.transaction_log.push(operation);
    }

    pub fn print_table(&self, table_name: &str) -> Result<(), String> {
        let table = self
            .get_table(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        println!("\n📋 表: {}", table_name);
        println!("┌─────────────────────────────────────────────────────────┐");

        // 打印列头
        print!("│ ");
        for (i, column) in table.schema.columns.iter().enumerate() {
            print!("{:12}", column.name);
            if i < table.schema.columns.len() - 1 {
                print!(" │ ");
            }
        }
        println!(" │");
        println!("├─────────────────────────────────────────────────────────┤");

        // 打印数据行
        if table.tuples.is_empty() {
            println!("│                      (无数据)                          │");
        } else {
            for tuple in &table.tuples {
                print!("│ ");
                for (i, value) in tuple.values.iter().enumerate() {
                    print!("{:12}", format!("{}", value));
                    if i < tuple.values.len() - 1 {
                        print!(" │ ");
                    }
                }
                println!(" │");
            }
        }

        println!("└─────────────────────────────────────────────────────────┘");
        println!("记录数: {}", table.tuples.len());
        Ok(())
    }

    pub fn print_transaction_log(&self) {
        println!("\n📜 事务日志:");
        for (i, operation) in self.transaction_log.iter().enumerate() {
            println!("  {}: {}", i + 1, operation);
        }
        println!("总操作数: {}", self.transaction_log.len());
    }
}

// 查询执行器
pub struct QueryExecutor {
    database: Database,
}

impl QueryExecutor {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn execute_create_table(
        &mut self,
        table_name: String,
        columns: Vec<(String, DataType)>,
    ) -> Result<(), String> {
        let schema_columns: Vec<Column> = columns
            .into_iter()
            .map(|(name, data_type)| Column {
                name,
                data_type,
                nullable: true,
            })
            .collect();

        let schema = Schema::new(schema_columns);
        let mut op = CreateTableOperator::new(table_name, schema);
        op.execute(&mut self.database)
    }

    pub fn execute_insert(
        &mut self,
        table_name: String,
        values: Vec<Value>,
    ) -> Result<(), String> {
        let tuple = Tuple::new(values);
        let mut op = InsertOperator::new(table_name, tuple);
        op.execute(&mut self.database)
    }

    pub fn execute_select(&self, table_name: &str) -> Result<Vec<Tuple>, String> {
        let table = self
            .database
            .get_table(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        let mut scan_op = SeqScanOperator::new(table.clone());
        let mut results = Vec::new();

        println!("🔍 SEQSCAN: 扫描表 {}", table_name);
        while let Some(tuple) = scan_op.next() {
            results.push(tuple);
        }

        println!("   扫描完成，找到 {} 条记录", results.len());
        Ok(results)
    }

    pub fn execute_select_with_filter<F>(
        &self,
        table_name: &str,
        predicate: F,
    ) -> Result<Vec<Tuple>, String>
    where
        F: Fn(&Tuple) -> bool + 'static,
    {
        let table = self
            .database
            .get_table(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        let scan_op = SeqScanOperator::new(table.clone());
        let mut filter_op = FilterOperator::new(Box::new(scan_op), Box::new(predicate));
        let mut results = Vec::new();

        println!("🔍 SEQSCAN + FILTER: 扫描并过滤表 {}", table_name);
        while let Some(tuple) = filter_op.next() {
            results.push(tuple);
        }

        println!("   过滤完成，找到 {} 条匹配记录", results.len());
        Ok(results)
    }

    pub fn execute_select_with_projection(
        &self,
        table_name: &str,
        column_names: Vec<&str>,
    ) -> Result<Vec<Tuple>, String> {
        let table = self
            .database
            .get_table(table_name)
            .ok_or_else(|| format!("Table '{}' not found", table_name))?;

        // 查找列索引
        let mut column_indices = Vec::new();
        for column_name in column_names {
            if let Some(index) = table.schema.find_column(column_name) {
                column_indices.push(index);
            } else {
                return Err(format!("Column '{}' not found", column_name));
            }
        }

        let scan_op = SeqScanOperator::new(table.clone());
        let mut project_op = ProjectOperator::new(Box::new(scan_op), column_indices);
        let mut results = Vec::new();

        println!("🔍 SEQSCAN + PROJECT: 扫描并投影表 {}", table_name);
        while let Some(tuple) = project_op.next() {
            results.push(tuple);
        }

        println!("   投影完成，返回 {} 条记录", results.len());
        Ok(results)
    }

    pub fn execute_delete(&mut self, table_name: &str) -> Result<usize, String> {
        let deleted_count = self.database.delete_from_table(table_name)?;
        println!("🗑️ DELETE: 从表 {} 删除 {} 条记录", table_name, deleted_count);
        Ok(deleted_count)
    }

    pub fn get_database(&self) -> &Database {
        &self.database
    }

    pub fn get_database_mut(&mut self) -> &mut Database {
        &mut self.database
    }
}

// 交互式测试工具
pub struct InteractiveTest {
    executor: QueryExecutor,
}

impl InteractiveTest {
    pub fn new() -> Self {
        Self {
            executor: QueryExecutor::new(Database::new()),
        }
    }

    pub fn run_comprehensive_test(&mut self) -> Result<(), String> {
        println!("=== MiniDB 数据库系统完整生命周期测试 ===");
        println!();

        // 阶段1: 建表
        println!("=== 阶段1: 建表操作 ===");
        self.test_create_tables()?;

        // 阶段2: 插入数据
        println!("\n=== 阶段2: 插入数据 ===");
        self.test_insert_data()?;

        // 阶段3: 查询操作
        println!("\n=== 阶段3: 查询操作 ===");
        self.test_query_operations()?;

        // 阶段4: 高级查询
        println!("\n=== 阶段4: 高级查询操作 ===");
        self.test_advanced_queries()?;

        // 阶段5: 删除操作
        println!("\n=== 阶段5: 删除操作 ===");
        self.test_delete_operations()?;

        // 阶段6: 再次查询验证
        println!("\n=== 阶段6: 删除后查询验证 ===");
        self.test_verify_after_delete()?;

        // 阶段7: 事务日志和统计
        println!("\n=== 阶段7: 系统统计和日志 ===");
        self.test_system_stats()?;

        println!("\n✅ 完整生命周期测试完成！");
        Ok(())
    }

    fn test_create_tables(&mut self) -> Result<(), String> {
        // 创建用户表
        self.executor.execute_create_table(
            "users".to_string(),
            vec![
                ("id".to_string(), DataType::Integer),
                ("name".to_string(), DataType::String),
                ("age".to_string(), DataType::Integer),
                ("email".to_string(), DataType::String),
            ],
        )?;

        // 创建订单表
        self.executor.execute_create_table(
            "orders".to_string(),
            vec![
                ("order_id".to_string(), DataType::Integer),
                ("user_id".to_string(), DataType::Integer),
                ("amount".to_string(), DataType::Float),
                ("status".to_string(), DataType::String),
            ],
        )?;

        // 创建产品表
        self.executor.execute_create_table(
            "products".to_string(),
            vec![
                ("product_id".to_string(), DataType::Integer),
                ("name".to_string(), DataType::String),
                ("price".to_string(), DataType::Float),
                ("in_stock".to_string(), DataType::Boolean),
            ],
        )?;

        println!("✅ 创建了3个表: users, orders, products");
        Ok(())
    }

    fn test_insert_data(&mut self) -> Result<(), String> {
        // 插入用户数据
        let users_data = vec![
            vec![Value::Integer(1), Value::String("Alice".to_string()), Value::Integer(25), Value::String("alice@email.com".to_string())],
            vec![Value::Integer(2), Value::String("Bob".to_string()), Value::Integer(30), Value::String("bob@email.com".to_string())],
            vec![Value::Integer(3), Value::String("Carol".to_string()), Value::Integer(28), Value::String("carol@email.com".to_string())],
            vec![Value::Integer(4), Value::String("David".to_string()), Value::Integer(35), Value::String("david@email.com".to_string())],
            vec![Value::Integer(5), Value::String("Eve".to_string()), Value::Integer(22), Value::String("eve@email.com".to_string())],
        ];

        for user in users_data {
            self.executor.execute_insert("users".to_string(), user)?;
        }

        // 插入订单数据
        let orders_data = vec![
            vec![Value::Integer(101), Value::Integer(1), Value::Float(99.99), Value::String("completed".to_string())],
            vec![Value::Integer(102), Value::Integer(2), Value::Float(149.99), Value::String("pending".to_string())],
            vec![Value::Integer(103), Value::Integer(1), Value::Float(79.99), Value::String("completed".to_string())],
            vec![Value::Integer(104), Value::Integer(3), Value::Float(199.99), Value::String("shipped".to_string())],
            vec![Value::Integer(105), Value::Integer(4), Value::Float(59.99), Value::String("completed".to_string())],
        ];

        for order in orders_data {
            self.executor.execute_insert("orders".to_string(), order)?;
        }

        // 插入产品数据
        let products_data = vec![
            vec![Value::Integer(201), Value::String("Laptop".to_string()), Value::Float(999.99), Value::Boolean(true)],
            vec![Value::Integer(202), Value::String("Mouse".to_string()), Value::Float(29.99), Value::Boolean(true)],
            vec![Value::Integer(203), Value::String("Keyboard".to_string()), Value::Float(79.99), Value::Boolean(false)],
            vec![Value::Integer(204), Value::String("Monitor".to_string()), Value::Float(299.99), Value::Boolean(true)],
        ];

        for product in products_data {
            self.executor.execute_insert("products".to_string(), product)?;
        }

        println!("✅ 插入完成: users(5条), orders(5条), products(4条)");
        Ok(())
    }

    fn test_query_operations(&mut self) -> Result<(), String> {
        // 查询所有用户
        println!("\n1. 查询所有用户:");
        let users = self.executor.execute_select("users")?;
        self.executor.get_database().print_table("users")?;

        // 查询所有订单
        println!("\n2. 查询所有订单:");
        let orders = self.executor.execute_select("orders")?;
        self.executor.get_database().print_table("orders")?;

        // 查询所有产品
        println!("\n3. 查询所有产品:");
        let products = self.executor.execute_select("products")?;
        self.executor.get_database().print_table("products")?;

        Ok(())
    }

    fn test_advanced_queries(&mut self) -> Result<(), String> {
        // 查询年龄大于25的用户
        println!("\n1. 查询年龄大于25的用户:");
        let filtered_users = self.executor.execute_select_with_filter("users", |tuple| {
            if let Some(Value::Integer(age)) = tuple.get_value(2) {
                *age > 25
            } else {
                false
            }
        })?;
        
        println!("   找到 {} 个用户:", filtered_users.len());
        for (i, tuple) in filtered_users.iter().enumerate() {
            println!("   {}. {:?}", i + 1, tuple.values);
        }

        // 查询已完成的订单
        println!("\n2. 查询已完成的订单:");
        let completed_orders = self.executor.execute_select_with_filter("orders", |tuple| {
            if let Some(Value::String(status)) = tuple.get_value(3) {
                status == "completed"
            } else {
                false
            }
        })?;
        
        println!("   找到 {} 个已完成订单:", completed_orders.len());
        for (i, tuple) in completed_orders.iter().enumerate() {
            println!("   {}. {:?}", i + 1, tuple.values);
        }

        // 投影查询：只查询用户名和邮箱
        println!("\n3. 投影查询 - 用户名和邮箱:");
        let projected_users = self.executor.execute_select_with_projection("users", vec!["name", "email"])?;
        println!("   用户名和邮箱列表:");
        for (i, tuple) in projected_users.iter().enumerate() {
            println!("   {}. {:?}", i + 1, tuple.values);
        }

        // 投影查询：产品名和价格
        println!("\n4. 投影查询 - 产品名和价格:");
        let projected_products = self.executor.execute_select_with_projection("products", vec!["name", "price"])?;
        println!("   产品名和价格列表:");
        for (i, tuple) in projected_products.iter().enumerate() {
            println!("   {}. {:?}", i + 1, tuple.values);
        }

        Ok(())
    }

    fn test_delete_operations(&mut self) -> Result<(), String> {
        // 删除orders表中的所有记录
        println!("\n删除orders表中的所有记录...");
        let deleted_count = self.executor.execute_delete("orders")?;
        println!("已删除 {} 条记录", deleted_count);

        Ok(())
    }

    fn test_verify_after_delete(&mut self) -> Result<(), String> {
        // 验证删除后的状态
        println!("\n验证删除操作:");
        
        println!("\n1. 查询users表（应该有数据）:");
        self.executor.get_database().print_table("users")?;

        println!("\n2. 查询orders表（应该为空）:");
        self.executor.get_database().print_table("orders")?;

        println!("\n3. 查询products表（应该有数据）:");
        self.executor.get_database().print_table("products")?;

        Ok(())
    }

    fn test_system_stats(&mut self) -> Result<(), String> {
        // 显示所有表
        println!("\n数据库表列表:");
        let tables = self.executor.get_database().list_tables();
        for table_name in tables {
            println!("  📋 {}", table_name);
        }

        // 显示事务日志
        self.executor.get_database().print_transaction_log();

        Ok(())
    }

    pub fn run_interactive_mode(&mut self) {
        println!("\n=== MiniDB 交互模式 ===");
        println!("输入命令进行数据库操作（输入 'help' 查看帮助，'quit' 退出）:");

        loop {
            print!("minidb> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let command = input.trim();
                    
                    if command.is_empty() {
                        continue;
                    }

                    match command {
                        "quit" | "exit" => {
                            println!("再见！");
                            break;
                        }
                        "help" => self.show_help(),
                        "show tables" => self.show_tables(),
                        "show log" => self.executor.get_database().print_transaction_log(),
                        cmd if cmd.starts_with("show ") => {
                            let table_name = &cmd[5..];
                            if let Err(e) = self.executor.get_database().print_table(table_name) {
                                println!("❌ 错误: {}", e);
                            }
                        }
                        cmd if cmd.starts_with("select ") => {
                            let table_name = &cmd[7..];
                            match self.executor.execute_select(table_name) {
                                Ok(_) => {
                                    if let Err(e) = self.executor.get_database().print_table(table_name) {
                                        println!("❌ 错误: {}", e);
                                    }
                                }
                                Err(e) => println!("❌ 查询错误: {}", e),
                            }
                        }
                        cmd if cmd.starts_with("delete ") => {
                            let table_name = &cmd[7..];
                            match self.executor.execute_delete(table_name) {
                                Ok(count) => println!("✅ 删除了 {} 条记录", count),
                                Err(e) => println!("❌ 删除错误: {}", e),
                            }
                        }
                        "test" => {
                            if let Err(e) = self.run_comprehensive_test() {
                                println!("❌ 测试错误: {}", e);
                            }
                        }
                        _ => println!("❓ 未知命令: {}，输入 'help' 查看帮助", command),
                    }
                }
                Err(e) => {
                    println!("❌ 输入错误: {}", e);
                    break;
                }
            }
        }
    }

    fn show_help(&self) {
        println!("\n📚 MiniDB 命令帮助:");
        println!("  help              - 显示此帮助信息");
        println!("  show tables       - 显示所有表");
        println!("  show <table>      - 显示指定表的内容");
        println!("  select <table>    - 查询指定表（使用SEQSCAN操作符）");
        println!("  delete <table>    - 删除指定表的所有记录");
        println!("  show log          - 显示事务日志");
        println!("  test              - 运行完整的生命周期测试");
        println!("  quit/exit         - 退出程序");
        println!("\n示例:");
        println!("  show users");
        println!("  select orders");
        println!("  delete products");
    }

    fn show_tables(&self) {
        println!("\n📋 数据库表列表:");
        let tables = self.executor.get_database().list_tables();
        if tables.is_empty() {
            println!("  (无表)");
        } else {
            for table_name in tables {
                println!("  📋 {}", table_name);
            }
        }
    }
}

fn main() {
    println!("=== MiniDB 数据库系统演示 ===");
    println!();

    let mut test = InteractiveTest::new();

    // 询问用户运行模式
    println!("请选择运行模式:");
    println!("1. 自动演示模式（运行完整测试）");
    println!("2. 交互模式（手动输入命令）");
    println!("3. 两者都运行");
    
    print!("请输入选择 (1/2/3): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    match choice {
        "1" => {
            // 只运行自动测试
            if let Err(e) = test.run_comprehensive_test() {
                println!("❌ 测试失败: {}", e);
            }
        }
        "2" => {
            // 只运行交互模式
            test.run_interactive_mode();
        }
        "3" | "" => {
            // 先运行自动测试，再进入交互模式
            if let Err(e) = test.run_comprehensive_test() {
                println!("❌ 测试失败: {}", e);
            }
            
            println!("\n{}", "=".repeat(50));
            test.run_interactive_mode();
        }
        _ => {
            println!("❌ 无效选择，运行默认测试");
            if let Err(e) = test.run_comprehensive_test() {
                println!("❌ 测试失败: {}", e);
            }
        }
    }

    println!("\n🎉 MiniDB 数据库系统演示结束");
}