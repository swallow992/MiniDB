# MiniDB 数据持久化修复计划

## 问题分析

当前MiniDB的主要问题是数据只存储在内存中的HashMap，程序重启后数据丢失。需要实现真正的文件系统持久化。

## 修复计划

### Phase 1: 简单文件持久化 (2-4小时)

#### 1.1 实现表数据序列化
```rust
// 在 src/engine/database.rs 中添加
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
struct TableData {
    schema: Schema,
    rows: Vec<Tuple>,
}

impl Database {
    fn save_table(&self, table_id: u32, table_name: &str) -> Result<(), ExecutionError> {
        let table_data = TableData {
            schema: self.table_schemas.get(&table_id).unwrap().clone(),
            rows: self.table_data.get(&table_id).unwrap_or(&vec![]).clone(),
        };
        
        let file_path = self.data_dir.join(format!("table_{}.json", table_id));
        let json = serde_json::to_string_pretty(&table_data)
            .map_err(|e| ExecutionError::StorageError(format!("Serialization error: {}", e)))?;
            
        let mut file = File::create(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("File creation error: {}", e)))?;
            
        file.write_all(json.as_bytes())
            .map_err(|e| ExecutionError::StorageError(format!("Write error: {}", e)))?;
            
        Ok(())
    }
    
    fn load_table(&mut self, table_id: u32) -> Result<(), ExecutionError> {
        let file_path = self.data_dir.join(format!("table_{}.json", table_id));
        
        if !file_path.exists() {
            return Ok(()); // 文件不存在，跳过
        }
        
        let mut file = File::open(file_path)
            .map_err(|e| ExecutionError::StorageError(format!("File open error: {}", e)))?;
            
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| ExecutionError::StorageError(format!("Read error: {}", e)))?;
            
        let table_data: TableData = serde_json::from_str(&contents)
            .map_err(|e| ExecutionError::StorageError(format!("Deserialization error: {}", e)))?;
            
        self.table_schemas.insert(table_id, table_data.schema);
        self.table_data.insert(table_id, table_data.rows);
        
        Ok(())
    }
}
```

#### 1.2 修改CRUD操作以触发持久化
```rust
// 在每个数据修改操作后调用save_table
impl Database {
    fn execute_create_table(&mut self, table_name: String, columns: Vec<ColumnDef>) -> Result<QueryResult, ExecutionError> {
        // ... 现有代码 ...
        
        // 添加持久化
        self.save_table(table_id, &table_name)?;
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: 0,
            message: format!("Table '{}' created successfully", table_name),
        })
    }
    
    fn execute_insert(&mut self, table_name: String, values: Vec<Vec<Value>>) -> Result<QueryResult, ExecutionError> {
        // ... 现有代码 ...
        
        // 添加持久化
        self.save_table(*table_id, &table_name)?;
        
        Ok(QueryResult {
            rows: vec![],
            schema: None,
            affected_rows: values.len(),
            message: format!("Inserted {} row(s) into table '{}'", values.len(), table_name),
        })
    }
    
    // 类似地修改 UPDATE 和 DELETE 操作
}
```

#### 1.3 启动时加载所有表
```rust
impl Database {
    pub fn new(data_dir: &str) -> Result<Self, ExecutionError> {
        // ... 现有初始化代码 ...
        
        // 加载现有表
        let mut db = Database {
            // ... 初始化字段 ...
        };
        
        db.load_existing_tables()?;
        Ok(db)
    }
    
    fn load_existing_tables(&mut self) -> Result<(), ExecutionError> {
        // 扫描数据目录，加载所有表文件
        let entries = std::fs::read_dir(&self.data_dir)
            .map_err(|e| ExecutionError::StorageError(format!("Directory read error: {}", e)))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| ExecutionError::StorageError(format!("Entry error: {}", e)))?;
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("table_") && name.ends_with(".json") {
                    // 提取table_id
                    if let Some(id_str) = name.strip_prefix("table_").and_then(|s| s.strip_suffix(".json")) {
                        if let Ok(table_id) = id_str.parse::<u32>() {
                            self.load_table(table_id)?;
                            
                            // 重建table_catalog (需要额外的元数据文件或在JSON中存储表名)
                            // 这里需要改进设计
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### Phase 2: 元数据管理改进 (1-2小时)

#### 2.1 创建数据库元数据文件
```rust
#[derive(Serialize, Deserialize)]
struct DatabaseMetadata {
    next_table_id: u32,
    table_catalog: HashMap<String, u32>,
}

impl Database {
    fn save_metadata(&self) -> Result<(), ExecutionError> {
        let metadata = DatabaseMetadata {
            next_table_id: self.next_table_id,
            table_catalog: self.table_catalog.clone(),
        };
        
        let file_path = self.data_dir.join("metadata.json");
        let json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(file_path, json)?;
        
        Ok(())
    }
    
    fn load_metadata(&mut self) -> Result<(), ExecutionError> {
        let file_path = self.data_dir.join("metadata.json");
        
        if file_path.exists() {
            let contents = std::fs::read_to_string(file_path)?;
            let metadata: DatabaseMetadata = serde_json::from_str(&contents)?;
            
            self.next_table_id = metadata.next_table_id;
            self.table_catalog = metadata.table_catalog;
        }
        
        Ok(())
    }
}
```

### Phase 3: 测试验证 (30分钟)

#### 3.1 创建持久性测试脚本
```powershell
# test_persistence_fix.ps1
Write-Host "🔄 测试数据持久性修复"

# 清理环境
Remove-Item -Recurse -Force minidb_data -ErrorAction SilentlyContinue

# 第一次会话：创建数据
$sql1 = @"
CREATE TABLE persistence_test (id INT, name TEXT, value INT);
INSERT INTO persistence_test VALUES (1, 'Data1', 100);
INSERT INTO persistence_test VALUES (2, 'Data2', 200);
SELECT * FROM persistence_test;
"@

Write-Host "第一次会话：创建数据"
echo $sql1 | cargo run

# 第二次会话：验证数据
Write-Host "`n第二次会话：验证数据持久性"
echo "SELECT * FROM persistence_test;" | cargo run

# 检查文件
Write-Host "`n文件系统状态："
Get-ChildItem minidb_data -Recurse | Format-Table Name, Length
```

### Phase 4: 性能优化 (可选，1-2小时)

#### 4.1 批量写入优化
```rust
impl Database {
    fn batch_save_tables(&self) -> Result<(), ExecutionError> {
        // 批量保存所有修改的表
        for (&table_id, table_name) in &self.table_catalog {
            if self.is_table_modified(table_id) {
                self.save_table(table_id, table_name)?;
            }
        }
        Ok(())
    }
}
```

#### 4.2 异步写入 (高级)
```rust
use tokio::fs;
use tokio::task;

impl Database {
    async fn save_table_async(&self, table_id: u32) -> Result<(), ExecutionError> {
        // 异步文件写入，不阻塞查询操作
    }
}
```

## 实现步骤

1. **添加依赖** (Cargo.toml)
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

2. **修改数据结构**
   - 为Schema和相关类型添加Serialize/Deserialize
   - 确保所有需要持久化的类型可序列化

3. **实现核心功能**
   - save_table()
   - load_table()
   - save_metadata()
   - load_metadata()

4. **修改CRUD操作**
   - 在每个修改操作后调用持久化
   - 在Database::new()中加载现有数据

5. **测试验证**
   - 创建数据后重启验证
   - 测试大量数据的持久化性能
   - 验证错误恢复机制

## 预期结果

修复完成后，MiniDB将具备：

✅ **完整的数据持久性** - 重启后数据不丢失
✅ **表结构恢复** - 重启后表定义正确恢复
✅ **元数据管理** - 表目录和ID正确维护
✅ **错误恢复** - 文件损坏时的适当处理

## 进一步改进建议

1. **实现二进制格式** - 比JSON更高效的存储格式
2. **添加WAL日志** - 写前日志保证ACID特性
3. **实现页式存储** - 真正的数据库页管理
4. **压缩和索引** - 提高存储效率和查询性能

这样MiniDB就能成为一个真正可用的持久化数据库系统！