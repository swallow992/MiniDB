# MiniDB 快速测试脚本

## 🚀 一键测试所有功能

将以下 SQL 语句依次复制粘贴到 MiniDB 命令行中进行测试：

### 阶段 1: 环境准备
```sql
-- 创建用户表
CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100), email VARCHAR(255), age INT, department VARCHAR(50));

-- 创建订单表
CREATE TABLE orders (order_id INT PRIMARY KEY, user_id INT, product_name VARCHAR(100), price DOUBLE);

-- 创建部门表
CREATE TABLE departments (dept_id INT PRIMARY KEY, dept_name VARCHAR(50), location VARCHAR(100));
```

### 阶段 2: 数据准备
```sql
-- 插入用户数据
INSERT INTO users VALUES (1, 'Alice Zhang', 'alice@company.com', 25, 'Engineering');
INSERT INTO users VALUES (2, 'Bob Wang', 'bob@company.com', 30, 'Sales');
INSERT INTO users VALUES (3, 'Charlie Li', 'charlie@company.com', 28, 'Engineering');
INSERT INTO users VALUES (4, 'Diana Chen', 'diana@company.com', 32, 'Marketing');
INSERT INTO users VALUES (5, 'Eve Liu', 'eve@company.com', 24, 'Sales');

-- 插入订单数据
INSERT INTO orders VALUES (101, 1, 'Laptop Pro', 2999.99);
INSERT INTO orders VALUES (102, 2, 'Mouse Wireless', 89.99);
INSERT INTO orders VALUES (103, 1, 'Keyboard Mechanical', 199.99);
INSERT INTO orders VALUES (104, 3, 'Monitor 4K', 599.99);
INSERT INTO orders VALUES (105, 4, 'Tablet Pro', 899.99);

-- 插入部门数据
INSERT INTO departments VALUES (1, 'Engineering', 'Building A');
INSERT INTO departments VALUES (2, 'Sales', 'Building B');
INSERT INTO departments VALUES (3, 'Marketing', 'Building C');
```

### 阶段 3: 基础查询测试
```sql
-- 测试 SELECT *
SELECT * FROM users;

-- 测试列投影
SELECT name, email FROM users;

-- 测试 WHERE 条件
SELECT * FROM users WHERE age > 27;

-- 测试 WHERE 字符串匹配
SELECT * FROM users WHERE department = 'Engineering';
```

### 阶段 4: 高级查询测试
```sql
-- 测试 ORDER BY 升序
SELECT * FROM users ORDER BY age;

-- 测试 ORDER BY 降序
SELECT * FROM users ORDER BY age DESC;

-- 测试多列排序
SELECT * FROM users ORDER BY department, age DESC;

-- 测试 LIMIT
SELECT * FROM users LIMIT 3;

-- 测试 LIMIT + OFFSET
SELECT * FROM users ORDER BY id LIMIT 2 OFFSET 1;
```

### 阶段 5: 聚合函数测试
```sql
-- 测试 COUNT
SELECT department, COUNT(*) FROM users GROUP BY department;

-- 测试 AVG
SELECT department, AVG(age) FROM users GROUP BY department;

-- 测试所有聚合函数
SELECT department, COUNT(*) as count, AVG(age) as avg_age, MAX(age) as max_age, MIN(age) as min_age FROM users GROUP BY department;

-- 测试订单聚合
SELECT user_id, COUNT(*) as order_count, SUM(price) as total_amount FROM orders GROUP BY user_id;
```

### 阶段 6: 复杂查询测试
```sql
-- 测试复杂条件
SELECT * FROM users WHERE age > 25 AND department = 'Engineering';

-- 测试价格范围查询
SELECT * FROM orders WHERE price > 100 AND price < 1000;

-- 测试组合查询
SELECT department, COUNT(*) as emp_count FROM users WHERE age > 25 GROUP BY department ORDER BY emp_count DESC;
```

### 阶段 7: 数据修改测试
```sql
-- 测试 UPDATE
UPDATE users SET age = 26 WHERE name = 'Alice Zhang';
SELECT * FROM users WHERE name = 'Alice Zhang';

-- 测试批量 UPDATE
UPDATE users SET age = age + 1 WHERE department = 'Sales';
SELECT * FROM users WHERE department = 'Sales';

-- 测试 DELETE
DELETE FROM orders WHERE price < 100;
SELECT COUNT(*) FROM orders;
```

### 阶段 8: 错误处理测试
```sql
-- 语法错误测试
SELEC * FROM users;

-- 表不存在错误
SELECT * FROM non_existent_table;

-- 列不存在错误  
SELECT invalid_column FROM users;

-- 类型错误测试
INSERT INTO users VALUES ('abc', 'Test User', 'email', 25, 'dept');
```

### 阶段 9: 边界条件测试
```sql
-- 空表测试
CREATE TABLE empty_table (id INT, name VARCHAR(50));
SELECT * FROM empty_table;
SELECT COUNT(*) FROM empty_table;

-- 特殊字符测试
INSERT INTO users VALUES (10, 'O''Connor & Smith', 'special@email.com', 35, 'Legal');
SELECT * FROM users WHERE name LIKE '%O''Connor%';
```

## 📋 预期结果检查清单

### ✅ 基础功能
- [ ] CREATE TABLE: 显示 "Table created successfully"
- [ ] INSERT: 显示 "X row(s) inserted" 
- [ ] SELECT *: 返回所有数据行
- [ ] WHERE: 返回符合条件的数据
- [ ] UPDATE: 显示 "X row(s) updated"
- [ ] DELETE: 显示 "X row(s) deleted"

### ✅ 高级功能  
- [ ] ORDER BY: 数据按指定列正确排序
- [ ] LIMIT: 返回指定数量的行
- [ ] OFFSET: 正确跳过指定行数
- [ ] GROUP BY: 按列分组并统计
- [ ] COUNT(*): 返回正确的行数
- [ ] AVG/SUM/MAX/MIN: 返回正确的计算结果

### ✅ 错误处理
- [ ] 语法错误: 显示清晰的错误消息
- [ ] 表不存在: 显示 "Table not found" 类似错误
- [ ] 列不存在: 显示 "Column not found" 类似错误
- [ ] 类型错误: 显示类型不匹配错误

### ✅ 预期数据示例

**用户表查询结果**:
```
| id | name        | email              | age | department  |
|----|-------------|--------------------|-----|-------------|
| 1  | Alice Zhang | alice@company.com  | 25  | Engineering |
| 2  | Bob Wang    | bob@company.com    | 30  | Sales       |
| 3  | Charlie Li  | charlie@company.com| 28  | Engineering |
| 4  | Diana Chen  | diana@company.com  | 32  | Marketing   |
| 5  | Eve Liu     | eve@company.com    | 24  | Sales       |
```

**部门统计结果**:
```
| department  | count | avg_age |
|-------------|-------|---------|
| Engineering |   2   |  26.5   |
| Sales       |   2   |  27.0   |  
| Marketing   |   1   |  32.0   |
```

## 🎯 快速验证要点

1. **启动测试**: `cargo run --bin minidb`
2. **逐步执行**: 复制每个阶段的 SQL 语句
3. **验证结果**: 对比预期输出
4. **记录问题**: 如有异常，记录错误信息
5. **完整测试**: 确保所有阶段都通过

## 📞 故障排除

**如果遇到问题**:
1. 检查 SQL 语法是否正确
2. 确认表是否已创建
3. 验证数据是否已插入
4. 查看错误消息提示
5. 重新编译: `cargo build --bin minidb`

---

*快速测试版本: v1.0*  
*估计测试时间: 15-20 分钟*