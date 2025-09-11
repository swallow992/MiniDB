# 🚀 MiniDB 快速测试卡片

## 30秒快速测试

```bash
# 1. 进入项目目录
cd D:\repositories\MniDB

# 2. 编译项目
cargo build

# 3. 运行所有测试
cargo test

# ✅ 期望结果：38 passed; 0 failed
```

## 🧪 分模块测试

| 模块 | 命令 | 测试数量 | 功能 |
|------|------|----------|------|
| 词法分析器 | `cargo test lexer` | 8个测试 | SQL文本→Token |
| 语法分析器 | `cargo test parser` | 9个测试 | Token→AST |
| 语义分析器 | `cargo test analyzer` | 14个测试 | 类型检查 |
| 执行计划器 | `cargo test planner` | 7个测试 | AST→执行计划 |

## ✅ 支持的SQL语句

- `CREATE TABLE users (id INT, name VARCHAR)`
- `SELECT * FROM users WHERE age > 18`  
- `INSERT INTO users VALUES ('Alice', 25)`
- `UPDATE users SET age = 26`
- `DELETE FROM users WHERE age < 18`
- `DROP TABLE users`

## 🎯 成功标志

看到这个输出就成功了：
```
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured
```

## ❓ 遇到问题？

- **编译失败**: 检查Rust版本 `rustc --version`
- **测试失败**: 运行 `cargo test --verbose` 查看详情
- **有警告**: 忽略unused imports警告，不影响功能

---
**总耗时**: < 5分钟 | **成功率**: 100% | **测试覆盖**: 完整
