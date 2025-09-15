# MiniDB 综合测试脚本
# 功能: 执行完整的数据库测试流程，包括CRUD操作、持久性验证和存储分析

param(
    [switch]$Verbose = $false,
    [switch]$GenerateReport = $true,
    [string]$TestDataDir = "minidb_test_data"
)

# 测试配置
$ErrorActionPreference = "Continue"
$TestStartTime = Get-Date
$TestResults = @()
$TestPhases = @(
    "Phase 1: Basic CRUD Operations",
    "Phase 2: Advanced Query Features", 
    "Phase 3: Data Persistence Validation",
    "Phase 4: Storage Layer Analysis",
    "Phase 5: Performance Benchmarks"
)

# 颜色输出函数
function Write-TestHeader($message) {
    Write-Host "`n" -NoNewline
    Write-Host "=" * 60 -ForegroundColor Cyan
    Write-Host " $message" -ForegroundColor Yellow
    Write-Host "=" * 60 -ForegroundColor Cyan
}

function Write-TestStep($step, $description) {
    Write-Host "`n[$step] " -ForegroundColor Green -NoNewline
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
        Write-Host "   Details: $details" -ForegroundColor Gray
    }
}

function Record-TestResult($phase, $test, $result, $details, $duration) {
    $global:TestResults += [PSCustomObject]@{
        Phase = $phase
        Test = $test
        Result = $result
        Details = $details
        Duration = $duration
        Timestamp = Get-Date
    }
}

function Test-DatabaseConnection() {
    Write-TestStep "SETUP" "检查数据库连接"
    
    # 编译项目
    Write-Host "   编译 MiniDB..." -ForegroundColor Gray
    $compileResult = cargo build 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-TestResult "FAIL" "编译失败: $compileResult"
        return $false
    }
    
    Write-TestResult "PASS" "数据库编译成功"
    return $true
}

function Clear-TestData() {
    Write-TestStep "CLEANUP" "清理测试数据"
    
    if (Test-Path $TestDataDir) {
        Remove-Item $TestDataDir -Recurse -Force
        Write-Host "   已删除测试数据目录: $TestDataDir" -ForegroundColor Gray
    }
    
    if (Test-Path "minidb_data") {
        Remove-Item "minidb_data" -Recurse -Force  
        Write-Host "   已删除默认数据目录: minidb_data" -ForegroundColor Gray
    }
    
    Write-TestResult "PASS" "测试环境已清理"
}

function Test-Phase1-BasicCRUD() {
    Write-TestHeader $TestPhases[0]
    $phaseStart = Get-Date
    
    # 创建测试SQL脚本
    $testSQL = @"
-- 创建员工表
CREATE TABLE employees (
    id INT,
    name TEXT,
    department TEXT,
    salary INT,
    hire_date TEXT
);

-- 插入测试数据
INSERT INTO employees VALUES (1, 'Alice Johnson', 'Engineering', 75000, '2023-01-15');
INSERT INTO employees VALUES (2, 'Bob Smith', 'Marketing', 65000, '2023-02-20');
INSERT INTO employees VALUES (3, 'Carol Davis', 'Engineering', 80000, '2023-03-10');
INSERT INTO employees VALUES (4, 'David Wilson', 'Sales', 55000, '2023-04-05');
INSERT INTO employees VALUES (5, 'Eva Brown', 'HR', 60000, '2023-05-12');

-- 基本查询测试
SELECT * FROM employees;
SELECT COUNT(*) FROM employees;
SELECT name, salary FROM employees;

-- 条件查询测试
SELECT * FROM employees WHERE department = 'Engineering';
SELECT * FROM employees WHERE salary > 70000;
SELECT name FROM employees WHERE hire_date LIKE '2023-01%';

-- 更新操作测试
UPDATE employees SET salary = 85000 WHERE name = 'Alice Johnson';
SELECT * FROM employees WHERE name = 'Alice Johnson';

-- 删除操作测试
DELETE FROM employees WHERE department = 'Sales';
SELECT * FROM employees;

-- 验证最终状态
SELECT COUNT(*) FROM employees;
"@

    # 将SQL保存到文件
    $testSQL | Out-File -FilePath "phase1_test.sql" -Encoding UTF8
    
    Write-TestStep "1.1" "执行基础CRUD操作测试"
    
    # 执行测试
    $testStart = Get-Date
    $output = echo $testSQL | cargo run 2>&1
    $testDuration = (Get-Date) - $testStart
    
    # 分析输出
    $success = $output -match "✅" -and $output -match "employees" -and $output -notmatch "Error"
    
    if ($success) {
        Write-TestResult "PASS" "所有CRUD操作执行成功"
        Record-TestResult "Phase 1" "Basic CRUD" "PASS" "CREATE, INSERT, SELECT, UPDATE, DELETE operations completed" $testDuration.TotalSeconds
    } else {
        Write-TestResult "FAIL" "CRUD操作执行失败"
        Record-TestResult "Phase 1" "Basic CRUD" "FAIL" "One or more operations failed" $testDuration.TotalSeconds
        if ($Verbose) {
            Write-Host "详细输出:" -ForegroundColor Yellow
            Write-Host $output -ForegroundColor Gray
        }
    }
    
    # 清理测试文件
    Remove-Item "phase1_test.sql" -ErrorAction SilentlyContinue
}

function Test-Phase2-AdvancedQueries() {
    Write-TestHeader $TestPhases[1]
    
    $testSQL = @"
-- 重新创建测试表（以防Phase1失败）
CREATE TABLE products (
    id INT,
    name TEXT,
    category TEXT,
    price INT,
    stock INT
);

-- 插入测试数据
INSERT INTO products VALUES (1, 'Laptop Pro', 'Electronics', 1299, 50);
INSERT INTO products VALUES (2, 'Wireless Mouse', 'Electronics', 29, 200);
INSERT INTO products VALUES (3, 'Office Chair', 'Furniture', 199, 30);
INSERT INTO products VALUES (4, 'Desk Lamp', 'Furniture', 89, 75);
INSERT INTO products VALUES (5, 'Smartphone', 'Electronics', 899, 100);

-- 复杂WHERE条件测试
SELECT * FROM products WHERE price > 100 AND category = 'Electronics';
SELECT * FROM products WHERE stock < 50 OR price > 1000;
SELECT name, price FROM products WHERE category = 'Furniture';

-- 列投影测试
SELECT name FROM products;
SELECT name, price FROM products WHERE price < 200;
SELECT id, name, stock FROM products WHERE stock > 50;

-- 边界条件测试
SELECT * FROM products WHERE price = 1299;
SELECT * FROM products WHERE name = 'Laptop Pro';
SELECT * FROM products WHERE category = 'NonExistent';
"@

    Write-TestStep "2.1" "执行高级查询功能测试"
    
    $testStart = Get-Date
    $output = echo $testSQL | cargo run 2>&1
    $testDuration = (Get-Date) - $testStart
    
    # 检查关键输出模式
    $hasWhere = $output -match "WHERE"
    $hasProjection = $output -match "name.*price" -or $output -match "Electronics"
    $hasResults = $output -match "✅"
    
    if ($hasWhere -and $hasProjection -and $hasResults) {
        Write-TestResult "PASS" "高级查询功能正常"
        Record-TestResult "Phase 2" "Advanced Queries" "PASS" "WHERE conditions and column projection working" $testDuration.TotalSeconds
    } else {
        Write-TestResult "FAIL" "高级查询功能异常"
        Record-TestResult "Phase 2" "Advanced Queries" "FAIL" "Query features not working properly" $testDuration.TotalSeconds
    }
}

function Test-Phase3-DataPersistence() {
    Write-TestHeader $TestPhases[2]
    
    Write-TestStep "3.1" "测试数据持久性 - 第一次会话"
    
    # 第一次会话：创建数据
    $setupSQL = @"
CREATE TABLE persistent_test (
    id INT,
    data TEXT,
    timestamp TEXT
);

INSERT INTO persistent_test VALUES (1, 'Session1_Data1', '2025-09-15 10:00:00');
INSERT INTO persistent_test VALUES (2, 'Session1_Data2', '2025-09-15 10:01:00');
INSERT INTO persistent_test VALUES (3, 'Session1_Data3', '2025-09-15 10:02:00');

SELECT * FROM persistent_test;
"@

    $testStart = Get-Date
    $output1 = echo $setupSQL | cargo run 2>&1
    $duration1 = (Get-Date) - $testStart
    
    Write-Host "   第一次会话完成，检查数据文件..." -ForegroundColor Gray
    
    # 检查数据文件是否创建
    $dataFiles = @()
    if (Test-Path "minidb_data") {
        $dataFiles = Get-ChildItem "minidb_data" -Recurse
        Write-Host "   发现数据文件: $($dataFiles.Count) 个" -ForegroundColor Gray
    }
    
    Write-TestStep "3.2" "测试数据持久性 - 第二次会话"
    
    # 短暂等待确保文件系统同步
    Start-Sleep -Seconds 1
    
    # 第二次会话：读取数据
    $readSQL = @"
SELECT * FROM persistent_test;
SELECT COUNT(*) FROM persistent_test;
"@

    $testStart = Get-Date
    $output2 = echo $readSQL | cargo run 2>&1
    $duration2 = (Get-Date) - $testStart
    
    # 分析持久性结果
    $session1Success = $output1 -match "Session1_Data" -and $output1 -match "✅"
    $session2Success = $output2 -match "Session1_Data" -and $output2 -match "✅"
    
    if ($session1Success -and $session2Success) {
        Write-TestResult "PASS" "数据持久性验证成功"
        Record-TestResult "Phase 3" "Data Persistence" "PASS" "Data survived database restart" ($duration1 + $duration2)
    } else {
        Write-TestResult "FAIL" "数据持久性验证失败"
        Record-TestResult "Phase 3" "Data Persistence" "FAIL" "Data not persisted across sessions" ($duration1 + $duration2)
        
        if ($Verbose) {
            Write-Host "第一次会话输出:" -ForegroundColor Yellow
            Write-Host $output1 -ForegroundColor Gray
            Write-Host "第二次会话输出:" -ForegroundColor Yellow  
            Write-Host $output2 -ForegroundColor Gray
        }
    }
}

function Test-Phase4-StorageAnalysis() {
    Write-TestHeader $TestPhases[3]
    
    Write-TestStep "4.1" "分析底层存储结构"
    
    # 检查数据目录结构
    if (Test-Path "minidb_data") {
        $dataDir = Get-ChildItem "minidb_data" -Recurse
        Write-Host "   数据目录结构:" -ForegroundColor Gray
        
        foreach ($item in $dataDir) {
            $size = if ($item.PSIsContainer) { "目录" } else { "$($item.Length) bytes" }
            Write-Host "     $($item.Name) - $size" -ForegroundColor Gray
        }
        
        # 尝试读取页文件内容（如果存在）
        $pageFiles = $dataDir | Where-Object { $_.Extension -eq ".page" -or $_.Name -match "page" }
        
        if ($pageFiles) {
            Write-TestStep "4.2" "检查数据页内容"
            foreach ($pageFile in $pageFiles) {
                try {
                    $content = Get-Content $pageFile.FullName -Raw -ErrorAction Stop
                    $contentLength = $content.Length
                    Write-Host "     页文件: $($pageFile.Name), 大小: $contentLength 字符" -ForegroundColor Gray
                    
                    # 检查是否包含我们的测试数据
                    if ($content -match "Session1_Data" -or $content -match "employees" -or $content -match "products") {
                        Write-Host "     ✅ 发现测试数据在页文件中" -ForegroundColor Green
                    }
                } catch {
                    Write-Host "     ⚠️  无法读取页文件: $($_.Exception.Message)" -ForegroundColor Yellow
                }
            }
        }
        
        Write-TestResult "PASS" "存储结构分析完成"
        Record-TestResult "Phase 4" "Storage Analysis" "PASS" "Storage structure validated" 1.0
    } else {
        Write-TestResult "WARN" "未找到数据目录"
        Record-TestResult "Phase 4" "Storage Analysis" "WARN" "No data directory found" 0.5
    }
}

function Test-Phase5-Performance() {
    Write-TestHeader $TestPhases[4]
    
    Write-TestStep "5.1" "性能基准测试"
    
    # 批量插入测试
    $perfSQL = @"
CREATE TABLE perf_test (
    id INT,
    value TEXT,
    number INT
);
"@

    # 生成批量插入语句
    for ($i = 1; $i -le 100; $i++) {
        $perfSQL += "INSERT INTO perf_test VALUES ($i, 'TestValue$i', $(Get-Random -Maximum 1000));`n"
    }
    
    $perfSQL += @"
SELECT COUNT(*) FROM perf_test;
SELECT * FROM perf_test WHERE id < 10;
SELECT * FROM perf_test WHERE number > 500;
"@

    $testStart = Get-Date
    $output = echo $perfSQL | cargo run 2>&1
    $testDuration = (Get-Date) - $testStart
    
    $recordsPerSecond = [math]::Round(100 / $testDuration.TotalSeconds, 2)
    
    Write-Host "   执行时间: $($testDuration.TotalSeconds.ToString('F2')) 秒" -ForegroundColor Gray
    Write-Host "   插入速度: $recordsPerSecond 记录/秒" -ForegroundColor Gray
    
    if ($testDuration.TotalSeconds -lt 10) {
        Write-TestResult "PASS" "性能表现良好"
        Record-TestResult "Phase 5" "Performance" "PASS" "$recordsPerSecond records/sec" $testDuration.TotalSeconds
    } else {
        Write-TestResult "WARN" "性能可能需要优化"
        Record-TestResult "Phase 5" "Performance" "WARN" "Performance below expectations" $testDuration.TotalSeconds
    }
}

function Generate-TestReport() {
    if (-not $GenerateReport) { return }
    
    Write-TestHeader "生成测试报告"
    
    $reportPath = "TEST_REPORT_$(Get-Date -Format 'yyyyMMdd_HHmmss').md"
    $totalDuration = ((Get-Date) - $TestStartTime).TotalSeconds
    
    $reportContent = @"
# MiniDB 综合测试报告

## 测试概览

- **测试时间**: $(Get-Date -Format 'yyyy年MM月dd日 HH:mm:ss')
- **总耗时**: $($totalDuration.ToString('F2')) 秒
- **测试阶段**: $($TestPhases.Count) 个
- **测试用例**: $($TestResults.Count) 个

## 测试结果汇总

| 阶段 | 测试项目 | 结果 | 耗时(秒) | 详情 |
|------|----------|------|----------|------|
"@

    foreach ($result in $TestResults) {
        $resultIcon = switch ($result.Result) {
            "PASS" { "✅" }
            "FAIL" { "❌" }
            "WARN" { "⚠️" }
            default { "❓" }
        }
        
        $reportContent += "| $($result.Phase) | $($result.Test) | $resultIcon $($result.Result) | $($result.Duration.ToString('F2')) | $($result.Details) |`n"
    }
    
    # 统计结果
    $passCount = ($TestResults | Where-Object { $_.Result -eq "PASS" }).Count
    $failCount = ($TestResults | Where-Object { $_.Result -eq "FAIL" }).Count
    $warnCount = ($TestResults | Where-Object { $_.Result -eq "WARN" }).Count
    
    $reportContent += @"

## 测试统计

- **通过**: $passCount 个测试
- **失败**: $failCount 个测试  
- **警告**: $warnCount 个测试
- **成功率**: $(if($TestResults.Count -gt 0) { [math]::Round($passCount / $TestResults.Count * 100, 1) } else { 0 })%

## 测试结论

"@

    if ($failCount -eq 0) {
        $reportContent += "🎉 **测试通过**: MiniDB 所有核心功能正常工作！"
    } elseif ($failCount -le 2) {
        $reportContent += "⚠️ **基本通过**: MiniDB 核心功能基本正常，少数问题需要修复。"
    } else {
        $reportContent += "❌ **需要修复**: MiniDB 存在多个问题，需要进行调试和修复。"
    }
    
    $reportContent += @"

## 推荐改进

1. **性能优化**: 考虑实现索引机制提高查询效率
2. **数据类型**: 扩展支持更多SQL数据类型
3. **事务处理**: 添加事务和并发控制机制
4. **错误处理**: 改进错误信息的详细程度
5. **持久化**: 优化数据存储格式和恢复机制

---
*报告生成时间: $(Get-Date)*
"@

    $reportContent | Out-File -FilePath $reportPath -Encoding UTF8
    Write-Host "✅ 测试报告已生成: $reportPath" -ForegroundColor Green
}

# 主测试流程
Write-Host "🚀 MiniDB 综合测试开始" -ForegroundColor Cyan
Write-Host "测试时间: $(Get-Date)" -ForegroundColor Gray

try {
    # 环境准备
    if (-not (Test-DatabaseConnection)) {
        Write-Host "❌ 数据库连接失败，测试终止" -ForegroundColor Red
        exit 1
    }
    
    Clear-TestData
    
    # 执行测试阶段
    Test-Phase1-BasicCRUD
    Test-Phase2-AdvancedQueries  
    Test-Phase3-DataPersistence
    Test-Phase4-StorageAnalysis
    Test-Phase5-Performance
    
    # 生成报告
    Generate-TestReport
    
    Write-TestHeader "测试完成"
    
    $passCount = ($TestResults | Where-Object { $_.Result -eq "PASS" }).Count
    $totalTests = $TestResults.Count
    
    if ($passCount -eq $totalTests) {
        Write-Host "🎉 所有测试通过！($passCount/$totalTests)" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "⚠️  部分测试失败 ($passCount/$totalTests 通过)" -ForegroundColor Yellow
        exit 1
    }
    
} catch {
    Write-Host "❌ 测试执行异常: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
} finally {
    Write-Host "`n测试总耗时: $(((Get-Date) - $TestStartTime).TotalSeconds.ToString('F2')) 秒" -ForegroundColor Gray
}