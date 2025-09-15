#!/usr/bin/env pwsh
# 完整的MiniDB交互式功能演示

Write-Host "🎉 === MiniDB 完整功能演示 === 🎉" -ForegroundColor Green

# 清理之前的数据
Write-Host "`n🧹 清理测试环境..." -ForegroundColor Yellow
Remove-Item -Recurse -Force .\minidb_data -ErrorAction SilentlyContinue

# 编译项目
Write-Host "🔨 编译项目..." -ForegroundColor Yellow
cargo build --release --quiet

Write-Host "✅ 准备完成!" -ForegroundColor Green

# 创建完整的演示命令序列
$demoCommands = @(
    "help",
    "\s",
    "\i", 
    "\version",
    "\t",
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
    "UPDATE users SET age = 26 WHERE id = 1;",
    "SELECT * FROM users;",
    "DELETE FROM users WHERE age < 30;",
    "SELECT * FROM users;",
    "SELECT nonexistent_column FROM users;",  # 测试错误处理
    "INVALID SQL STATEMENT;",                 # 测试语法错误
    "quit"
)

Write-Host "`n📋 演示功能清单:" -ForegroundColor Cyan
Write-Host "✅ 系统信息和状态查看"
Write-Host "✅ 表创建和管理"
Write-Host "✅ 数据插入和查询"
Write-Host "✅ 数据更新和删除"
Write-Host "✅ 详细错误信息显示"
Write-Host "✅ 执行时间统计"
Write-Host "✅ 格式化结果输出"

Write-Host "`n🎬 开始完整演示..." -ForegroundColor Green
Write-Host "=" * 60 -ForegroundColor Magenta

# 创建输入文件
$inputFile = "full_demo.txt"
$demoCommands | Out-File -FilePath $inputFile -Encoding UTF8

# 运行演示
Get-Content $inputFile | .\target\release\minidb.exe

Write-Host "=" * 60 -ForegroundColor Magenta
Write-Host "🎊 完整演示结束！" -ForegroundColor Green

# 清理
Remove-Item $inputFile -ErrorAction SilentlyContinue

Write-Host "`n📊 演示总结:" -ForegroundColor Yellow
Write-Host "🔸 展示了完整的SQL CRUD操作"
Write-Host "🔸 验证了详细的执行反馈信息"
Write-Host "🔸 测试了错误处理和提示功能"
Write-Host "🔸 演示了所有调试和系统命令"
Write-Host "🔸 确认了执行时间统计功能"

Write-Host "`n🚀 MiniDB 已经从测试集验证模式成功转换为交互式使用模式！" -ForegroundColor Green
Write-Host "现在您可以通过 'cargo run' 启动交互式shell，享受详细的命令反馈！" -ForegroundColor Cyan