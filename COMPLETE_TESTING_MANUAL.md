# MiniDB 三模块完整测试指南

## 📋 测试概述

MiniDB 项目包含三个核心模块，每个模块都有专门的测试方法和验证标准。本指南将详细说明如何正确测试每个模块的功能。

## 🎯 测试环境准备

### 系统要求
- **操作系统**: Windows (支持 PowerShell)
- **Rust版本**: 1.70+
- **依赖工具**: Cargo, Git

### 预备步骤
```powershell
# 1. 克隆项目
git clone https://github.com/swallow992/MiniDB.git
cd MiniDB

# 2. 编译项目
cargo build

# 3. 验证编译成功
cargo build --bin sql_compiler_demo
cargo build --bin storage_demo  
cargo build --bin database_demo
cargo build --bin storage_system_test
```

---

## 🔧 模块一: SQL编译器测试

### 测试目标
验证 SQL 编译器的四个核心功能：
1. **词法分析** - 将SQL文本分解为Token
2. **语法分析** - 构建抽象语法树(AST)
3. **语义分析** - 类型检查和语义验证
4. **执行计划生成** - 生成查询执行计划

### 🚀 快速测试

```powershell
# 运行SQL编译器测试脚本
.\run_sql_compiler.ps1

# 选择测试模式
# 输入: 1 (运行标准测试套件 - 推荐)
```

### 📊 预期测试结果

#### 1. 词法分析验证
```
【四元式词法分析】
  (1: Keyword, 'CREATE', 1:1)
  (2: Keyword, 'TABLE', 1:8)
  (3: Identifier, 'student', 1:14)
  (4: Punctuation, '(', 1:22)
  ...
```

#### 2. 语法分析验证
```
【抽象语法树】
  CreateTable { 
    name: "student", 
    columns: [
      Column { name: "id", data_type: INT, nullable: true },
      Column { name: "name", data_type: VARCHAR(50), nullable: true }
    ] 
  }
```

#### 3. 语义分析验证
```
✅ 编译成功！
- 表名验证通过
- 列定义验证通过  
- 数据类型检查通过
```

#### 4. 执行计划验证
```
【执行计划】
  步骤 1: CreateTable { name: "student", schema: "id: INT, name: VARCHAR(50)" }
  预估成本: 1.00
```

### 🧪 详细测试案例

#### 测试用例1: CREATE TABLE
```sql
CREATE TABLE student (id INT, name VARCHAR(50), score DECIMAL(5,2))
```
**验证点**: 
- ✅ 关键字识别 (CREATE, TABLE)
- ✅ 标识符解析 (student, id, name, score)
- ✅ 数据类型解析 (INT, VARCHAR, DECIMAL)
- ✅ 语法结构正确

#### 测试用例2: INSERT语句
```sql
INSERT INTO student (id, name, score) VALUES (1, 'Alice', 95.5)
```
**验证点**:
- ✅ INSERT语法解析
- ✅ 列名匹配验证
- ✅ 值类型检查
- ✅ 执行计划生成

#### 测试用例3: SELECT查询
```sql
SELECT name, score FROM student WHERE score > 90
```
**验证点**:
- ✅ SELECT子句解析
- ✅ WHERE条件解析
- ✅ 比较操作符识别
- ✅ 查询优化计划

### 🎮 交互式测试

```powershell
# 选择交互模式
.\run_sql_compiler.ps1
# 输入: 2 (交互模式)

# 手动输入SQL进行测试
SQL> CREATE TABLE test (id INT);
SQL> INSERT INTO test VALUES (1);
SQL> SELECT * FROM test;
SQL> quit
```

### ✅ 通过标准
- [x] 所有测试用例编译成功
- [x] 词法分析输出正确的Token序列
- [x] 语法分析生成正确的AST
- [x] 语义分析通过类型检查
- [x] 执行计划生成合理的查询计划

---

## 💾 模块二: 磁盘存储系统设计测试

### 测试目标
验证存储系统的核心功能：
1. **页式存储管理** - 4KB页面分配和释放
2. **LRU缓存机制** - 缓冲池管理和命中率
3. **数据持久化** - 磁盘I/O和数据完整性
4. **性能指标** - 吞吐量和响应时间

### 🚀 快速测试

```powershell
# 运行存储系统测试脚本
.\run_storage_test.ps1

# 选择测试模式  
# 输入: 1 (完整自动化测试套件 - 推荐)
```

### 📊 预期测试结果

#### 1. 页面分配测试
```
开始页面分配测试 - 分配 500 个页面
已分配页面: 100/500
已分配页面: 200/500
已分配页面: 300/500
已分配页面: 400/500
已分配页面: 500/500
页面分配测试完成
```

#### 2. 缓存性能测试
```
开始缓存性能测试 - 2000 次随机访问
缓存访问进度: 1000/2000 (命中率: 33.40%)
缓存访问进度: 2000/2000 (命中率: 33.35%)
缓存性能测试完成
```

#### 3. 数据操作测试
```
开始数据操作测试 - 插入和查询 1000 条记录
插入记录进度: 1000/1000
查询记录进度: 1000/1000
数据操作测试完成
```

#### 4. 性能统计
```
=== 存储系统测试统计 ===
页面分配数: 500
页面释放数: 0
缓存命中: 667
缓存丢失: 1333
缓存命中率: 33.35%
记录插入: 1000
记录读取: 1000
写入字节: 4030 KB
读取字节: 16048 KB
总操作数: 4500
测试耗时: 8.1851ms
```

### 🧪 详细测试模式

#### 模式1: 完整自动化测试 (推荐)
```powershell
.\run_storage_test.ps1
# 输入: 1
```
**测试内容**: 页面管理 + 缓存机制 + 数据操作 + 性能分析

#### 模式2: 页式存储专项测试
```powershell
.\run_storage_test.ps1  
# 输入: 2
```
**测试内容**: 专注于页面分配、释放、扩展机制

#### 模式3: 缓存机制专项测试
```powershell
.\run_storage_test.ps1
# 输入: 3  
```
**测试内容**: LRU算法验证、命中率统计、缓存性能

#### 模式4: 数据操作专项测试
```powershell
.\run_storage_test.ps1
# 输入: 4
```
**测试内容**: 数据插入、查询、更新、删除操作

#### 模式5: 交互式测试
```powershell
.\run_storage_test.ps1
# 输入: 5
```
**测试内容**: 手动控制测试参数和操作

### ✅ 通过标准
- [x] 页面分配成功率 > 99%
- [x] 缓存命中率 > 30%
- [x] 数据读写操作无错误
- [x] 性能指标在合理范围内
- [x] 内存使用稳定，无内存泄漏

---

## 🗄️ 模块三: 数据库系统设计测试

### 测试目标
验证完整数据库系统的集成功能：
1. **SQL操作流程** - 建表→插入→查询→删除→验证
2. **数据持久性** - 程序重启后数据恢复
3. **事务完整性** - 操作日志和一致性保证
4. **系统集成** - 各模块协同工作

### 🚀 快速测试

```powershell
# 运行数据库系统测试
cargo run --bin database_demo

# 选择测试模式
# 输入: 1 (自动演示模式 - 推荐)
```

### 📊 预期测试结果

#### 阶段1: 建表操作
```
=== 阶段1: 建表操作 ===
🗃️ CREATE TABLE: users 创建成功
🗃️ CREATE TABLE: orders 创建成功
🗃️ CREATE TABLE: products 创建成功
✅ 创建了3个表: users, orders, products
```

#### 阶段2: 插入数据
```
=== 阶段2: 插入数据 ===
📝 INSERT: 插入记录到表 users
📝 INSERT: 插入记录到表 users
📝 INSERT: 插入记录到表 orders
📝 INSERT: 插入记录到表 products
✅ 插入完成: users(5条), orders(5条), products(4条)
```

#### 阶段3: 查询操作
```
=== 阶段3: 查询操作 ===

1. 查询所有用户:
📋 表: users
┌─────────────────────────────────────────────────────────┐
│ id           │ name         │ age          │ email        │
├─────────────────────────────────────────────────────────┤
│ 1            │ 'Alice'      │ 25           │ 'alice@email.com' │
│ 2            │ 'Bob'        │ 30           │ 'bob@email.com' │
└─────────────────────────────────────────────────────────┘
记录数: 5
```

#### 阶段4: 高级查询
```
1. 查询年龄大于25的用户:
🔍 SEQSCAN + FILTER: 扫描并过滤表 users
   过滤完成，找到 3 条匹配记录

2. 投影查询 - 用户名和邮箱:
🔍 SEQSCAN + PROJECT: 扫描并投影表 users
   投影完成，返回 5 条记录
```

#### 阶段5: 删除操作
```
=== 阶段5: 删除操作 ===
删除orders表中的所有记录...
🗑️ DELETE: 从表 orders 删除 5 条记录
已删除 5 条记录
```

#### 阶段6: 删除后验证
```
=== 阶段6: 删除后查询验证 ===

1. 查询users表（应该有数据）: ✅ 5条记录
2. 查询orders表（应该为空）: ✅ 0条记录  
3. 查询products表（应该有数据）: ✅ 4条记录
```

#### 阶段7: 系统统计
```
=== 阶段7: 系统统计和日志 ===

数据库表列表:
  📋 users
  📋 products  
  📋 orders

📜 事务日志:
  1: CREATE TABLE users
  2: CREATE TABLE orders
  3: CREATE TABLE products
  4-8: INSERT INTO users (5次)
  9-13: INSERT INTO orders (5次)
  14-17: INSERT INTO products (4次)
  18: DELETE FROM orders
总操作数: 18

✅ 完整生命周期测试完成！
```

### 🧪 详细测试验证

#### 1. SQL操作序列验证
**测试序列**:
```sql
CREATE TABLE → INSERT INTO → SELECT FROM → UPDATE SET → DELETE FROM
```

**验证点**:
- ✅ DDL操作 (CREATE TABLE) 成功
- ✅ DML操作 (INSERT/SELECT/UPDATE/DELETE) 正常
- ✅ 查询结果准确无误
- ✅ 数据修改操作生效

#### 2. 数据持久性验证
**测试流程**:
1. 执行数据操作
2. 模拟程序重启
3. 验证数据恢复

**验证点**:
- ✅ 数据自动保存到磁盘
- ✅ 程序重启后数据完整恢复
- ✅ 查询结果与重启前一致

#### 3. 事务完整性验证
**验证点**:
- ✅ 所有操作记录在事务日志中
- ✅ 操作顺序正确
- ✅ 数据状态一致性保证

#### 4. 系统集成验证
**验证点**:
- ✅ SQL编译器正确解析所有语句
- ✅ 存储系统稳定处理数据操作
- ✅ 查询引擎返回正确结果
- ✅ 各模块协同工作无错误

### 🎮 交互式测试

```powershell
cargo run --bin database_demo
# 输入: 2 (交互模式)

# 手动输入SQL命令
SQL> CREATE TABLE test (id INT, name VARCHAR(50));
SQL> INSERT INTO test VALUES (1, 'Test User');
SQL> SELECT * FROM test;
SQL> DROP TABLE test;
SQL> quit
```

### ✅ 通过标准
- [x] 所有SQL操作执行成功
- [x] 查询结果准确无误
- [x] 数据持久性验证通过
- [x] 事务日志完整记录
- [x] 系统运行稳定无崩溃
- [x] 内存使用合理
- [x] 错误处理机制正常

---

## 🏆 综合测试验证清单

### 必测项目检查表

#### SQL编译器模块 ✅
- [ ] 词法分析: Token正确识别
- [ ] 语法分析: AST结构正确
- [ ] 语义分析: 类型检查通过
- [ ] 执行计划: 计划生成合理
- [ ] 错误处理: 语法错误正确报告

#### 存储系统模块 ✅  
- [ ] 页面管理: 分配/释放正常
- [ ] 缓存机制: LRU算法工作正常
- [ ] 数据持久化: 磁盘读写无误
- [ ] 性能指标: 吞吐量达标
- [ ] 内存管理: 无内存泄漏

#### 数据库系统模块 ✅
- [ ] DDL操作: CREATE/DROP TABLE成功
- [ ] DML操作: INSERT/SELECT/UPDATE/DELETE正常
- [ ] 查询功能: 条件查询/投影查询正确
- [ ] 数据持久性: 重启后数据恢复
- [ ] 事务日志: 操作记录完整
- [ ] 系统集成: 各模块协同正常

### 🎯 性能基准

#### SQL编译器性能
- **解析速度**: < 1ms (简单SQL)
- **内存使用**: < 10MB
- **错误率**: 0% (正确SQL)

#### 存储系统性能
- **页面分配**: > 1000 页/秒
- **缓存命中率**: > 30%
- **I/O吞吐量**: > 10MB/秒

#### 数据库系统性能
- **查询响应**: < 10ms (1000条记录)
- **并发操作**: 支持基本并发
- **数据完整性**: 100%保证

## 🚀 故障排除

### 常见问题解决

#### 1. 编译失败
```powershell
# 解决方案: 更新Rust版本
rustup update
cargo clean
cargo build
```

#### 2. 测试脚本无法运行
```powershell
# 解决方案: 检查PowerShell执行策略
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### 3. 权限错误
```powershell
# 解决方案: 以管理员身份运行PowerShell
# 或者在项目目录下运行测试
```

#### 4. 端口被占用
```powershell
# 解决方案: 检查端口使用情况
netstat -ano | findstr :端口号
# 终止占用进程或更换端口
```

---

## 📞 技术支持

### 联系方式
- **项目仓库**: https://github.com/swallow992/MiniDB
- **问题反馈**: 在GitHub仓库提交Issue
- **文档参考**: 查看项目根目录下的README.md

### 学习资源
- **数据库理论**: 《数据库系统概念》
- **Rust编程**: 《Rust程序设计语言》
- **系统设计**: 《设计数据密集型应用》

---

## 🎓 总结

通过以上三个模块的完整测试，你将验证MiniDB数据库系统的：

1. **SQL编译能力** - 完整的SQL解析和优化流程
2. **存储管理能力** - 高效的页式存储和缓存机制  
3. **系统集成能力** - 各模块协同的完整数据库功能

这套测试框架不仅验证了功能正确性，还提供了性能基准和故障排除指南，是学习数据库系统实现的完整测试方案。

**祝你测试顺利！** 🎉