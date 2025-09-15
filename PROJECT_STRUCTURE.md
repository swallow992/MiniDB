# MiniDB 项目结构

## 📁 项目概述

MiniDB 是一个使用 Rust 语言构建的小型数据库系统，实现了基本的 SQL 编译器、存储系统和查询执行引擎。

## 🏗️ 核心架构

```
src/
├── lib.rs                    # 库根文件，定义公共接口
├── main.rs                   # 应用程序入口点，交互式shell
├── sql/                      # SQL编译器模块
│   ├── mod.rs                # SQL模块定义
│   ├── lexer.rs              # 词法分析器
│   ├── lexer_tests.rs        # 词法分析器测试
│   ├── parser.rs             # 语法分析器
│   ├── analyzer.rs           # 语义分析器
│   └── planner.rs            # 执行计划生成器
├── storage/                  # 存储系统模块
│   ├── mod.rs                # 存储模块定义
│   ├── page.rs               # 页式存储管理
│   ├── buffer.rs             # 缓存管理
│   ├── file.rs               # 文件系统接口
│   └── index.rs              # 索引管理
├── engine/                   # 数据库引擎模块
│   ├── mod.rs                # 引擎模块定义
│   ├── database.rs           # 数据库核心实现
│   ├── executor.rs           # 查询执行器
│   ├── table.rs              # 表管理
│   ├── transaction.rs        # 事务管理
│   └── tests.rs              # 引擎测试
├── types/                    # 类型定义模块
│   └── mod.rs                # 数据类型定义
└── utils/                    # 工具模块
    ├── mod.rs                # 工具模块定义
    ├── bitset.rs             # 位集合工具
    ├── hash.rs               # 哈希工具
    └── serialize.rs          # 序列化工具
```

## 🚀 核心功能

### SQL 编译器
- **词法分析**: 支持 SQL 关键字、标识符、数字、字符串字面量
- **语法分析**: 递归下降解析器，支持基本 SQL 语句
- **语义分析**: 类型检查和作用域解析
- **执行计划**: 查询优化和执行计划生成

### 存储系统
- **页式存储**: 4KB 固定大小页面管理
- **缓存管理**: LRU 替换算法
- **文件管理**: 表文件和元数据管理
- **数据持久化**: JSON 格式数据存储

### 查询引擎
- **表管理**: CREATE/DROP TABLE 操作
- **数据操作**: INSERT/SELECT/UPDATE/DELETE 操作
- **WHERE 条件**: 基本条件查询支持
- **列投影**: SELECT 指定列功能

## 📊 数据持久化

### 存储格式
- `metadata.json`: 数据库元数据(表目录、下一个表ID)
- `table_N.json`: 表N的模式和数据，JSON格式

### 持久化策略
- 自动持久化: CRUD操作后自动保存数据
- 启动恢复: 数据库启动时自动加载现有数据
- 容错处理: 持久化失败不影响正常操作

## 🧪 测试系统

### 核心测试脚本
- `comprehensive_test.ps1`: 完整功能测试套件
- `test_persistence.ps1`: 数据持久化专项测试

### 测试覆盖
- DDL 操作: CREATE TABLE, DROP TABLE
- DML 操作: INSERT, SELECT, UPDATE, DELETE
- 高级功能: WHERE 条件, 列投影, 数据类型转换
- 持久化: 重启数据恢复验证
- 性能: 基础性能指标测试

## 📋 支持的 SQL 语法

### DDL (数据定义语言)
```sql
CREATE TABLE table_name (
    column_name data_type [NULL|NOT NULL],
    ...
);
DROP TABLE table_name;
```

### DML (数据操作语言)
```sql
INSERT INTO table_name VALUES (value1, value2, ...);
SELECT [column_list|*] FROM table_name [WHERE condition];
UPDATE table_name SET column = value [WHERE condition];
DELETE FROM table_name [WHERE condition];
```

### 支持的数据类型
- `INTEGER`: 32位整数
- `BIGINT`: 64位整数
- `FLOAT`: 32位浮点数
- `DOUBLE`: 64位浮点数
- `VARCHAR(n)`: 可变长度字符串
- `BOOLEAN`: 布尔值
- `DATE`: 日期
- `TIMESTAMP`: 时间戳

## 🔧 开发环境

### 必需依赖
- Rust 1.70+
- PowerShell (Windows测试)
- serde + serde_json (序列化)
- thiserror (错误处理)

### 构建运行
```bash
# 构建项目
cargo build

# 运行交互式shell
cargo run

# 运行测试
.\comprehensive_test.ps1
```

## 📈 性能特点

- **内存存储**: 主要数据存储在内存HashMap中
- **即时持久化**: 每次修改立即写入文件
- **JSON格式**: 易于调试和检查的存储格式
- **单线程**: 当前版本为单线程设计

## 🎯 设计目标

1. **教学价值**: 展示数据库系统核心概念
2. **代码清晰**: 易于理解和修改的代码结构
3. **功能完整**: 支持基本的数据库操作
4. **数据安全**: 可靠的数据持久化机制

## 🚧 未来扩展

- 索引系统完善
- 事务处理增强
- 多用户并发支持
- 查询优化改进
- 网络接口支持

---

**MiniDB** - 一个简洁而功能完整的教学数据库系统 🎓