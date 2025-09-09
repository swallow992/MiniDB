# GitHub Copilot Instructions for MiniDB

## 项目概述

MiniDB 是一个使用 Rust 语言构建的小型数据库系统，旨在贯通编译原理、操作系统与数据库知识。

### 项目目标

- 实现 SQL 编译器（词法分析、语法分析、语义分析、执行计划生成）
- 构建简化存储系统（页式存储模型、缓存管理）
- 开发数据库核心功能（表定义、数据存储、基本查询）

## Rust 代码规范

### 代码风格

- 使用标准 Rust 格式化工具 `rustfmt`
- 遵循 Rust 官方命名约定（snake_case 用于函数和变量，PascalCase 用于类型）
- 使用详细的文档注释（`///`）描述公共 API
- 错误处理优先使用 `Result<T, E>` 而非 `panic!`

### 项目结构约定

```
src/
├── main.rs              # 应用程序入口点
├── lib.rs               # 库根文件
├── sql/                 # SQL编译器模块
│   ├── lexer.rs         # 词法分析器
│   ├── parser.rs        # 语法分析器
│   ├── analyzer.rs      # 语义分析器
│   └── planner.rs       # 执行计划生成器
├── storage/             # 存储系统模块
│   ├── page.rs          # 页式存储
│   ├── buffer.rs        # 缓存管理
│   └── file.rs          # 文件系统接口
├── engine/              # 数据库引擎模块
│   ├── executor.rs      # 执行器
│   ├── table.rs         # 表管理
│   └── query.rs         # 查询处理
├── types/               # 类型定义
└── utils/               # 工具函数
```

## 开发指导原则

### 模块化设计

- 每个模块应具有清晰的职责边界
- 使用 trait 来定义接口，便于测试和扩展
- 模块间通过明确的 API 进行通信

### 错误处理策略

- 定义项目特定的错误类型
- 使用 `thiserror` 或 `anyhow` 进行错误管理
- 提供有意义的错误信息和上下文

### 测试要求

- 为所有公共 API 编写单元测试
- 使用集成测试验证模块间交互
- 考虑使用属性测试（property testing）进行模糊测试

### 性能考虑

- 优先考虑代码清晰性，然后优化性能
- 使用 benchmark 测试关键路径
- 避免不必要的内存分配

## SQL 编译器开发指南

### 词法分析器

- 支持 SQL 关键字、标识符、数字、字符串字面量
- 处理注释和空白字符
- 提供准确的错误位置信息

### 语法分析器

- 实现递归下降解析器或使用 nom 解析器组合器
- 支持基本 SQL 语句：SELECT, INSERT, UPDATE, DELETE, CREATE TABLE
- 生成抽象语法树（AST）

### 语义分析器

- 类型检查和作用域解析
- 验证表和列的存在性
- 处理聚合函数和表达式

## 存储系统开发指南

### 页式存储

- 定义固定大小页面（推荐 4KB 或 8KB）
- 实现页面分配和释放机制
- 支持不同类型的页面（数据页、索引页、元数据页）

### 缓存管理

- 实现 LRU 或 Clock 替换算法
- 考虑脏页写回策略
- 提供缓存统计信息

## 代码生成偏好

### 函数签名

```rust
// 推荐：明确的返回类型和错误处理
pub fn parse_sql(input: &str) -> Result<Statement, ParseError> {
    // 实现
}

// 避免：隐藏错误或使用unwrap
pub fn parse_sql(input: &str) -> Statement {
    // 不推荐的实现
}
```

### 结构体定义

```rust
// 推荐：使用构建器模式或工厂函数
#[derive(Debug, Clone)]
pub struct Table {
    name: String,
    columns: Vec<Column>,
    // 私有字段
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Result<Self, TableError> {
        // 验证逻辑
    }
}
```

## 依赖项推荐

### 核心依赖

- `serde` - 序列化/反序列化
- `tokio` - 异步运行时（如需要）
- `thiserror` - 错误处理
- `log` 和 `env_logger` - 日志记录

### 解析相关

- `nom` - 解析器组合器
- `pest` - 基于 PEG 的解析器生成器

### 测试相关

- `proptest` - 属性测试
- `criterion` - 基准测试

## 注释和文档

### 公共 API 文档

````rust
/// 解析SQL语句并返回抽象语法树
///
/// # 参数
/// * `sql` - 要解析的SQL字符串
///
/// # 返回值
/// 成功时返回 `Statement`，失败时返回 `ParseError`
///
/// # 例子
/// ```
/// let stmt = parse_sql("SELECT * FROM users")?;
/// ```
pub fn parse_sql(sql: &str) -> Result<Statement, ParseError> {
    // 实现
}
````

### 内联注释

```rust
// 使用LRU算法管理缓存页面
// TODO: 考虑实现ARC算法提高性能
let mut cache = LruCache::new(capacity);
```

## 调试和诊断

### 日志记录

```rust
use log::{debug, info, warn, error};

// 在关键路径添加日志
debug!("Parsing SQL: {}", sql);
info!("Table created: {}", table_name);
warn!("Buffer pool nearly full: {}/{}", used, capacity);
error!("Failed to write page: {}", error);
```

### 调试辅助

- 为关键数据结构实现 `Debug` trait
- 提供 dump 功能用于调试复杂状态
- 使用条件编译包含调试代码

## 代码审查要点

在生成代码时，请确保：

1. 所有 `unsafe` 代码都有详细注释说明其安全性
2. 公共 API 具有适当的文档注释
3. 错误路径得到适当处理
4. 资源（文件句柄、内存）得到正确释放
5. 并发安全性（如果适用）

## 性能指导

### 内存管理

- 使用 `Vec` 而非链表结构
- 考虑对象池模式减少分配
- 使用 `Cow` 类型避免不必要的克隆

### I/O 优化

- 批量操作减少系统调用
- 使用内存映射文件适当场景
- 实现预读和写合并策略

这些指导原则将帮助 Copilot 为 MiniDB 项目生成高质量、符合 Rust 最佳实践的代码。
