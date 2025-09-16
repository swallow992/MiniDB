# MiniDB 手动测试指南 🧪

## 🎯 测试目标

这份文档提供完整的手动测试用例，让你能够逐步验证 MiniDB 数据库系统的所有功能。所有测试都可以通过交互式命令行进行。

---

## 🚀 开始测试

### 启动 MiniDB
```bash
# 编译并启动 MiniDB
cargo run --bin minidb

# 你会看到欢迎信息
MiniDB v0.1.0 - 小型数据库系统
输入 SQL 语句进行测试，输入 'quit' 退出
minidb>
```

---

## 📋 完整测试流程

### 阶段 1: 基础 DDL 操作测试

#### 测试 1.1: 创建基础表
```sql
-- 在 minidb> 提示符下输入以下命令

-- 创建用户表
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    age INT,
    department VARCHAR(50)
);
```
**预期结果**: 显示 "Table 'users' created successfully"

#### 测试 1.2: 创建关联表
```sql
-- 创建部门表
CREATE TABLE departments (
    dept_id INT PRIMARY KEY,
    dept_name VARCHAR(50) NOT NULL,
    location VARCHAR(100)
);

-- 创建订单表
CREATE TABLE orders (
    order_id INT PRIMARY KEY,
    user_id INT,
    product_name VARCHAR(100),
    price DOUBLE,
    order_date TIMESTAMP
);
```
**预期结果**: 每个表创建后都显示成功消息

### 阶段 2: 数据插入测试 (INSERT)

#### 测试 2.1: 基础数据插入
```sql
-- 插入用户数据
INSERT INTO users VALUES (1, 'Alice Zhang', 'alice@company.com', 25, 'Engineering');
INSERT INTO users VALUES (2, 'Bob Wang', 'bob@company.com', 30, 'Sales');  
INSERT INTO users VALUES (3, 'Charlie Li', 'charlie@company.com', 28, 'Engineering');
INSERT INTO users VALUES (4, 'Diana Chen', 'diana@company.com', 32, 'Marketing');
INSERT INTO users VALUES (5, 'Eve Liu', 'eve@company.com', 24, 'Sales');
```
**预期结果**: 每次插入后显示 "1 row inserted"

#### 测试 2.2: 部门数据插入
```sql
-- 插入部门数据
INSERT INTO departments VALUES (1, 'Engineering', 'Building A');
INSERT INTO departments VALUES (2, 'Sales', 'Building B');
INSERT INTO departments VALUES (3, 'Marketing', 'Building C');
INSERT INTO departments VALUES (4, 'HR', 'Building D');
```

#### 测试 2.3: 订单数据插入
```sql
-- 插入订单数据
INSERT INTO orders VALUES (101, 1, 'Laptop Pro', 2999.99, '2024-01-15');
INSERT INTO orders VALUES (102, 2, 'Mouse Wireless', 89.99, '2024-01-16');
INSERT INTO orders VALUES (103, 1, 'Keyboard Mechanical', 199.99, '2024-01-17');
INSERT INTO orders VALUES (104, 3, 'Monitor 4K', 599.99, '2024-01-18');
INSERT INTO orders VALUES (105, 4, 'Tablet Pro', 899.99, '2024-01-19');
```

### 阶段 3: 基础查询测试 (SELECT)

#### 测试 3.1: 全表查询
```sql
-- 查看所有用户
SELECT * FROM users;
```
**预期结果**: 
```
| id | name        | email              | age | department  |
|----|-------------|--------------------|-----|-------------|
| 1  | Alice Zhang | alice@company.com  | 25  | Engineering |
| 2  | Bob Wang    | bob@company.com    | 30  | Sales       |
| 3  | Charlie Li  | charlie@company.com| 28  | Engineering |
| 4  | Diana Chen  | diana@company.com  | 32  | Marketing   |
| 5  | Eve Liu     | eve@company.com    | 24  | Sales       |
```

#### 测试 3.2: 列投影查询
```sql
-- 只查看姓名和邮箱
SELECT name, email FROM users;

-- 查看所有部门
SELECT * FROM departments;

-- 查看所有订单
SELECT * FROM orders;
```

#### 测试 3.3: 条件查询 (WHERE 子句)
```sql
-- 查询年龄大于 27 的用户
SELECT * FROM users WHERE age > 27;

-- 查询工程部的员工
SELECT name, age FROM users WHERE department = 'Engineering';

-- 查询价格超过 500 的订单
SELECT order_id, product_name, price FROM orders WHERE price > 500;
```

### 阶段 4: 高级查询功能测试

#### 测试 4.1: 排序功能 (ORDER BY)
```sql
-- 按年龄升序排列
SELECT * FROM users ORDER BY age;

-- 按年龄降序排列  
SELECT * FROM users ORDER BY age DESC;

-- 按部门和年龄排序
SELECT * FROM users ORDER BY department, age DESC;

-- 按价格降序排列订单
SELECT * FROM orders ORDER BY price DESC;
```

#### 测试 4.2: 分页功能 (LIMIT & OFFSET)
```sql
-- 获取前 3 个用户
SELECT * FROM users LIMIT 3;

-- 跳过第一个，获取接下来的 2 个用户
SELECT * FROM users ORDER BY id LIMIT 2 OFFSET 1;

-- 获取最贵的 2 个订单
SELECT * FROM orders ORDER BY price DESC LIMIT 2;
```

#### 测试 4.3: 聚合函数测试 (GROUP BY)
```sql
-- 统计每个部门的人数
SELECT department, COUNT(*) as emp_count FROM users GROUP BY department;

-- 计算每个部门的平均年龄
SELECT department, AVG(age) as avg_age FROM users GROUP BY department;

-- 统计每个用户的订单总数和总金额
SELECT user_id, COUNT(*) as order_count, SUM(price) as total_amount 
FROM orders GROUP BY user_id;

-- 测试所有聚合函数
SELECT department, 
       COUNT(*) as count,
       AVG(age) as avg_age,
       MAX(age) as max_age,
       MIN(age) as min_age
FROM users GROUP BY department;
```

### 阶段 5: 复杂查询测试 (JOIN)

#### 测试 5.1: 内连接 (INNER JOIN)
```sql
-- 关联用户和订单（需要手动实现简化版本）
-- 注意: 如果 JOIN 语法不支持，可以用子查询模拟
SELECT u.name, o.product_name, o.price 
FROM users u, orders o 
WHERE u.id = o.user_id;
```

#### 测试 5.2: 复杂条件查询
```sql
-- 查询购买了超过 1000 元商品的用户
SELECT DISTINCT u.name, u.email 
FROM users u, orders o 
WHERE u.id = o.user_id AND o.price > 1000;

-- 查询工程部员工的所有订单
SELECT u.name, u.department, o.product_name, o.price
FROM users u, orders o 
WHERE u.id = o.user_id AND u.department = 'Engineering';
```

### 阶段 6: 数据修改测试 (UPDATE)

#### 测试 6.1: 单行更新
```sql
-- 更新 Alice 的年龄
UPDATE users SET age = 26 WHERE name = 'Alice Zhang';

-- 验证更新结果
SELECT * FROM users WHERE name = 'Alice Zhang';
```

#### 测试 6.2: 批量更新
```sql
-- 给所有销售部员工涨一岁
UPDATE users SET age = age + 1 WHERE department = 'Sales';

-- 验证更新结果
SELECT * FROM users WHERE department = 'Sales';

-- 更新订单价格（打 9 折）
UPDATE orders SET price = price * 0.9 WHERE price > 500;
```

### 阶段 7: 数据删除测试 (DELETE)

#### 测试 7.1: 条件删除
```sql
-- 先查看当前数据
SELECT * FROM orders;

-- 删除价格最低的订单
DELETE FROM orders WHERE price = (SELECT MIN(price) FROM orders);

-- 验证删除结果
SELECT * FROM orders;
```

#### 测试 7.2: 批量删除
```sql
-- 删除价格低于 100 的订单
DELETE FROM orders WHERE price < 100;

-- 验证结果
SELECT COUNT(*) FROM orders;
SELECT * FROM orders;
```

### 阶段 8: 错误处理测试

#### 测试 8.1: 语法错误测试
```sql
-- 故意的语法错误
SELEC * FROM users;  -- 拼写错误
SELECT * FROM;       -- 缺少表名
INSERT INTO users;   -- 缺少值
CREATE TABLE;        -- 语法不完整
```
**预期结果**: 应该显示清晰的错误消息和位置

#### 测试 8.2: 语义错误测试
```sql
-- 表不存在
SELECT * FROM non_existent_table;

-- 列不存在
SELECT invalid_column FROM users;

-- 类型不匹配
INSERT INTO users VALUES ('abc', 'Alice', 'email', 25, 'dept');  -- id 应该是数字
```

#### 测试 8.3: 约束违规测试
```sql
-- 主键重复
INSERT INTO users VALUES (1, 'Duplicate User', 'dup@email.com', 30, 'IT');

-- 空值测试（如果有 NOT NULL 约束）
INSERT INTO users (id, email, age) VALUES (10, 'test@email.com', 25);  -- 缺少 name
```

### 阶段 9: 边界条件测试

#### 测试 9.1: 空表查询
```sql
-- 创建空表并查询
CREATE TABLE empty_table (id INT, name VARCHAR(50));
SELECT * FROM empty_table;
SELECT COUNT(*) FROM empty_table;
```

#### 测试 9.2: 大量数据测试
```sql
-- 插入多条相似数据
INSERT INTO users VALUES (10, 'User 10', 'user10@email.com', 25, 'IT');
INSERT INTO users VALUES (11, 'User 11', 'user11@email.com', 26, 'IT');
INSERT INTO users VALUES (12, 'User 12', 'user12@email.com', 27, 'IT');
INSERT INTO users VALUES (13, 'User 13', 'user13@email.com', 28, 'IT');
INSERT INTO users VALUES (14, 'User 14', 'user14@email.com', 29, 'IT');

-- 测试大数据量查询
SELECT * FROM users ORDER BY id;
SELECT department, COUNT(*) FROM users GROUP BY department;
```

#### 测试 9.3: 极限字符串测试
```sql
-- 测试长字符串
INSERT INTO users VALUES (20, 'Very Long Name That Tests String Handling Capabilities', 'very.long.email.address@company.example.com', 30, 'Research');

-- 测试特殊字符
INSERT INTO users VALUES (21, 'O''Connor & Smith-Johnson', 'o.connor@email.com', 35, 'Legal');
```

---

## 🎯 测试验证清单

### 基础功能验证 ✅
- [ ] DDL: CREATE TABLE 成功创建表结构
- [ ] INSERT: 数据成功插入，返回正确行数
- [ ] SELECT: 全表查询返回所有数据
- [ ] WHERE: 条件过滤返回正确结果
- [ ] UPDATE: 数据更新成功，影响正确行数  
- [ ] DELETE: 数据删除成功，影响正确行数

### 高级功能验证 ✅
- [ ] ORDER BY: 单列和多列排序正确
- [ ] LIMIT/OFFSET: 分页功能工作正常
- [ ] GROUP BY: 分组统计结果正确
- [ ] 聚合函数: COUNT, SUM, AVG, MAX, MIN 正确计算
- [ ] 复杂查询: 多表关联查询正确执行
- [ ] 组合查询: WHERE + ORDER BY + LIMIT 组合正确

### 错误处理验证 ✅  
- [ ] 语法错误: 显示清晰错误信息
- [ ] 语义错误: 检测不存在的表/列
- [ ] 类型错误: 检测类型不匹配
- [ ] 约束错误: 检测主键重复等约束违规
- [ ] 边界条件: 空表、特殊字符正确处理

### 性能验证 ✅
- [ ] 响应速度: 基本查询在 1 秒内返回
- [ ] 内存使用: 大量数据不导致内存溢出
- [ ] 并发处理: 多个查询不互相干扰
- [ ] 数据一致性: 修改操作保持数据完整性

---

## 📊 测试结果记录模板

```
测试日期: ___________
测试环境: Windows 11, Rust 1.75+
测试执行者: ___________

功能测试结果:
├── DDL 操作 
│   ├── CREATE TABLE: ✅/❌ 
│   └── 表结构验证: ✅/❌
├── DML 操作
│   ├── INSERT: ✅/❌ 
│   ├── SELECT: ✅/❌
│   ├── UPDATE: ✅/❌
│   └── DELETE: ✅/❌
├── 查询功能
│   ├── WHERE 条件: ✅/❌
│   ├── ORDER BY: ✅/❌
│   ├── LIMIT/OFFSET: ✅/❌
│   └── GROUP BY: ✅/❌
├── 聚合函数
│   ├── COUNT: ✅/❌
│   ├── SUM/AVG: ✅/❌
│   └── MAX/MIN: ✅/❌
├── 错误处理
│   ├── 语法错误: ✅/❌
│   ├── 语义错误: ✅/❌
│   └── 约束检查: ✅/❌
└── 性能表现
    ├── 响应速度: ✅/❌
    └── 稳定性: ✅/❌

总体评价: ___________
通过率: ___/___
```

---

## 🚀 快速测试脚本

如果你想快速验证所有功能，可以将以下 SQL 语句保存为文件，然后逐行执行：

```sql
-- 快速功能验证脚本
-- 1. 创建测试环境
CREATE TABLE test_users (id INT PRIMARY KEY, name VARCHAR(50), age INT, dept VARCHAR(30));
INSERT INTO test_users VALUES (1, 'Alice', 25, 'IT'), (2, 'Bob', 30, 'Sales'), (3, 'Charlie', 28, 'IT');

-- 2. 基础查询测试  
SELECT * FROM test_users;
SELECT name, age FROM test_users WHERE age > 26;
SELECT * FROM test_users ORDER BY age DESC;
SELECT dept, COUNT(*) FROM test_users GROUP BY dept;

-- 3. 数据修改测试
UPDATE test_users SET age = 26 WHERE name = 'Alice';
SELECT * FROM test_users WHERE name = 'Alice';
DELETE FROM test_users WHERE age < 27;
SELECT COUNT(*) FROM test_users;

-- 4. 错误测试
SELECT * FROM non_existent_table; -- 应该报错
SELECT invalid_column FROM test_users; -- 应该报错
```

---

## 🎯 测试完成标志

当你成功完成所有测试用例并且：
- ✅ 所有基础 CRUD 操作正常工作
- ✅ 高级查询功能（ORDER BY, GROUP BY, LIMIT）正确执行  
- ✅ 聚合函数返回预期结果
- ✅ 错误处理提供清晰的错误信息
- ✅ 性能表现满足预期

**恭喜！你已经成功验证了 MiniDB 的所有核心功能！** 🎉

---

*手动测试指南版本: v1.0*  
*更新日期: 2025年9月16日*  
*适用于: MiniDB v0.1.0*