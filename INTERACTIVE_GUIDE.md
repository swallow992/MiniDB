# MiniDB 交互式使用指南

## 概述

MiniDB 现在支持完全交互式的命令行界面，提供丰富的反馈信息和调试功能。您可以通过具体的命令获得具体的回复，而不是简单的 OK/FAIL 响应。

## 启动数据库

```bash
# 使用默认数据目录 (./minidb_data)
cargo run

# 或者指定自定义数据目录
cargo run -- /path/to/your/database
```

## 系统命令

### 基础命令
- `help` 或 `\h` - 显示完整的帮助信息
- `quit`, `exit` 或 `\q` - 退出程序
- `clear` 或 `\c` - 清空屏幕

### 查看命令
- `\d` - 列出所有表（待实现）
- `\s` - 显示系统状态
- `\i` - 显示内部系统信息
- `\version` - 显示版本信息

### 测试命令
- `\t` - 运行快速功能测试

## SQL 命令

支持以下 SQL 语句：

### 表管理
```sql
-- 创建表
CREATE TABLE users (id INT, name VARCHAR(50), age INT);

-- 删除表
DROP TABLE users;
```

### 数据操作
```sql
-- 插入数据
INSERT INTO users VALUES (1, 'Alice', 25);
INSERT INTO users VALUES (2, 'Bob', 30);

-- 查询数据
SELECT * FROM users;
SELECT name FROM users WHERE age > 25;

-- 更新数据
UPDATE users SET age = 26 WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE age < 25;
```

## 输出格式说明

### 成功执行
当 SQL 命令成功执行时，您会看到：
```
✅ 查询执行成功!
⏱️  执行时间: 1.23ms
💬 消息: [系统消息]
📊 查询结果:
═══════════════════════════════════════
[格式化的表格数据]
═══════════════════════════════════════
📈 总共 X 行数据
🔄 影响行数: Y
```

### 执行失败
当命令失败时，您会看到：
```
❌ 查询执行失败!
⏱️  执行时间: 0.25ms
🚨 错误信息: [详细错误描述]
💡 提示: [相关的解决建议]
```

### 提示类型
- **语法错误**: "请检查SQL语法是否正确"
- **表不存在**: "表不存在，请使用 \d 查看可用的表"
- **列不存在**: "列不存在，请检查列名是否正确"

## 示例会话

```
=== MiniDB Interactive Shell v0.1.0 ===
欢迎使用 MiniDB！
输入 'help' 查看可用命令，输入 'quit' 退出。

正在打开数据库: ./minidb_data
数据库已成功加载！

minidb> CREATE TABLE users (id INT, name VARCHAR(50), age INT);
📝 执行SQL: CREATE TABLE users (id INT, name VARCHAR(50), age INT);
✅ 查询执行成功!
⏱️  执行时间: 2.15ms
💬 消息: Table 'users' created successfully

minidb> INSERT INTO users VALUES (1, 'Alice', 25);
📝 执行SQL: INSERT INTO users VALUES (1, 'Alice', 25);
✅ 查询执行成功!
⏱️  执行时间: 1.05ms
🔄 影响行数: 1

minidb> SELECT * FROM users;
📝 执行SQL: SELECT * FROM users;
✅ 查询执行成功!
⏱️  执行时间: 0.85ms
📊 查询结果:
═══════════════════════════════════════
          id │         name │          age
       (INT) │  (VARCHAR50) │        (INT)
─────────────┼──────────────┼─────────────
           1 │        Alice │           25
═══════════════════════════════════════
📈 总共 1 行数据

minidb> \s
=== 系统状态 ===
数据库引擎: MiniDB v0.1.0
缓冲池状态: (待实现)
内存使用: (待实现)
活跃连接: 1

minidb> quit
再见！感谢使用 MiniDB!
```

## 调试功能

### 内部信息 (`\i`)
显示数据库引擎的内部配置：
- 数据库版本
- 数据目录位置
- 页面大小和缓冲池配置
- 编译模式信息

### 快速测试 (`\t`)
自动运行一系列测试命令来验证基本功能：
- 创建测试表
- 插入测试数据
- 查询测试数据

### 版本信息 (`\version`)
显示详细的版本和构建信息：
- MiniDB 版本号
- Rust 编译器信息
- 目标平台
- 项目描述

## 自动化测试

### 交互式测试脚本
运行完整的功能测试：
```bash
.\test_interactive.ps1
```

### 调试命令测试
测试调试相关命令：
```bash
.\test_debug.ps1
```

### 手动交互式演示
启动交互式演示：
```bash
.\demo_interactive.ps1
```

## 性能监控

每个 SQL 命令都会显示执行时间，帮助您：
- 监控查询性能
- 识别慢查询
- 优化数据库操作

## 错误处理

系统提供智能错误提示：
- 详细的错误描述
- 基于错误类型的建议
- 执行时间统计（即使失败也会显示）

## 下一步功能

当前待实现的功能：
- `\d` 命令的完整表列表功能
- 系统状态的详细缓冲池信息
- 更复杂的 SQL 语句支持
- 事务管理
- 索引支持

---

*这个交互式界面让您能够实时看到数据库的操作结果，提供比传统测试集更直观的使用体验。*