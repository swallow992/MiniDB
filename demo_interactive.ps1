#!/usr/bin/env pwsh
# MiniDB 交互式测试演示脚本

Write-Host "=== MiniDB 交互式测试演示 ===" -ForegroundColor Green

Write-Host "`n🚀 正在启动 MiniDB..." -ForegroundColor Yellow

# 编译项目
Write-Host "编译项目..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 编译成功!" -ForegroundColor Green

Write-Host "`n📋 可用的交互式命令:" -ForegroundColor Yellow
Write-Host "  help 或 \h     - 显示帮助信息"
Write-Host "  \d            - 列出所有表"
Write-Host "  \s            - 显示系统状态"
Write-Host "  clear 或 \c   - 清空屏幕"
Write-Host "  quit 或 exit  - 退出程序"

Write-Host "`n📝 SQL 命令示例:" -ForegroundColor Yellow
Write-Host "  CREATE TABLE users (id INT, name VARCHAR(50), age INT);"
Write-Host "  INSERT INTO users VALUES (1, 'Alice', 25);"
Write-Host "  SELECT * FROM users;"
Write-Host "  SELECT name FROM users WHERE age > 20;"

Write-Host "`n🎯 现在启动交互式shell..." -ForegroundColor Green
Write-Host "提示：输入上面的命令来测试数据库功能！" -ForegroundColor Cyan
Write-Host ""

# 启动交互式shell
.\target\release\minidb.exe