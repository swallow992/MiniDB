# MiniDB 测试数据清理脚本 🧹

## 🎯 清理目的

这个脚本帮助你快速清理所有测试过程中创建的表和数据，让数据库恢复到初始状态。

---

## 🚀 如何使用清理脚本

### 启动 MiniDB
```bash
# 启动 MiniDB 交互式命令行
cargo run --bin minidb
```

### 执行清理命令
在 `minidb>` 提示符下，复制粘贴以下 SQL 命令：

---

## 🧹 完整清理脚本

### 第一步：删除所有测试表
```sql
-- 删除用户相关表
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS test_users;

-- 删除订单相关表  
DROP TABLE IF EXISTS orders;

-- 删除部门和员工表
DROP TABLE IF EXISTS departments;
DROP TABLE IF EXISTS employees;

-- 删除测试用的空表
DROP TABLE IF EXISTS empty_test;
DROP TABLE IF EXISTS empty_table;

-- 删除其他可能创建的测试表
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS test_table;
```

### 第二步：验证表已被删除
```sql
-- 尝试查询已删除的表，应该返回"表不存在"错误
SELECT * FROM users;        -- 预期：Table 'users' not found
SELECT * FROM orders;       -- 预期：Table 'orders' not found  
SELECT * FROM departments;  -- 预期：Table 'departments' not found
```

### 第三步：确认清理完成
```sql
-- 如果你的 MiniDB 支持查看所有表的功能，可以用以下命令验证
-- (注意：这个命令可能不被支持，取决于你的实现)
SHOW TABLES;  -- 预期：返回空结果或报错
```

---

## 🔄 重新开始测试

清理完成后，如果你想重新开始测试，可以使用以下命令重建基础测试环境：

### 重建基础表结构
```sql
-- 重新创建用户表
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    age INT,
    department VARCHAR(50)
);

-- 重新创建订单表
CREATE TABLE orders (
    order_id INT PRIMARY KEY,
    user_id INT,
    product_name VARCHAR(100),
    price DOUBLE
);

-- 插入基础测试数据
INSERT INTO users VALUES (1, 'Alice', 'alice@test.com', 25, 'Engineering');
INSERT INTO users VALUES (2, 'Bob', 'bob@test.com', 30, 'Sales');
INSERT INTO users VALUES (3, 'Charlie', 'charlie@test.com', 28, 'Engineering');

INSERT INTO orders VALUES (101, 1, 'Laptop Pro', 2999.99);
INSERT INTO orders VALUES (102, 2, 'Mouse Wireless', 89.99);
INSERT INTO orders VALUES (103, 1, 'Keyboard Mechanical', 199.99);
```

---

## 🚨 故障排除

### 如果 DROP TABLE 命令失败：

#### 问题 1：表不存在
```
错误：Table 'tablename' not found
解决：这是正常的，说明表已经不存在了
```

#### 问题 2：语法不支持
```
错误：DROP TABLE command not supported
解决：你的 MiniDB 可能还没有实现 DROP TABLE 功能
```

#### 问题 3：无法删除表
```
错误：Cannot drop table
解决方案：
1. 重启 MiniDB: 退出程序然后重新运行 cargo run --bin minidb
2. 手动删除数据文件: 删除 MiniDB 的数据存储文件
3. 重新编译: cargo clean && cargo build --bin minidb
```

---

## 🛠️ 手动清理方法

如果 SQL 清理命令不工作，你可以使用以下方法：

### 方法 1：重启 MiniDB
```bash
# 1. 退出当前 MiniDB 会话 (输入 quit 或 Ctrl+C)
# 2. 重新启动
cargo run --bin minidb
```

### 方法 2：重新编译
```bash
# 清理编译缓存并重新编译
cargo clean
cargo build --bin minidb
cargo run --bin minidb
```

### 方法 3：删除数据文件（如果有）
```bash
# 如果 MiniDB 将数据持久化到文件中，删除这些文件
# 查找可能的数据文件
ls *.db
ls *.data
ls data/*

# 删除数据文件（小心操作）
rm -f *.db *.data
rm -rf data/
```

---

## 📋 清理检查清单

完成清理后，确保：

- [ ] **所有 DROP TABLE 命令成功执行**
- [ ] **查询已删除的表时返回"表不存在"错误**
- [ ] **MiniDB 仍能正常响应新的 CREATE TABLE 命令**
- [ ] **可以重新创建和使用新表**
- [ ] **系统运行稳定，没有异常错误**

---

## ⏱️ 快速清理（一键执行）

如果你想要最快的清理，复制以下完整脚本到 MiniDB 命令行：

```sql
-- === 快速清理脚本 ===
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS departments;
DROP TABLE IF EXISTS employees;
DROP TABLE IF EXISTS test_users;
DROP TABLE IF EXISTS empty_test;
DROP TABLE IF EXISTS empty_table;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS test_table;

-- 验证清理完成
SELECT * FROM users;  -- 应该报错
```

---

## 🎉 清理完成

当你看到所有表查询都返回"表不存在"错误时，恭喜！你的 MiniDB 已经成功清理，可以开始新的测试了。

---

*清理脚本版本: v1.0*  
*创建日期: 2025年9月16日*  
*适用于: MiniDB v0.1.0*  
*预计清理时间: 1-2 分钟*