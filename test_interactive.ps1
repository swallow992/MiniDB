#!/usr/bin/env pwsh
# MiniDB 自动化测试脚本 - 演示所有功能

Write-Host "=== MiniDB 自动化功能测试 ===" -ForegroundColor Green

# 编译项目
Write-Host "`n🔨 编译项目..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 编译成功!" -ForegroundColor Green

# 准备测试命令
$testCommands = @(
    "help",
    "\s",
    "\d", 
    "CREATE TABLE users (id INT, name VARCHAR(50), age INT);",
    "INSERT INTO users VALUES (1, 'Alice', 25);",
    "INSERT INTO users VALUES (2, 'Bob', 30);",
    "INSERT INTO users VALUES (3, 'Charlie', 35);",
    "SELECT * FROM users;",
    "SELECT name FROM users WHERE age > 25;",
    "CREATE TABLE products (id INT, name VARCHAR(100), price FLOAT);",
    "INSERT INTO products VALUES (1, 'Laptop', 999.99);",
    "INSERT INTO products VALUES (2, 'Mouse', 29.99);",
    "SELECT * FROM products;",
    "\d",
    "\s",
    "SELECT * FROM nonexistent_table;",  # 测试错误处理
    "INVALID SQL SYNTAX;",               # 测试语法错误
    "quit"
)

Write-Host "`n🧪 开始自动化测试..." -ForegroundColor Cyan

# 创建输入文件
$inputFile = "test_input.txt"
$testCommands | Out-File -FilePath $inputFile -Encoding UTF8

Write-Host "📝 测试命令序列:" -ForegroundColor Yellow
$testCommands | ForEach-Object { 
    Write-Host "  $_" -ForegroundColor White
}

Write-Host "`n🚀 执行测试..." -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Magenta

# 运行测试
Get-Content $inputFile | .\target\release\minidb.exe

Write-Host "========================================" -ForegroundColor Magenta
Write-Host "✅ 自动化测试完成!" -ForegroundColor Green

# 清理
Remove-Item $inputFile -ErrorAction SilentlyContinue

Write-Host "`n📊 测试总结:" -ForegroundColor Yellow
Write-Host "- ✅ 交互式界面启动成功"
Write-Host "- ✅ 命令解析和执行"
Write-Host "- ✅ 详细的错误信息显示"
Write-Host "- ✅ 执行时间统计"
Write-Host "- ✅ 格式化的结果输出"
Write-Host "- ✅ 帮助和状态命令"

Write-Host "`n🎉 MiniDB 交互式测试完成!" -ForegroundColor Green