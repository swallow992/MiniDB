# MiniDB - Rust数据库系统

一个用Rust构建的小型数据库系统，用于学习数据库内核技术。

## 🚀 项目状态

### ✅ 已完成模块

- **SQL编译器** (100% 完成)
  - 词法分析器 (Lexer) - 8个测试 ✅
  - 语法分析器 (Parser) - 9个测试 ✅  
  - 语义分析器 (Analyzer) - 14个测试 ✅
  - 执行计划生成器 (Planner) - 7个测试 ✅

### 🚧 待开发模块

- 存储引擎 (Storage Engine)
- 查询执行器 (Query Executor)  
- 事务处理 (Transaction Manager)
- 索引系统 (Index System)

## 📋 快速测试

### 30秒快速验证
```bash
cd D:\repositories\MniDB
cargo build
cargo test
# 期望结果: 38 passed; 0 failed
```

### 详细测试指南
- **完整教程**: [TESTING_GUIDE.md](TESTING_GUIDE.md) - 详细的测试说明
- **快速测试**: [QUICK_TEST.md](QUICK_TEST.md) - 5分钟快速验证  
- **验证清单**: [TEST_CHECKLIST.md](TEST_CHECKLIST.md) - 逐项验证功能

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

### 核心组件
- **Lexer**: 词法分析，支持所有SQL关键字和操作符
- **Parser**: 递归下降语法分析器，构建完整AST
- **Analyzer**: 语义分析和类型检查，支持模式验证
- **Planner**: 查询优化和执行计划生成

## 📊 测试覆盖

| 模块 | 测试数量 | 覆盖功能 |
|------|----------|----------|
| Lexer | 8 | 关键字、标识符、字面量、运算符、注释 |
| Parser | 9 | 所有SQL语句、表达式解析 |
| Analyzer | 14 | 类型检查、语义验证、错误检测 |
| Planner | 7 | 执行计划生成、查询优化准备 |
| **总计** | **38** | **100%核心功能覆盖** |

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

1. **存储系统** - 实现页式存储和缓冲池
2. **执行器** - 实现查询执行引擎
3. **事务处理** - 添加ACID事务支持  
4. **索引系统** - 实现B+树索引
5. **更多SQL特性** - JOIN、GROUP BY、ORDER BY等

## 📖 学习资源

这个项目适合学习：
- Rust系统编程
- 数据库内核设计
- 编译器前端技术
- 查询处理算法

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

MIT License

---

**当前版本**: v0.1.0  
**开发状态**: SQL编译器完成，存储引擎开发中  
**测试状态**: 38/38 通过 ✅