#!/usr/bin/env pwsh

# Test column projection for SELECT statements

Write-Host "🚀 测试 SELECT 列过滤功能" -ForegroundColor Cyan

# Create test input file
$testCommands = @"
CREATE TABLE employees (id INTEGER, name VARCHAR(50), age INTEGER, salary FLOAT);
INSERT INTO employees VALUES (1, 'Alice', 25, 50000.0);
INSERT INTO employees VALUES (2, 'Bob', 30, 60000.0);
INSERT INTO employees VALUES (3, 'Charlie', 35, 70000.0);
INSERT INTO employees VALUES (4, 'Diana', 28, 55000.0);
SELECT * FROM employees;
SELECT name FROM employees;
SELECT name, age FROM employees;
SELECT age, salary FROM employees WHERE age > 28;
SELECT id, name FROM employees WHERE salary < 60000;
\q
"@

$testCommands | Out-File -FilePath "test_column_input.sql" -Encoding utf8

Write-Host "📝 执行列过滤测试..." -ForegroundColor Yellow

# Run the test
$output = Get-Content "test_column_input.sql" | & ".\target\debug\minidb.exe"

Write-Host $output

# Clean up
Remove-Item "test_column_input.sql" -ErrorAction SilentlyContinue

Write-Host "`n🏁 列过滤测试完成!" -ForegroundColor Green