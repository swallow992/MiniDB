# 测试数据生成器
# 为 MiniDB 生成各种测试数据和测试场景

param(
    [ValidateSet("basic", "performance", "stress", "edge-cases", "all")]
    [string]$TestType = "basic",
    [int]$RecordCount = 100,
    [string]$OutputFile = "generated_test.sql"
)

function Generate-BasicTestData() {
    @"
-- 基础功能测试数据
CREATE TABLE users (
    id INT,
    username TEXT,
    email TEXT,
    age INT,
    status TEXT
);

CREATE TABLE orders (
    order_id INT,
    user_id INT,
    product TEXT,
    amount INT,
    order_date TEXT
);

-- 用户数据
INSERT INTO users VALUES (1, 'alice_smith', 'alice@example.com', 28, 'active');
INSERT INTO users VALUES (2, 'bob_jones', 'bob@example.com', 35, 'active');
INSERT INTO users VALUES (3, 'carol_white', 'carol@example.com', 42, 'inactive');
INSERT INTO users VALUES (4, 'david_brown', 'david@example.com', 29, 'active');
INSERT INTO users VALUES (5, 'eva_davis', 'eva@example.com', 33, 'pending');

-- 订单数据
INSERT INTO orders VALUES (101, 1, 'Laptop', 1299, '2025-01-15');
INSERT INTO orders VALUES (102, 2, 'Mouse', 29, '2025-01-16');
INSERT INTO orders VALUES (103, 1, 'Keyboard', 89, '2025-01-17');
INSERT INTO orders VALUES (104, 3, 'Monitor', 299, '2025-01-18');
INSERT INTO orders VALUES (105, 4, 'Headphones', 159, '2025-01-19');

-- 基础查询测试
SELECT * FROM users;
SELECT * FROM orders;
SELECT username, age FROM users WHERE age > 30;
SELECT * FROM orders WHERE amount > 100;
SELECT COUNT(*) FROM users;
"@
}

function Generate-PerformanceTestData($recordCount) {
    $sql = @"
-- 性能测试数据 ($recordCount 条记录)
CREATE TABLE performance_test (
    id INT,
    name TEXT,
    category TEXT,
    value INT,
    timestamp TEXT
);

"@

    # 生成大量插入语句
    for ($i = 1; $i -le $recordCount; $i++) {
        $categories = @('A', 'B', 'C', 'D', 'E')
        $category = $categories[(Get-Random -Maximum $categories.Length)]
        $value = Get-Random -Maximum 10000
        $name = "Item_$($i.ToString().PadLeft(6, '0'))"
        $timestamp = "2025-01-$(($i % 28 + 1).ToString().PadLeft(2, '0')) 10:00:00"
        
        $sql += "INSERT INTO performance_test VALUES ($i, '$name', '$category', $value, '$timestamp');`n"
        
        # 每100条记录添加一次查询以测试性能
        if ($i % 100 -eq 0) {
            $sql += "-- Progress: $i/$recordCount records`n"
        }
    }
    
    $sql += @"

-- 性能测试查询
SELECT COUNT(*) FROM performance_test;
SELECT * FROM performance_test WHERE id < 10;
SELECT category, COUNT(*) FROM performance_test WHERE value > 5000;
SELECT * FROM performance_test WHERE name LIKE 'Item_000001%';
"@

    return $sql
}

function Generate-StressTestData() {
    @"
-- 压力测试数据
CREATE TABLE stress_test (
    id INT,
    data TEXT,
    number INT
);

-- 大量连续插入
"@ + (1..1000 | ForEach-Object {
    $data = "StressTest_" + (1..20 | ForEach-Object { Get-Random -Maximum 10 }) -join ""
    "INSERT INTO stress_test VALUES ($_, '$data', $(Get-Random -Maximum 100000));"
} | Out-String) + @"

-- 混合操作
SELECT COUNT(*) FROM stress_test;
UPDATE stress_test SET number = number + 1 WHERE id < 100;
DELETE FROM stress_test WHERE number > 90000;
SELECT COUNT(*) FROM stress_test;
"@
}

function Generate-EdgeCaseTestData() {
    @"
-- 边界条件测试数据
CREATE TABLE edge_cases (
    id INT,
    text_field TEXT,
    number_field INT
);

-- 空值和特殊字符测试
INSERT INTO edge_cases VALUES (1, '', 0);
INSERT INTO edge_cases VALUES (2, 'Normal text', 42);
INSERT INTO edge_cases VALUES (3, 'Text with spaces and symbols !@#$%', -1);
INSERT INTO edge_cases VALUES (4, 'Very long text that might test the limits of text storage in the database system', 2147483647);
INSERT INTO edge_cases VALUES (5, 'Text with "quotes" and ''apostrophes''', -2147483648);

-- 边界值查询
SELECT * FROM edge_cases WHERE number_field = 0;
SELECT * FROM edge_cases WHERE text_field = '';
SELECT * FROM edge_cases WHERE number_field > 1000000;
SELECT * FROM edge_cases WHERE text_field LIKE '%symbols%';

-- 创建重复表名测试（应该失败）
CREATE TABLE edge_cases (id INT, duplicate TEXT);

-- 查询不存在的表（应该失败）
SELECT * FROM non_existent_table;

-- 插入到不存在的表（应该失败）
INSERT INTO missing_table VALUES (1, 'test');
"@
}

function Generate-ComplexQueryTestData() {
    @"
-- 复杂查询测试数据
CREATE TABLE employees (
    emp_id INT,
    name TEXT,
    department TEXT,
    salary INT,
    hire_date TEXT,
    manager_id INT
);

CREATE TABLE departments (
    dept_id INT,
    dept_name TEXT,
    location TEXT,
    budget INT
);

-- 员工数据
INSERT INTO employees VALUES (1, 'John Manager', 'Engineering', 90000, '2020-01-15', 0);
INSERT INTO employees VALUES (2, 'Alice Developer', 'Engineering', 75000, '2021-03-20', 1);
INSERT INTO employees VALUES (3, 'Bob Developer', 'Engineering', 72000, '2021-06-10', 1);
INSERT INTO employees VALUES (4, 'Carol Lead', 'Marketing', 85000, '2020-05-30', 0);
INSERT INTO employees VALUES (5, 'David Analyst', 'Marketing', 65000, '2022-01-15', 4);
INSERT INTO employees VALUES (6, 'Eva Designer', 'Design', 70000, '2021-09-01', 0);
INSERT INTO employees VALUES (7, 'Frank Junior', 'Engineering', 60000, '2023-01-10', 2);

-- 部门数据
INSERT INTO departments VALUES (1, 'Engineering', 'Building A', 500000);
INSERT INTO departments VALUES (2, 'Marketing', 'Building B', 300000);
INSERT INTO departments VALUES (3, 'Design', 'Building A', 200000);
INSERT INTO departments VALUES (4, 'HR', 'Building C', 150000);

-- 复杂查询测试
SELECT * FROM employees WHERE salary > 70000;
SELECT name, salary FROM employees WHERE department = 'Engineering' AND salary < 80000;
SELECT department, COUNT(*) FROM employees;
SELECT * FROM employees WHERE hire_date LIKE '2021%';
SELECT * FROM employees WHERE manager_id > 0;

-- 子查询模拟（通过多步查询）
SELECT * FROM employees WHERE salary > 75000;
SELECT * FROM departments WHERE budget > 250000;

-- 聚合查询模拟
SELECT COUNT(*) FROM employees WHERE department = 'Engineering';
SELECT COUNT(*) FROM employees WHERE salary > 70000;
"@
}

# 根据测试类型生成数据
Write-Host "🎯 生成测试数据..." -ForegroundColor Cyan
Write-Host "测试类型: $TestType" -ForegroundColor Gray
Write-Host "记录数量: $RecordCount" -ForegroundColor Gray
Write-Host "输出文件: $OutputFile" -ForegroundColor Gray

$testData = switch ($TestType) {
    "basic" { 
        Generate-BasicTestData
    }
    "performance" { 
        Generate-PerformanceTestData $RecordCount
    }
    "stress" { 
        Generate-StressTestData
    }
    "edge-cases" { 
        Generate-EdgeCaseTestData
    }
    "all" {
        @"
-- 综合测试数据集
-- 生成时间: $(Get-Date)

$((Generate-BasicTestData))

$((Generate-PerformanceTestData ([Math]::Min($RecordCount, 50))))

$((Generate-EdgeCaseTestData))

$((Generate-ComplexQueryTestData))
"@
    }
}

# 添加测试头部信息
$header = @"
-- MiniDB 测试数据
-- 测试类型: $TestType
-- 生成时间: $(Get-Date)
-- 记录数量: $RecordCount
-- ========================================

"@

$fullTestData = $header + $testData

# 保存到文件
$fullTestData | Out-File -FilePath $OutputFile -Encoding UTF8

Write-Host "✅ 测试数据已生成: $OutputFile" -ForegroundColor Green
Write-Host "📊 文件大小: $((Get-Item $OutputFile).Length) 字节" -ForegroundColor Gray

# 显示生成的SQL统计
$lines = $fullTestData.Split("`n")
$createTables = ($lines | Where-Object { $_ -match "CREATE TABLE" }).Count
$inserts = ($lines | Where-Object { $_ -match "INSERT INTO" }).Count
$selects = ($lines | Where-Object { $_ -match "SELECT" }).Count

Write-Host "`n📈 SQL 语句统计:" -ForegroundColor Yellow
Write-Host "  CREATE TABLE: $createTables" -ForegroundColor Gray
Write-Host "  INSERT: $inserts" -ForegroundColor Gray  
Write-Host "  SELECT: $selects" -ForegroundColor Gray

Write-Host "`n💡 使用方法:" -ForegroundColor Cyan
Write-Host "  Get-Content $OutputFile | cargo run" -ForegroundColor Gray