#!/usr/bin/env pwsh

# Simple WHERE conditions test - Fixed UTF-8 without BOM

Write-Host "🚀 测试 WHERE 条件功能" -ForegroundColor Cyan

# Create test input file without BOM
$testCommands = @"
CREATE TABLE employees (id INTEGER, name VARCHAR(50), age INTEGER, salary FLOAT);
INSERT INTO employees VALUES (1, 'Alice', 25, 50000.0);
INSERT INTO employees VALUES (2, 'Bob', 30, 60000.0);
INSERT INTO employees VALUES (3, 'Charlie', 35, 70000.0);
INSERT INTO employees VALUES (4, 'Diana', 28, 55000.0);
SELECT * FROM employees;
SELECT * FROM employees WHERE age = 30;
SELECT * FROM employees WHERE age > 28;
SELECT * FROM employees WHERE salary < 60000;
SELECT * FROM employees WHERE name = 'Alice';
\q
"@

# Use UTF8NoBOM encoding to avoid BOM issues
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText("test_input.sql", $testCommands, $utf8NoBom)

Write-Host "📝 执行测试命令..." -ForegroundColor Yellow

# Run the test
$output = Get-Content "test_input.sql" | & ".\target\debug\minidb.exe"

Write-Host $output

# Clean up
Remove-Item "test_input.sql" -ErrorAction SilentlyContinue

Write-Host "`n🏁 测试完成!" -ForegroundColor Green