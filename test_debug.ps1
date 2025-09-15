#!/usr/bin/env pwsh
# 手动演示MiniDB的调试命令

Write-Host "=== MiniDB 调试命令演示 ===" -ForegroundColor Green

# 准备调试命令测试
$debugCommands = @(
    "\i",      # 显示内部信息
    "\t",      # 运行快速测试
    "\version", # 显示版本信息
    "quit"
)

Write-Host "`n🔧 演示调试命令..." -ForegroundColor Cyan

# 创建输入文件
$inputFile = "debug_input.txt"
$debugCommands | Out-File -FilePath $inputFile -Encoding UTF8

Write-Host "📝 调试命令序列:" -ForegroundColor Yellow
$debugCommands | ForEach-Object { 
    Write-Host "  $_" -ForegroundColor White
}

Write-Host "`n🚀 执行调试命令..." -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Magenta

# 运行调试测试
Get-Content $inputFile | .\target\release\minidb.exe

Write-Host "========================================" -ForegroundColor Magenta
Write-Host "✅ 调试命令演示完成!" -ForegroundColor Green

# 清理
Remove-Item $inputFile -ErrorAction SilentlyContinue