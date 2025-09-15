#!/usr/bin/env pwsh
# 简单的SQL解析测试

Write-Host "=== SQL解析调试测试 ===" -ForegroundColor Green

# 创建简单的SQL测试
$testSQLs = @(
    "CREATE TABLE simple (id INT)",
    "CREATE TABLE users (id INT, name VARCHAR(50))",
    "CREATE TABLE test_table (id INT, name VARCHAR(20))"
)

Write-Host "`n📝 测试SQL语句:" -ForegroundColor Yellow
$testSQLs | ForEach-Object { 
    Write-Host "  $_" -ForegroundColor White
}

# 创建输入文件
$inputFile = "sql_test.txt"
@(
    ($testSQLs | ForEach-Object { $_ + ";" })
    "quit"
) | Out-File -FilePath $inputFile -Encoding UTF8

Write-Host "`n🚀 执行SQL解析测试..." -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Magenta

# 运行测试
Get-Content $inputFile | cargo run

Write-Host "========================================" -ForegroundColor Magenta
Write-Host "✅ SQL解析测试完成!" -ForegroundColor Green

# 清理
Remove-Item $inputFile -ErrorAction SilentlyContinue