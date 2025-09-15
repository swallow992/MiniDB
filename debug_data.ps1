#!/usr/bin/env pwsh
# 数据存储调试测试

Write-Host "=== 数据存储调试测试 ===" -ForegroundColor Green

# 清理数据
Remove-Item -Recurse -Force .\minidb_data -ErrorAction SilentlyContinue

$testCommands = @(
    "CREATE TABLE debug_test (id INT, name VARCHAR(10));",
    "\d",
    "INSERT INTO debug_test VALUES (1, 'Test');",
    "SELECT * FROM debug_test;",
    "\d",
    "quit"
)

Write-Host "`n📝 调试命令序列:" -ForegroundColor Yellow
$testCommands | ForEach-Object { 
    Write-Host "  $_" -ForegroundColor White
}

# 创建输入文件
$inputFile = "debug_data.txt"
$testCommands | Out-File -FilePath $inputFile -Encoding UTF8

Write-Host "`n🚀 执行调试测试..." -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Magenta

# 运行测试
Get-Content $inputFile | cargo run

Write-Host "========================================" -ForegroundColor Magenta
Write-Host "✅ 调试测试完成!" -ForegroundColor Green

# 清理
Remove-Item $inputFile -ErrorAction SilentlyContinue