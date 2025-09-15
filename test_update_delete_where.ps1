#!/usr/bin/env pwsh

# Test UPDATE and DELETE with WHERE conditions

Write-Host "🚀 测试 UPDATE/DELETE WHERE 条件功能" -ForegroundColor Cyan

# Create test input file
$testCommands = @"
CREATE TABLE employees (id INTEGER, name VARCHAR(50), age INTEGER, salary FLOAT);
INSERT INTO employees VALUES (1, 'Alice', 25, 50000.0);
INSERT INTO employees VALUES (2, 'Bob', 30, 60000.0);
INSERT INTO employees VALUES (3, 'Charlie', 35, 70000.0);
INSERT INTO employees VALUES (4, 'Diana', 28, 55000.0);
SELECT * FROM employees;
UPDATE employees SET salary = 75000.0 WHERE age > 30;
SELECT * FROM employees;
UPDATE employees SET age = 26 WHERE name = 'Alice';
SELECT * FROM employees;
DELETE FROM employees WHERE salary > 65000;
SELECT * FROM employees;
DELETE FROM employees WHERE age < 27;
SELECT * FROM employees;
\q
"@

$testCommands | Out-File -FilePath "test_update_delete_input.sql" -Encoding utf8

Write-Host "📝 执行 UPDATE/DELETE WHERE 测试..." -ForegroundColor Yellow

# Run the test
$output = Get-Content "test_update_delete_input.sql" | & ".\target\debug\minidb.exe"

Write-Host $output

# Clean up
Remove-Item "test_update_delete_input.sql" -ErrorAction SilentlyContinue

Write-Host "`n🏁 UPDATE/DELETE WHERE 测试完成!" -ForegroundColor Green