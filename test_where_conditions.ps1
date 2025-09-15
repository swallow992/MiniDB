#!/usr/bin/env pwsh

# Test WHERE conditions for SELECT statements

Write-Host "🚀 测试 WHERE 条件功能" -ForegroundColor Cyan
Write-Host "=" * 50

# Start the interactive shell in background
$process = Start-Process -FilePath ".\target\debug\minidb.exe" -PassThru -WindowStyle Hidden -RedirectStandardInput -RedirectStandardOutput -RedirectStandardError

# Wait a moment for startup
Start-Sleep -Milliseconds 500

# Function to send command and read output
function Send-Command {
    param([string]$command)
    
    $process.StandardInput.WriteLine($command)
    $process.StandardInput.Flush()
    Start-Sleep -Milliseconds 200
    
    $output = ""
    while ($process.StandardOutput.Peek() -ne -1) {
        $output += $process.StandardOutput.ReadLine() + "`n"
    }
    return $output
}

try {
    Write-Host "📋 1. 创建测试表和数据..." -ForegroundColor Yellow
    
    # Create table
    $null = Send-Command "CREATE TABLE employees (id INTEGER, name VARCHAR(50), age INTEGER, salary FLOAT);"
    
    # Insert test data
    $null = Send-Command "INSERT INTO employees VALUES (1, 'Alice', 25, 50000.0);"
    $null = Send-Command "INSERT INTO employees VALUES (2, 'Bob', 30, 60000.0);"
    $null = Send-Command "INSERT INTO employees VALUES (3, 'Charlie', 35, 70000.0);"
    $null = Send-Command "INSERT INTO employees VALUES (4, 'Diana', 28, 55000.0);"
    
    Write-Host "✅ 测试数据插入完成" -ForegroundColor Green
    
    Write-Host "`n🔍 2. 测试 WHERE 条件查询..." -ForegroundColor Yellow
    
    # Test WHERE with equals
    Write-Host "`n📝 测试: SELECT * FROM employees WHERE age = 30;" -ForegroundColor Cyan
    $output1 = Send-Command "SELECT * FROM employees WHERE age = 30;"
    Write-Host $output1
    
    # Test WHERE with greater than
    Write-Host "`n📝 测试: SELECT * FROM employees WHERE age > 28;" -ForegroundColor Cyan
    $output2 = Send-Command "SELECT * FROM employees WHERE age > 28;"
    Write-Host $output2
    
    # Test WHERE with less than
    Write-Host "`n📝 测试: SELECT * FROM employees WHERE salary < 60000;" -ForegroundColor Cyan
    $output3 = Send-Command "SELECT * FROM employees WHERE salary < 60000;"
    Write-Host $output3
    
    # Test WHERE with string comparison
    Write-Host "`n📝 测试: SELECT * FROM employees WHERE name = 'Alice';" -ForegroundColor Cyan
    $output4 = Send-Command "SELECT * FROM employees WHERE name = 'Alice';"
    Write-Host $output4
    
    # Test SELECT without WHERE (should return all rows)
    Write-Host "`n📝 对比: SELECT * FROM employees;" -ForegroundColor Cyan
    $output5 = Send-Command "SELECT * FROM employees;"
    Write-Host $output5
    
    Write-Host "`n🎯 测试总结:" -ForegroundColor Magenta
    Write-Host "- WHERE 等值查询: $(if ($output1 -match 'Bob') { '✅ 成功' } else { '❌ 失败' })"
    Write-Host "- WHERE 大于查询: $(if ($output2 -match 'Charlie') { '✅ 成功' } else { '❌ 失败' })"
    Write-Host "- WHERE 小于查询: $(if ($output3 -match 'Alice') { '✅ 成功' } else { '❌ 失败' })"
    Write-Host "- WHERE 字符串查询: $(if ($output4 -match 'Alice') { '✅ 成功' } else { '❌ 失败' })"
    
} catch {
    Write-Host "❌ 测试过程中出现错误: $($_.Exception.Message)" -ForegroundColor Red
} finally {
    # Clean up
    $null = Send-Command "\q"
    if (!$process.HasExited) {
        $process.Kill()
    }
    $process.Dispose()
}

Write-Host "`n🏁 WHERE 条件测试完成!" -ForegroundColor Green