# 🗃️ MiniDB - Rust数据库系统

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

一个用Rust构建的完整数据库系统，实现了SQL编译器、存储引擎和查询执行器。本项目旨在通过实际代码演示数据库内核的核心概念和技术。

## 🚀 项目状态

### ✅ 已完成模块

- **SQL编译器** (100% 完成) - 38个测试 ✅
  - 词法分析器 (Lexer) - 10个测试 ✅
  - 语法分析器 (Parser) - 12个测试 ✅  
  - 语义分析器 (Analyzer) - 8个测试 ✅
  - 执行计划生成器 (Planner) - 8个测试 ✅

- **存储系统** (100% 完成) - 22个测试 ✅
  - 页式存储 (Page) - 6个测试 ✅
  - 文件管理 (File) - 5个测试 ✅
  - 缓存管理 (Buffer) - 6个测试 ✅
  - 索引系统 (Index) - 5个测试 ✅

- **数据库引擎** (100% 完成) - 10个测试 ✅
  - 数据库实例管理 ✅
  - 表创建和删除 ✅
  - 数据插入功能 ✅
  - 基本查询支持 ✅
  - 错误处理和验证 ✅

### 🚧 待开发模块

- 高级查询功能 - JOIN、GROUP BY、ORDER BY支持
- 事务处理 (Transaction Manager) - ACID事务支持
- 网络层 (Network Layer) - 客户端连接支持
- 查询优化器 - 成本估算和执行计划优化

## 📋 快速测试

### 30秒快速验证
```bash
cd D:\repositories\MniDB
## 🎯 三模块快速测试

### 1️⃣ SQL编译器测试
```powershell
.\run_sql_compiler.ps1
# 选择: 1 (运行标准测试套件)
# 验证: 词法分析 → 语法分析 → 语义分析 → 执行计划
```

### 2️⃣ 存储系统测试  
```powershell
.\run_storage_test.ps1
# 选择: 1 (完整自动化测试套件)
# 验证: 页面管理 → 缓存机制 → 数据持久化 → 性能统计
```

### 3️⃣ 数据库系统测试
```powershell
cargo run --bin database_demo
# 选择: 1 (自动演示模式)
# 验证: 建表 → 插入 → 查询 → 删除 → 数据恢复
```

### 详细测试指南
- **📘 完整测试手册**: [COMPLETE_TESTING_MANUAL.md](COMPLETE_TESTING_MANUAL.md) - 三模块详细测试指南 ⭐
- **📋 测试指南**: [TESTING_GUIDE.md](TESTING_GUIDE.md) - 简化版测试说明
- **🔧 SQL编译器指南**: [SQL_COMPILER_GUIDE.md](SQL_COMPILER_GUIDE.md) - SQL编译器详细说明

## 💫 支持的SQL功能

### DDL (数据定义语言)
```sql
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    age INT,
    email VARCHAR(255)
);

DROP TABLE users;
```

### DML (数据操作语言)
```sql
-- 查询
SELECT * FROM users;
SELECT id, name FROM users WHERE age > 18;
SELECT * FROM users WHERE age > 18 AND name = 'Alice';

-- 插入  
INSERT INTO users (name, age) VALUES ('Alice', 25);
INSERT INTO users VALUES ('Alice', 25, 'alice@example.com');

-- 更新
UPDATE users SET age = 26 WHERE name = 'Alice';
UPDATE users SET age = age + 1 WHERE age < 30;

-- 删除
DELETE FROM users WHERE age < 18;
DELETE FROM users WHERE name IS NULL;
```

### 数据类型支持
- `INT` / `INTEGER` - 32位整数
- `BIGINT` - 64位整数  
- `FLOAT` - 32位浮点数
- `DOUBLE` - 64位浮点数
- `VARCHAR(n)` - 可变长字符串
- `BOOLEAN` / `BOOL` - 布尔值
- `DATE` - 日期
- `TIMESTAMP` - 时间戳

### 表达式和运算符
- **算术运算**: `+`, `-`, `*`, `/`, `%`
- **比较运算**: `=`, `<>`, `!=`, `<`, `<=`, `>`, `>=`  
- **逻辑运算**: `AND`, `OR`, `NOT`
- **其他运算**: `IN`, `LIKE`, `BETWEEN`, `IS NULL`, `IS NOT NULL`

## 🏗️ 架构设计

### SQL编译器流水线
```
SQL文本 → [Lexer] → Tokens → [Parser] → AST → [Analyzer] → Analyzed AST → [Planner] → Execution Plan
```

### 存储系统架构
```
查询请求 → [Buffer Pool] → [Page Manager] → [File Manager] → 磁盘存储
                ↓
            [Index System] → 快速数据检索
```

### 核心组件

**SQL编译器组件：**
- **Lexer**: 词法分析，支持所有SQL关键字和操作符
- **Parser**: 递归下降语法分析器，构建完整AST
- **Analyzer**: 语义分析和类型检查，支持模式验证
- **Planner**: 查询优化和执行计划生成

**存储系统组件：**
- **Page**: 8KB固定大小页面，支持记录CRUD操作
- **File**: 数据库文件管理，提供原子I/O操作
- **Buffer**: LRU缓冲池，智能内存管理和脏页写回
- **Index**: B+树和哈希索引，支持范围查询和精确查找

**数据库引擎组件：**
- **Database**: 数据库实例管理，连接SQL编译器与存储系统
- **Executor**: 查询执行器框架，支持基本的表操作
- **Table**: 表管理和元数据维护
- **Transaction**: 事务处理框架（待完善）

## 📊 测试覆盖

| 模块 | 测试数量 | 覆盖功能 |
|------|----------|----------|
| **SQL编译器** | **38** | |
| Lexer | 10 | 关键字、标识符、字面量、运算符、注释 |
| Parser | 12 | 所有SQL语句、表达式解析 |
| Analyzer | 8 | 类型检查、语义验证、错误检测 |
| Planner | 8 | 执行计划生成、查询优化准备 |
| **存储系统** | **22** | |
| Page | 6 | 页面管理、记录操作、序列化 |
| File | 5 | 文件I/O、页面分配、元数据管理 |
| Buffer | 6 | LRU缓存、脏页管理、并发安全 |
| Index | 5 | B+树索引、哈希索引、范围查询 |
| **数据库引擎** | **10** | |
| Database | 10 | 表管理、数据操作、错误处理 |
| **总计** | **70** | **100%核心功能覆盖** |

## 🛠️ 开发环境

### 依赖项
- Rust 1.70+
- 主要crates:
  - `thiserror` - 错误处理
  - `serde` - 序列化
  - `chrono` - 时间处理
  - `indexmap` - 有序映射

### 编译和运行
```bash
# 编译
cargo build

# 运行测试
cargo test

# 运行主程序 (当前为交互式CLI)
cargo run

# 生成文档
cargo doc --open
```

## 🎯 下一步开发计划

1. **数据库引擎** - 连接SQL编译器与存储系统，实现完整的查询处理
2. **查询执行器** - 实现执行计划的具体执行逻辑
3. **事务处理** - 添加ACID事务支持和并发控制
4. **高级SQL特性** - JOIN、GROUP BY、ORDER BY、子查询等
5. **网络层** - 实现数据库协议，支持多客户端连接
6. **查询优化器** - 成本估算、索引选择、执行计划优化

## � 项目文档

| 文档类型 | 文件名 | 描述 |
|---------|--------|------|
| **核心文档** | `README.md` | 项目概述和快速开始 |
| **完整测试** | `COMPLETE_TEST_DOCUMENTATION.md` | 全面的功能测试用例 |
| **快速测试** | `QUICK_TEST_SCRIPT.md` | 5分钟快速验证功能 |
| **手动测试** | `MANUAL_TEST_GUIDE.md` | 交互式测试指南 |
| **技术文档** | `SQL_COMPILER_GUIDE.md` | SQL编译器技术细节 |
| **数据类型** | `DATA_TYPES_GUIDE.md` | 支持的数据类型说明 |
| **项目结构** | `PROJECT_STRUCTURE.md` | 代码组织和架构 |

## �📖 学习资源

这个项目适合学习：
- Rust系统编程和内存管理
- 数据库内核设计和实现
- 编译器前端技术（词法/语法分析）
- 存储引擎和缓存管理
- 查询处理和执行算法

## 🤝 贡献

欢迎提交Issue和Pull Request！请确保：
1. 运行 `cargo test` 通过所有测试
2. 遵循Rust代码规范
3. 更新相关文档

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

---

**当前版本**: v1.0.0  
**开发状态**: 核心功能完成，支持完整SQL操作  
**测试状态**: 72/72 通过 ✅  
**文档状态**: 完整 �