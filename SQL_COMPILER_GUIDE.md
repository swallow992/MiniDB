# MiniDB SQL编译器 - 使用指南

## 编译和运行

### 编译SQL编译器
```powershell
# 在项目根目录运行
cargo build --bin sql_compiler_demo
```

### 运行方式

#### 1. 自动测试模式（推荐）
程序启动后会自动运行内置的5个测试用例，然后根据输入方式选择模式：

```powershell
# 直接运行，查看自动测试 + 进入交互模式
.\target\debug\sql_compiler_demo.exe
```

#### 2. 批量测试模式（使用测试文件）
```powershell
# 使用标准测试文件
Get-Content sql_compiler_tests.sql | .\target\debug\sql_compiler_demo.exe

# 或者使用简单的测试
echo "CREATE TABLE test (id INT)" | .\target\debug\sql_compiler_demo.exe
```

#### 3. 交互模式
直接运行程序，完成自动测试后会进入交互模式：
```powershell
.\target\debug\sql_compiler_demo.exe
# 然后输入SQL语句，输入 'quit' 或 'exit' 退出
```

## 测试文件说明

### `sql_compiler_tests.sql` - 标准测试套件
- 包含10个全面的测试用例
- 验证CREATE TABLE、INSERT、SELECT的完整流程
- 测试表状态在多个操作间的持续性
- 验证条件查询和复杂数据类型

### 支持的SQL语法

#### 数据类型
- `INT` - 整数类型
- `VARCHAR(n)` - 变长字符串，n为最大长度
- `DECIMAL(p,s)` - 定点数，p为精度，s为小数位数

#### CREATE TABLE语法
```sql
CREATE TABLE table_name (
    column1 datatype,
    column2 datatype,
    ...
)
```

#### INSERT语法
```sql
-- 指定列插入
INSERT INTO table_name (col1, col2, ...) VALUES (val1, val2, ...)

-- 全列插入
INSERT INTO table_name VALUES (val1, val2, ...)
```

#### SELECT语法
```sql
-- 查询所有列
SELECT * FROM table_name

-- 查询指定列
SELECT col1, col2 FROM table_name

-- 条件查询
SELECT col1, col2 FROM table_name WHERE condition
```

#### 支持的WHERE条件
- `=` 等于
- `>` 大于
- `<` 小于
- 更多操作符可以扩展

## 输出说明

每个SQL语句的编译过程会显示：

1. **四元式词法分析**：显示token序列，包含类型、值、位置信息
2. **抽象语法树**：显示解析后的AST结构
3. **执行计划**：显示查询优化后的执行步骤和成本估算

## 关键特性

### ✅ 已实现
- 完整的词法分析（支持关键字、标识符、数字、字符串、操作符）
- 语法分析和AST生成
- 语义分析（表存在性检查、列验证）
- 查询计划生成
- **会话状态管理**：表定义在多个SQL语句间持续有效
- 交互式和批量处理模式
- 详细的错误报告

### 🔄 实现亮点
- **SQLCompilerSession**：解决了语义分析器状态管理的核心问题
- **智能输入检测**：自动识别交互式终端vs管道输入
- **健壮的字符处理**：支持UTF-8并处理不可见字符
- **学术级输出**：四元式词法分析符合编译原理标准

## 故障排除

### 常见问题
1. **编译错误**：确保在项目根目录运行 `cargo build`
2. **权限错误**：在Windows上可能需要管理员权限
3. **字符编码问题**：确保SQL文件使用UTF-8编码

### 调试信息
程序会提供详细的错误信息，包括：
- 错误类型（词法/语法/语义）
- 精确的行列位置
- 具体的错误描述