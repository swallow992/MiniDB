#!/usr/bin/env pwsh
# 数据持久化测试脚本

Write-Host "=== MiniDB 数据持久化测试 ===" -ForegroundColor Green

# 清理之前的测试数据
Write-Host "清理之前的测试数据..." -ForegroundColor Yellow
if (Test-Path "minidb_data") {
    Remove-Item -Recurse -Force "minidb_data"
}

# 构建项目
Write-Host "构建项目..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "构建失败!" -ForegroundColor Red
    exit 1
}

# 阶段1：创建数据并验证
Write-Host "`n=== 阶段1：创建数据 ===" -ForegroundColor Cyan

$stage1_sql = @"
CREATE TABLE users (id INTEGER, name VARCHAR(50), age INTEGER);
INSERT INTO users VALUES (1, 'Alice', 25), (2, 'Bob', 30), (3, 'Charlie', 35);
SELECT * FROM users;
CREATE TABLE products (id INTEGER, name VARCHAR(100), price DOUBLE);
INSERT INTO products VALUES (1, 'Laptop', 999.99), (2, 'Phone', 599.50);
SELECT * FROM products;
"@

Write-Host "执行阶段1 SQL..." -ForegroundColor Yellow
$stage1_sql | ./target/debug/minidb.exe

# 验证数据文件已创建
Write-Host "`n验证数据文件..." -ForegroundColor Yellow
if (Test-Path "minidb_data") {
    Write-Host "数据目录已创建:" -ForegroundColor Green
    Get-ChildItem "minidb_data" | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
} else {
    Write-Host "错误：数据目录未创建!" -ForegroundColor Red
    exit 1
}

# 阶段2：重新启动并验证数据持久化
Write-Host "`n=== 阶段2：重新启动并验证数据持久化 ===" -ForegroundColor Cyan

$stage2_sql = @"
SELECT * FROM users;
SELECT * FROM products;
INSERT INTO users VALUES (4, 'Diana', 28);
SELECT * FROM users;
"@

Write-Host "重新启动并执行阶段2 SQL..." -ForegroundColor Yellow
$stage2_sql | ./target/debug/minidb.exe

# 阶段3：修改数据并再次重启验证
Write-Host "`n=== 阶段3：修改数据并再次重启验证 ===" -ForegroundColor Cyan

$stage3_sql = @"
UPDATE users SET age = 26 WHERE name = 'Alice';
DELETE FROM users WHERE name = 'Bob';
SELECT * FROM users;
"@

Write-Host "执行阶段3 修改操作..." -ForegroundColor Yellow
$stage3_sql | ./target/debug/minidb.exe

# 阶段4：最终验证
Write-Host "`n=== 阶段4：最终验证 ===" -ForegroundColor Cyan

$stage4_sql = @"
SELECT * FROM users;
SELECT * FROM products;
"@

Write-Host "最终重启验证..." -ForegroundColor Yellow
$stage4_sql | ./target/debug/minidb.exe

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green
Write-Host "如果您看到所有阶段的正确输出，说明数据持久化功能正常工作！" -ForegroundColor Green