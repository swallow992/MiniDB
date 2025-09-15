# 快速综合测试脚本
# 执行核心功能验证，关注主要结果

$ErrorActionPreference = "Continue"

function Write-TestStep($step, $description) {
    Write-Host "[$step] " -ForegroundColor Green -NoNewline
    Write-Host $description -ForegroundColor White
}

function Write-TestResult($result, $details = "") {
    if ($result -eq "PASS") {
        Write-Host "✅ PASS" -ForegroundColor Green
    } elseif ($result -eq "FAIL") {
        Write-Host "❌ FAIL" -ForegroundColor Red
    } else {
        Write-Host "⚠️  WARN" -ForegroundColor Yellow
    }
    if ($details) {
        Write-Host "   $details" -ForegroundColor Gray
    }
}

Write-Host "🚀 MiniDB 快速验证测试" -ForegroundColor Cyan

# 测试1: 基础CRUD
Write-TestStep "1" "基础CRUD操作测试"
$testSQL1 = @"
CREATE TABLE test_table (id INT, name TEXT, value INT);
INSERT INTO test_table VALUES (1, 'Test1', 100);
INSERT INTO test_table VALUES (2, 'Test2', 200);
SELECT * FROM test_table;
SELECT name FROM test_table WHERE value > 150;
UPDATE test_table SET value = 250 WHERE id = 2;
SELECT * FROM test_table WHERE id = 2;
DELETE FROM test_table WHERE id = 1;
SELECT COUNT(*) FROM test_table;
"@

$startTime = Get-Date
$output1 = echo $testSQL1 | cargo run 2>&1
$endTime = Get-Date
$duration1 = ($endTime - $startTime).TotalSeconds

if ($output1 -match "✅" -and $output1 -match "Test" -and $output1 -notmatch "Error") {
    Write-TestResult "PASS" "CRUD操作成功 (${duration1}s)"
} else {
    Write-TestResult "FAIL" "CRUD操作失败"
    Write-Host $output1 -ForegroundColor Red
}

# 测试2: WHERE条件
Write-TestStep "2" "WHERE条件查询测试"
$testSQL2 = @"
CREATE TABLE products (id INT, name TEXT, price INT, category TEXT);
INSERT INTO products VALUES (1, 'Laptop', 1000, 'Electronics');
INSERT INTO products VALUES (2, 'Chair', 200, 'Furniture');
INSERT INTO products VALUES (3, 'Phone', 800, 'Electronics');
SELECT * FROM products WHERE price > 500;
SELECT name FROM products WHERE category = 'Electronics';
SELECT * FROM products WHERE price < 300 AND category = 'Furniture';
"@

$startTime = Get-Date
$output2 = echo $testSQL2 | cargo run 2>&1
$endTime = Get-Date
$duration2 = ($endTime - $startTime).TotalSeconds

if ($output2 -match "✅" -and $output2 -match "Laptop|Electronics" -and $output2 -notmatch "Error") {
    Write-TestResult "PASS" "WHERE查询成功 (${duration2}s)"
} else {
    Write-TestResult "FAIL" "WHERE查询失败"
}

# 测试3: 持久性
Write-TestStep "3" "数据持久性测试"
$testSQL3 = @"
CREATE TABLE persist_test (id INT, data TEXT);
INSERT INTO persist_test VALUES (1, 'Persistent_Data_1');
INSERT INTO persist_test VALUES (2, 'Persistent_Data_2');
SELECT * FROM persist_test;
"@

$startTime = Get-Date
$output3 = echo $testSQL3 | cargo run 2>&1
$endTime = Get-Date
$duration3 = ($endTime - $startTime).TotalSeconds

# 等待文件写入
Start-Sleep -Seconds 1

# 第二次会话读取
$readSQL = "SELECT * FROM persist_test;"
$startTime = Get-Date
$output4 = echo $readSQL | cargo run 2>&1
$endTime = Get-Date
$duration4 = ($endTime - $startTime).TotalSeconds

if ($output3 -match "Persistent_Data" -and $output4 -match "Persistent_Data") {
    Write-TestResult "PASS" "数据持久性验证成功 (${duration3}s + ${duration4}s)"
} else {
    Write-TestResult "FAIL" "数据持久性验证失败"
}

# 测试4: 存储分析
Write-TestStep "4" "存储文件检查"
if (Test-Path "minidb_data") {
    $files = Get-ChildItem "minidb_data" -Recurse
    $fileCount = $files.Count
    $totalSize = ($files | Where-Object { -not $_.PSIsContainer } | Measure-Object Length -Sum).Sum
    Write-TestResult "PASS" "发现 $fileCount 个文件，总大小 $totalSize 字节"
} else {
    Write-TestResult "WARN" "未发现数据目录"
}

# 总结
$totalTime = $duration1 + $duration2 + $duration3 + $duration4
Write-Host "`n🎯 测试总结:" -ForegroundColor Yellow
Write-Host "   总耗时: ${totalTime} 秒" -ForegroundColor Gray
Write-Host "   平均响应: $([math]::Round($totalTime/4, 2)) 秒/测试" -ForegroundColor Gray

if ($totalTime -lt 20) {
    Write-Host "✅ 性能表现良好" -ForegroundColor Green
} else {
    Write-Host "⚠️  性能可能需要优化" -ForegroundColor Yellow
}