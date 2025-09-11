# MiniDB 测试指南

这份文档将指导你如何手动测试 MiniDB 项目的各个功能模块。

## 📋 测试准备

### 1. 确保环境就绪

首先确保你已经安装了 Rust 开发环境：

```bash
# 检查 Rust 版本
rustc --version

# 检查 Cargo 版本  
cargo --version
```

### 2. 克隆和编译项目

```bash
# 进入项目目录
cd D:\repositories\MniDB

# 编译项目
cargo build

# 编译成功后应该看到类似输出：
#    Compiling minidb v0.1.0
#    Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

## 🧪 测试模块详解

我们的 SQL 编译器由四个核心模块组成，每个模块都有完整的测试覆盖。

### 1. 词法分析器 (Lexer) 测试

词法分析器负责将 SQL 文本分解为 token。

**运行测试：**
```bash
cargo test lexer
```

**预期结果：**
```
running 8 tests
test sql::lexer::tests::test_comments ... ok
test sql::lexer::tests::test_identifiers ... ok  
test sql::lexer::tests::test_keywords ... ok
test sql::lexer::tests::test_numbers ... ok
test sql::lexer::tests::test_operators ... ok
test sql::lexer::tests::test_punctuation ... ok
test sql::lexer::tests::test_sql_statement ... ok
test sql::lexer::tests::test_strings ... ok

test result: ok. 8 passed; 0 failed
```

**测试覆盖的功能：**
- ✅ SQL关键字识别 (SELECT, FROM, WHERE等)
- ✅ 标识符解析 (表名、列名)
- ✅ 数字解析 (整数、浮点数)
- ✅ 字符串解析 (包括转义字符)
- ✅ 运算符解析 (+, -, *, /, =, <>, <, <=, >, >=)
- ✅ 标点符号 (括号、逗号、分号等)
- ✅ 注释处理 (单行 -- 和块注释 /* */)

### 2. 语法分析器 (Parser) 测试

语法分析器将 token 序列解析为抽象语法树 (AST)。

**运行测试：**
```bash
cargo test parser
```

**预期结果：**
```
running 9 tests
test sql::parser::tests::test_complex_expression ... ok
test sql::parser::tests::test_create_table ... ok
test sql::parser::tests::test_delete ... ok
test sql::parser::tests::test_drop_table ... ok
test sql::parser::tests::test_insert ... ok
test sql::parser::tests::test_select_simple ... ok
test sql::parser::tests::test_select_with_columns ... ok
test sql::parser::tests::test_select_with_where ... ok
test sql::parser::tests::test_update ... ok

test result: ok. 9 passed; 0 failed
```

**测试覆盖的 SQL 语句：**
- ✅ `CREATE TABLE` - 创建表（包括列定义和约束）
- ✅ `DROP TABLE` - 删除表
- ✅ `SELECT` - 查询语句（简单查询、列选择、WHERE条件）
- ✅ `INSERT` - 插入语句（包括多行插入）
- ✅ `UPDATE` - 更新语句（包括WHERE条件）
- ✅ `DELETE` - 删除语句（包括WHERE条件）
- ✅ 复杂表达式解析（嵌套括号、运算符优先级）

### 3. 语义分析器 (Analyzer) 测试

语义分析器执行类型检查和语义验证。

**运行测试：**
```bash
cargo test analyzer
```

**预期结果：**
```
running 14 tests
test sql::analyzer::tests::test_analyze_binary_operations ... ok
test sql::analyzer::tests::test_analyze_create_table ... ok
test sql::analyzer::tests::test_analyze_delete_valid ... ok
test sql::analyzer::tests::test_analyze_duplicate_table ... ok
test sql::analyzer::tests::test_analyze_expression_types ... ok
test sql::analyzer::tests::test_analyze_insert_column_mismatch ... ok
test sql::analyzer::tests::test_analyze_insert_invalid_column ... ok
test sql::analyzer::tests::test_analyze_insert_valid ... ok
test sql::analyzer::tests::test_analyze_select_invalid_column ... ok
test sql::analyzer::tests::test_analyze_select_invalid_table ... ok
test sql::analyzer::tests::test_analyze_select_type_mismatch ... ok
test sql::analyzer::tests::test_analyze_select_valid ... ok
test sql::analyzer::tests::test_analyze_update_invalid_column ... ok
test sql::analyzer::tests::test_analyze_update_valid ... ok

test result: ok. 14 passed; 0 failed
```

**测试覆盖的验证功能：**
- ✅ 表存在性检查
- ✅ 列存在性检查  
- ✅ 类型兼容性检查
- ✅ 重复表名检测
- ✅ 插入列数匹配验证
- ✅ 表达式类型推导
- ✅ WHERE条件必须是布尔类型
- ✅ 二元运算类型检查

### 4. 执行计划生成器 (Planner) 测试

执行计划生成器将语义分析后的 AST 转换为可执行的查询计划。

**运行测试：**
```bash
cargo test planner
```

**预期结果：**
```
running 7 tests
test sql::planner::tests::test_plan_create_table ... ok
test sql::planner::tests::test_plan_delete ... ok
test sql::planner::tests::test_plan_drop_table ... ok
test sql::planner::tests::test_plan_insert ... ok
test sql::planner::tests::test_plan_select_wildcard ... ok
test sql::planner::tests::test_plan_select_with_where ... ok
test sql::planner::tests::test_plan_update ... ok

test result: ok. 7 passed; 0 failed
```

**测试覆盖的执行计划：**
- ✅ CREATE TABLE 计划生成
- ✅ DROP TABLE 计划生成
- ✅ SELECT 计划生成（包括投影和过滤）
- ✅ INSERT 计划生成
- ✅ UPDATE 计划生成
- ✅ DELETE 计划生成
- ✅ 通配符 (*) 投影处理

## 🎯 完整测试流程

### 运行所有测试

```bash
cargo test
```

**预期输出：**
```
running 38 tests
[... 所有测试项目 ...]

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 测试指标

- **总测试数量**: 38 个
- **覆盖模块**: 4 个核心模块
- **通过率**: 100%

## 🔍 问题排查

### 常见问题

**1. 编译失败**
```bash
# 如果看到编译错误，请检查 Rust 版本
rustc --version
# 推荐使用 1.70+ 版本
```

**2. 测试失败**
```bash
# 查看详细错误信息
cargo test -- --nocapture

# 运行特定测试
cargo test test_name_here

# 查看测试覆盖详情
cargo test --verbose
```

**3. 警告信息**
项目可能显示一些未使用导入的警告，这是正常的，不影响功能：
```
warning: unused imports: `analyze_statement`, `create_plan`, and `parse_sql`
warning: unused imports: `BufferPool` and `FileManager`
```

### 验证关键功能

**手动验证词法分析：**
```bash
# 你可以查看 src/sql/lexer.rs 中的测试来理解支持的语法
```

**手动验证语法分析：**
```bash
# 查看 src/sql/parser.rs 中的测试来理解支持的 SQL 语句格式
```

## 📚 测试数据示例

我们的测试使用以下示例数据结构：

**用户表结构：**
```sql
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    age INT,
    email VARCHAR(255)
);
```

**测试的 SQL 语句示例：**
```sql
-- 基本查询
SELECT * FROM users;
SELECT id, name FROM users;

-- 条件查询  
SELECT * FROM users WHERE age > 18;
SELECT * FROM users WHERE age > 18 AND name = 'Alice';

-- 数据修改
INSERT INTO users (name, age) VALUES ('Alice', 25);
INSERT INTO users (name, age) VALUES ('Alice', 25), ('Bob', 30);

UPDATE users SET age = 26 WHERE name = 'Alice';

DELETE FROM users WHERE age < 18;

-- DDL 操作
CREATE TABLE test (id INT PRIMARY KEY, name VARCHAR NOT NULL);
DROP TABLE test;
```

## 🎉 测试成功标志

当你看到所有测试都通过时，说明：

1. ✅ **词法分析器** 可以正确解析各种 SQL token
2. ✅ **语法分析器** 可以构建正确的 AST
3. ✅ **语义分析器** 可以进行类型检查和语义验证  
4. ✅ **执行计划生成器** 可以生成可执行的查询计划

这意味着 MiniDB 的 SQL 编译器前端已经完全就绪，可以处理复杂的 SQL 语句！

## 🚀 下一步

现在 SQL 编译器已经完成，你可以：

1. 尝试添加新的 SQL 语句支持
2. 继续开发存储引擎
3. 实现查询执行器
4. 添加更多的数据类型支持

---

**祝你测试愉快！** 🎊

如果遇到任何问题，请检查控制台输出的详细错误信息，或者查看对应的测试代码来理解预期行为。
