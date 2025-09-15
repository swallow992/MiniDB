# MiniDB SQL编译器启动脚本
# 使用方法: .\run_sql_compiler.ps1

Write-Host "=== MiniDB SQL编译器 ===" -ForegroundColor Green
Write-Host ""

# 确保程序已编译
Write-Host "正在编译SQL编译器..." -ForegroundColor Yellow
$result = cargo build --bin sql_compiler_demo 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "编译失败!" -ForegroundColor Red
    Write-Host $result
    exit 1
}
Write-Host "编译成功!" -ForegroundColor Green
Write-Host ""

# 提供选择菜单
Write-Host "请选择运行模式:" -ForegroundColor Cyan
Write-Host "1. 运行标准测试套件 (推荐)"
Write-Host "2. 交互模式"
Write-Host "3. 查看帮助"
Write-Host ""

$choice = Read-Host "请输入选择 (1-3)"

switch ($choice) {
    "1" {
        Write-Host "运行标准测试套件..." -ForegroundColor Yellow
        Write-Host ""
        Get-Content sql_tests.sql | .\target\debug\sql_compiler_demo.exe
    }
    "2" {
        Write-Host "启动交互模式..." -ForegroundColor Yellow
        Write-Host "提示: 输入SQL语句，使用 'quit' 或 'exit' 退出"
        Write-Host ""
        .\target\debug\sql_compiler_demo.exe
    }
    "3" {
        Write-Host ""
        Write-Host "=== SQL编译器使用帮助 ===" -ForegroundColor Green
        Write-Host ""
        Write-Host "支持的SQL语法:" -ForegroundColor Cyan
        Write-Host "  CREATE TABLE table_name (col1 datatype, col2 datatype, ...)"
        Write-Host "  INSERT INTO table_name VALUES (val1, val2, ...)"
        Write-Host "  INSERT INTO table_name (col1, col2, ...) VALUES (val1, val2, ...)"
        Write-Host "  SELECT * FROM table_name"
        Write-Host "  SELECT col1, col2 FROM table_name WHERE condition"
        Write-Host ""
        Write-Host "支持的数据类型:" -ForegroundColor Cyan
        Write-Host "  INT - 整数"
        Write-Host "  VARCHAR(n) - 变长字符串"
        Write-Host "  DECIMAL(p,s) - 定点数"
        Write-Host ""
        Write-Host "文件说明:" -ForegroundColor Cyan
        Write-Host "  sql_tests.sql - 标准测试用例文件"
        Write-Host "  SQL_COMPILER_GUIDE.md - 详细使用指南"
        Write-Host ""
        Write-Host "再次运行此脚本可重新选择模式。"
    }
    default {
        Write-Host "无效选择，退出。" -ForegroundColor Red
    }
}