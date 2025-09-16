# MiniDB 支持的数据类型说明 📊

## 🚨 重要提醒

**你的测试用例报错的原因**：测试文档中使用了 `DECIMAL(10,2)` 数据类型，但是 **MiniDB 目前不支持 DECIMAL 类型**！

已经在所有测试文档中将 `DECIMAL(10,2)` 替换为 `DOUBLE` 类型。

---

## 📋 MiniDB 当前支持的数据类型

### 数值类型
| 类型 | 描述 | 大小 | 范围 | SQL 示例 |
|------|------|------|------|----------|
| **INTEGER** | 32位有符号整数 | 4字节 | -2,147,483,648 到 2,147,483,647 | `age INTEGER` |
| **BIGINT** | 64位有符号整数 | 8字节 | -9,223,372,036,854,775,808 到 9,223,372,036,854,775,807 | `id BIGINT` |
| **FLOAT** | 32位浮点数 | 4字节 | 约 7 位精度 | `rate FLOAT` |
| **DOUBLE** | 64位双精度浮点数 | 8字节 | 约 15 位精度 | `price DOUBLE` |

### 字符串类型
| 类型 | 描述 | 存储 | SQL 示例 |
|------|------|------|----------|
| **VARCHAR(n)** | 可变长度字符串 | 长度前缀 + 字符串数据 | `name VARCHAR(100)` |

### 逻辑类型
| 类型 | 描述 | 大小 | 值 | SQL 示例 |
|------|------|------|-----|----------|
| **BOOLEAN** | 布尔值 | 1字节 | `true` 或 `false` | `is_active BOOLEAN` |

### 日期时间类型
| 类型 | 描述 | 大小 | 格式 | SQL 示例 |
|------|------|------|------|----------|
| **DATE** | 日期 | 4字节 | YYYY-MM-DD | `birthday DATE` |
| **TIMESTAMP** | 日期时间 | 8字节 | YYYY-MM-DD HH:MM:SS | `created_at TIMESTAMP` |

---

## ❌ 不支持的数据类型

以下是标准 SQL 中常见但 MiniDB **目前不支持**的数据类型：

### 数值类型（不支持）
- ❌ `DECIMAL(p,s)` / `NUMERIC(p,s)` - 定点数
- ❌ `SMALLINT` - 16位整数
- ❌ `TINYINT` - 8位整数
- ❌ `REAL` - 单精度浮点数的别名

### 字符串类型（不支持）
- ❌ `CHAR(n)` - 固定长度字符串
- ❌ `TEXT` - 大文本
- ❌ `CLOB` - 字符大对象

### 二进制类型（不支持）
- ❌ `BINARY(n)` - 固定长度二进制
- ❌ `VARBINARY(n)` - 可变长度二进制
- ❌ `BLOB` - 二进制大对象

### 其他类型（不支持）
- ❌ `UUID` - 唯一标识符
- ❌ `JSON` - JSON 数据
- ❌ `XML` - XML 数据
- ❌ `ARRAY` - 数组类型
- ❌ `ENUM` - 枚举类型

---

## 🔧 测试用例修复说明

### 原来的问题代码：
```sql
-- ❌ 这会导致错误
CREATE TABLE orders (
    order_id INT PRIMARY KEY,
    user_id INT,
    product_name VARCHAR(100),
    price DECIMAL(10,2)  -- 不支持的类型！
);
```

### 修复后的代码：
```sql
-- ✅ 正确的写法
CREATE TABLE orders (
    order_id INT PRIMARY KEY,
    user_id INT,
    product_name VARCHAR(100),
    price DOUBLE  -- 使用 DOUBLE 替代 DECIMAL
);
```

---

## 💡 数据类型使用建议

### 1. 金钱/价格数据
```sql
-- 推荐使用 DOUBLE 存储价格（以分为单位可以提高精度）
price DOUBLE  -- 存储 99.99 这样的价格

-- 或者使用 INTEGER 存储分值
price_cents INTEGER  -- 存储 9999 代表 99.99 元
```

### 2. 标识符
```sql
-- 小范围ID使用 INTEGER
user_id INTEGER

-- 大范围ID使用 BIGINT
transaction_id BIGINT
```

### 3. 文本数据
```sql
-- 短文本
name VARCHAR(100)
email VARCHAR(255)

-- 长文本（目前只能用较大的 VARCHAR）
description VARCHAR(2000)
```

### 4. 状态标记
```sql
-- 使用 BOOLEAN
is_active BOOLEAN
is_deleted BOOLEAN

-- 或使用 INTEGER 表示状态码
status INTEGER  -- 0=inactive, 1=active, 2=suspended
```

---

## 🧪 正确的测试用例示例

### 创建表
```sql
-- 用户表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255),
    age INTEGER,
    salary DOUBLE,
    is_active BOOLEAN,
    created_at TIMESTAMP
);

-- 订单表
CREATE TABLE orders (
    order_id INTEGER PRIMARY KEY,
    user_id INTEGER,
    product_name VARCHAR(100),
    price DOUBLE,
    quantity INTEGER,
    order_date DATE
);
```

### 插入数据
```sql
-- 插入用户
INSERT INTO users VALUES (
    1, 
    'Alice Zhang', 
    'alice@company.com', 
    25, 
    75000.50, 
    true, 
    '2024-01-15 09:00:00'
);

-- 插入订单
INSERT INTO orders VALUES (
    101,
    1,
    'Laptop Pro',
    2999.99,
    1,
    '2024-01-15'
);
```

---

## 🔍 类型转换和兼容性

MiniDB 支持以下类型之间的自动转换：

### 数值类型转换
```sql
INTEGER → BIGINT     ✅ 自动提升
INTEGER → FLOAT      ✅ 自动提升  
INTEGER → DOUBLE     ✅ 自动提升
FLOAT → DOUBLE       ✅ 自动提升
BIGINT ↔ INTEGER     ✅ 双向兼容（范围内）
```

### 字符串类型转换
```sql
VARCHAR(小) → VARCHAR(大)  ✅ 自动提升
VARCHAR → INTEGER         ⚠️  需要解析（如 "123" → 123）
```

---

## 🚀 现在可以正常测试了！

所有测试文档已经修复，你现在可以：

1. **启动 MiniDB**：
```bash
cargo run --bin minidb
```

2. **使用修复后的测试用例**：
```sql
CREATE TABLE users (id INTEGER PRIMARY KEY, name VARCHAR(100), age INTEGER);
CREATE TABLE orders (order_id INTEGER PRIMARY KEY, price DOUBLE);
INSERT INTO orders VALUES (101, 2999.99);
SELECT * FROM orders;
```

3. **查看支持的数据类型**：
- 使用 `INTEGER` 代替 `DECIMAL` 存储整数
- 使用 `DOUBLE` 代替 `DECIMAL` 存储小数
- 使用 `VARCHAR(n)` 存储文本

---

*数据类型说明版本: v1.0*  
*创建日期: 2025年9月16日*  
*适用于: MiniDB v0.1.0*