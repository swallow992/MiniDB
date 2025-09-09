# 数据库引擎模块指令

## 模块职责
数据库引擎是MiniDB的核心执行层，负责执行查询计划、管理表结构、处理事务和实现基本的查询操作。

## 开发指导

### 执行器 (executor.rs)
- **执行模型**: 实现火山模型（Volcano Model）迭代器接口
- **算子实现**: 支持扫描、连接、聚合、排序等基本算子
- **内存管理**: 实现内存限制和溢出处理
- **并行执行**: 支持基本的并行查询执行

```rust
// 推荐的执行器接口
pub trait Executor: Send {
    type Item;
    
    fn next(&mut self) -> Result<Option<Self::Item>, ExecutionError>;
    fn schema(&self) -> &Schema;
    fn reset(&mut self) -> Result<(), ExecutionError>;
}

// 基础执行器实现
pub struct TableScanExecutor {
    table: Arc<Table>,
    filter: Option<Expression>,
    current_page: usize,
    current_slot: usize,
    schema: Schema,
}

pub struct ProjectionExecutor {
    input: Box<dyn Executor<Item = Tuple>>,
    expressions: Vec<Expression>,
    schema: Schema,
}

pub struct HashJoinExecutor {
    left: Box<dyn Executor<Item = Tuple>>,
    right: Box<dyn Executor<Item = Tuple>>,
    join_condition: Expression,
    hash_table: HashMap<HashKey, Vec<Tuple>>,
    probe_tuples: Vec<Tuple>,
    current_probe: usize,
    schema: Schema,
}

impl Executor for TableScanExecutor {
    type Item = Tuple;
    
    fn next(&mut self) -> Result<Option<Tuple>, ExecutionError> {
        // 实现表扫描逻辑
        while self.current_page < self.table.page_count() {
            let page = self.table.get_page(self.current_page)?;
            
            while self.current_slot < page.slot_count() {
                if let Some(tuple) = page.get_tuple(self.current_slot)? {
                    self.current_slot += 1;
                    
                    // 应用过滤条件
                    if let Some(ref filter) = self.filter {
                        if !self.evaluate_filter(filter, &tuple)? {
                            continue;
                        }
                    }
                    
                    return Ok(Some(tuple));
                }
                self.current_slot += 1;
            }
            
            self.current_page += 1;
            self.current_slot = 0;
        }
        
        Ok(None)
    }
}
```

### 表管理器 (table.rs)
- **表元数据**: 管理表定义、列信息、索引信息
- **数据插入**: 实现记录插入和页面管理
- **数据删除**: 支持逻辑删除和物理删除
- **索引维护**: 自动维护主键和二级索引

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub schema: Schema,
    pub primary_key: Option<String>,
    pages: Vec<Arc<RwLock<Page>>>,
    indexes: HashMap<String, Box<dyn Index>>,
    statistics: TableStatistics,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Integer,
    Float,
    Varchar(usize),
    Boolean,
    Date,
    Timestamp,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    NotNull,
    Unique,
    PrimaryKey,
    ForeignKey { referenced_table: String, referenced_column: String },
    Check(Expression),
}

impl Table {
    pub fn new(name: String, schema: Schema) -> Result<Self, TableError> {
        // 验证schema合法性
        Self::validate_schema(&schema)?;
        
        Ok(Table {
            name,
            schema,
            primary_key: Self::find_primary_key(&schema),
            pages: Vec::new(),
            indexes: HashMap::new(),
            statistics: TableStatistics::default(),
        })
    }
    
    pub fn insert_tuple(&mut self, tuple: &Tuple) -> Result<TupleId, TableError> {
        // 验证tuple与schema匹配
        self.validate_tuple(tuple)?;
        
        // 检查约束条件
        self.check_constraints(tuple)?;
        
        // 找到可用页面或分配新页面
        let page_id = self.find_available_page(tuple.size())?;
        let mut page = self.pages[page_id].write().unwrap();
        
        // 插入tuple到页面
        let slot_id = page.insert_tuple(tuple)?;
        let tuple_id = TupleId { page_id: page_id as u32, slot_id };
        
        // 更新索引
        self.update_indexes_on_insert(&tuple_id, tuple)?;
        
        // 更新统计信息
        self.statistics.tuple_count += 1;
        
        Ok(tuple_id)
    }
    
    pub fn delete_tuple(&mut self, tuple_id: &TupleId) -> Result<(), TableError> {
        let page_id = tuple_id.page_id as usize;
        let mut page = self.pages[page_id].write().unwrap();
        
        // 获取要删除的tuple用于索引更新
        let tuple = page.get_tuple(tuple_id.slot_id)?
            .ok_or(TableError::TupleNotFound)?;
        
        // 删除tuple
        page.delete_tuple(tuple_id.slot_id)?;
        
        // 更新索引
        self.update_indexes_on_delete(tuple_id, &tuple)?;
        
        // 更新统计信息
        self.statistics.tuple_count -= 1;
        
        Ok(())
    }
    
    pub fn get_tuple(&self, tuple_id: &TupleId) -> Result<Option<Tuple>, TableError> {
        let page_id = tuple_id.page_id as usize;
        if page_id >= self.pages.len() {
            return Ok(None);
        }
        
        let page = self.pages[page_id].read().unwrap();
        page.get_tuple(tuple_id.slot_id)
    }
}
```

### 查询处理器 (query.rs)
- **查询优化**: 实现基于规则的查询优化
- **计划执行**: 将逻辑计划转换为物理执行计划
- **结果处理**: 格式化查询结果
- **事务集成**: 与事务管理器集成

```rust
pub struct QueryProcessor {
    catalog: Arc<RwLock<Catalog>>,
    buffer_pool: Arc<BufferPool>,
    transaction_manager: Arc<TransactionManager>,
    optimizer: QueryOptimizer,
}

pub struct QueryResult {
    pub schema: Schema,
    pub tuples: Vec<Tuple>,
    pub execution_time: Duration,
    pub stats: ExecutionStats,
}

#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub tuples_processed: u64,
    pub pages_read: u64,
    pub pages_written: u64,
    pub index_lookups: u64,
}

impl QueryProcessor {
    pub fn new(
        catalog: Arc<RwLock<Catalog>>,
        buffer_pool: Arc<BufferPool>,
        transaction_manager: Arc<TransactionManager>,
    ) -> Self {
        Self {
            catalog,
            buffer_pool,
            transaction_manager,
            optimizer: QueryOptimizer::new(),
        }
    }
    
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, QueryError> {
        let start_time = Instant::now();
        
        // 1. SQL解析
        let statement = self.parse_sql(sql)?;
        
        // 2. 语义分析和验证
        let validated_statement = self.analyze_statement(statement)?;
        
        // 3. 生成逻辑查询计划
        let logical_plan = self.create_logical_plan(validated_statement)?;
        
        // 4. 查询优化
        let optimized_plan = self.optimizer.optimize(logical_plan)?;
        
        // 5. 生成物理执行计划
        let physical_plan = self.create_physical_plan(optimized_plan)?;
        
        // 6. 执行查询
        let tuples = self.execute_plan(physical_plan).await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(QueryResult {
            schema: self.extract_result_schema(&tuples),
            tuples,
            execution_time,
            stats: ExecutionStats::default(), // TODO: 收集实际统计信息
        })
    }
    
    async fn execute_plan(&self, mut plan: Box<dyn Executor<Item = Tuple>>) -> Result<Vec<Tuple>, QueryError> {
        let mut results = Vec::new();
        
        while let Some(tuple) = plan.next()? {
            results.push(tuple);
        }
        
        Ok(results)
    }
}
```

## 数据类型系统

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Varchar(String),
    Boolean(bool),
    Date(chrono::NaiveDate),
    Timestamp(chrono::NaiveDateTime),
}

impl Value {
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Null => DataType::Varchar(0), // Null可以是任意类型
            Value::Integer(_) => DataType::Integer,
            Value::Float(_) => DataType::Float,
            Value::Varchar(s) => DataType::Varchar(s.len()),
            Value::Boolean(_) => DataType::Boolean,
            Value::Date(_) => DataType::Date,
            Value::Timestamp(_) => DataType::Timestamp,
        }
    }
    
    pub fn is_compatible_with(&self, data_type: &DataType) -> bool {
        match (self, data_type) {
            (Value::Null, _) => true,
            (Value::Integer(_), DataType::Integer) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::Integer(_), DataType::Float) => true, // 允许隐式转换
            (Value::Varchar(s), DataType::Varchar(max_len)) => s.len() <= *max_len,
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Date(_), DataType::Date) => true,
            (Value::Timestamp(_), DataType::Timestamp) => true,
            _ => false,
        }
    }
    
    pub fn cast_to(&self, target_type: &DataType) -> Result<Value, ValueError> {
        // 实现类型转换逻辑
        match (self, target_type) {
            (Value::Integer(i), DataType::Float) => Ok(Value::Float(*i as f64)),
            (Value::Float(f), DataType::Integer) => Ok(Value::Integer(*f as i64)),
            (Value::Varchar(s), DataType::Integer) => {
                s.parse::<i64>()
                    .map(Value::Integer)
                    .map_err(|_| ValueError::InvalidCast)
            }
            // 更多转换规则...
            _ => Err(ValueError::InvalidCast),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tuple {
    values: Vec<Value>,
}

impl Tuple {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }
    
    pub fn get_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
    
    pub fn size(&self) -> usize {
        self.values.iter().map(|v| v.serialized_size()).sum()
    }
}
```

## 错误处理

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("执行错误: {message}")]
    ExecutionError { message: String },
    
    #[error("表错误: {message}")]
    TableError { message: String },
    
    #[error("查询错误: {message}")]
    QueryError { message: String },
    
    #[error("类型错误: {message}")]
    TypeError { message: String },
    
    #[error("约束违反: {constraint}")]
    ConstraintViolation { constraint: String },
    
    #[error("存储错误: {source}")]
    StorageError { #[from] source: crate::storage::StorageError },
}
```

## 性能优化

### 执行器优化
```rust
// 向量化执行
pub trait VectorizedExecutor {
    fn next_batch(&mut self, batch_size: usize) -> Result<Option<TupleBatch>, ExecutionError>;
}

pub struct TupleBatch {
    tuples: Vec<Tuple>,
    validity: BitVec, // 标记哪些tuple是有效的
}

// 内存池管理
pub struct ExecutionContext {
    memory_pool: Arc<MemoryPool>,
    temp_files: Vec<TempFile>,
    spill_threshold: usize,
}
```

### 缓存和预取
```rust
impl Table {
    // 预取页面优化顺序扫描
    pub fn prefetch_pages(&self, start_page: usize, count: usize) {
        for page_id in start_page..(start_page + count) {
            if page_id < self.pages.len() {
                // 异步预取页面到缓冲池
                self.buffer_pool.prefetch_page(page_id);
            }
        }
    }
}
```

## 测试和调试

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_operations() {
        let schema = Schema::new(vec![
            ColumnDefinition::new("id", DataType::Integer, false),
            ColumnDefinition::new("name", DataType::Varchar(50), false),
        ]);
        
        let mut table = Table::new("users".to_string(), schema).unwrap();
        
        // 测试插入
        let tuple = Tuple::new(vec![
            Value::Integer(1),
            Value::Varchar("Alice".to_string()),
        ]);
        
        let tuple_id = table.insert_tuple(&tuple).unwrap();
        
        // 测试查询
        let retrieved = table.get_tuple(&tuple_id).unwrap().unwrap();
        assert_eq!(retrieved.get_value(0), Some(&Value::Integer(1)));
        assert_eq!(retrieved.get_value(1), Some(&Value::Varchar("Alice".to_string())));
        
        // 测试删除
        table.delete_tuple(&tuple_id).unwrap();
        assert!(table.get_tuple(&tuple_id).unwrap().is_none());
    }
}
```

### 集成测试
```rust
#[tokio::test]
async fn test_query_execution() {
    let query_processor = setup_test_database().await;
    
    // 创建表
    query_processor.execute_query("CREATE TABLE users (id INT, name VARCHAR(50))").await.unwrap();
    
    // 插入数据
    query_processor.execute_query("INSERT INTO users VALUES (1, 'Alice')").await.unwrap();
    query_processor.execute_query("INSERT INTO users VALUES (2, 'Bob')").await.unwrap();
    
    // 查询数据
    let result = query_processor.execute_query("SELECT * FROM users WHERE id > 1").await.unwrap();
    
    assert_eq!(result.tuples.len(), 1);
    assert_eq!(result.tuples[0].get_value(1), Some(&Value::Varchar("Bob".to_string())));
}
```

这些指导原则确保数据库引擎模块具有完整的功能、良好的性能和可靠的错误处理机制。
