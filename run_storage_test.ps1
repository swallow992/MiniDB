# MiniDB 存储系统测试启动脚本
# 按照测试与验证要求设计的完整测试方案

Write-Host "=== MiniDB 存储系统测试 ===" -ForegroundColor Green
Write-Host ""

# 确保程序已编译
Write-Host "正在编译存储系统测试程序..." -ForegroundColor Yellow
$result = cargo build --bin storage_system_test 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "编译失败!" -ForegroundColor Red
    Write-Host $result
    exit 1
}
Write-Host "编译成功!" -ForegroundColor Green
Write-Host ""

# 提供测试选择菜单
Write-Host "请选择测试模式:" -ForegroundColor Cyan
Write-Host "1. 完整自动化测试套件 (推荐)"
Write-Host "2. 页式存储系统专项测试"
Write-Host "3. 缓存机制专项测试"
Write-Host "4. 数据操作专项测试"
Write-Host "5. 交互式测试模式"
Write-Host "6. 查看测试说明"
Write-Host ""

$choice = Read-Host "请输入选择 (1-6)"

switch ($choice) {
    "1" {
        Write-Host "运行完整自动化测试套件..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "测试范围:" -ForegroundColor Cyan
        Write-Host "✓ 页式存储系统 (4KB页面分配/释放)" 
        Write-Host "✓ LRU缓存机制验证"
        Write-Host "✓ 模拟数据表操作 (插入/查询/删除)"
        Write-Host "✓ 磁盘I/O与持久化验证"
        Write-Host "✓ 统计信息与性能分析"
        Write-Host ""
        .\target\debug\storage_system_test.exe
    }
    "2" {
        Write-Host "运行页式存储系统专项测试..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "测试内容:" -ForegroundColor Cyan
        Write-Host "• 4KB页面结构验证"
        Write-Host "• 页面分配与释放函数"
        Write-Host "• 页面读写与磁盘I/O模拟"
        Write-Host "• 页面类型管理 (Data/Index/Meta)"
        Write-Host ""
        
        # 创建专项测试配置
        $env:STORAGE_TEST_MODE = "PAGE_ONLY"
        .\target\debug\storage_system_test.exe
        Remove-Item Env:STORAGE_TEST_MODE -ErrorAction SilentlyContinue
    }
    "3" {
        Write-Host "运行缓存机制专项测试..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "测试内容:" -ForegroundColor Cyan
        Write-Host "• LRU缓存替换策略验证"
        Write-Host "• 缓存命中率统计"
        Write-Host "• 缓存刷新与持久化"
        Write-Host "• 不同缓存大小性能对比"
        Write-Host ""
        
        $env:STORAGE_TEST_MODE = "CACHE_ONLY"
        .\target\debug\storage_system_test.exe
        Remove-Item Env:STORAGE_TEST_MODE -ErrorAction SilentlyContinue
    }
    "4" {
        Write-Host "运行数据操作专项测试..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "测试内容:" -ForegroundColor Cyan
        Write-Host "• 构造模拟数据表"
        Write-Host "• 批量插入操作验证"
        Write-Host "• 多条件查询测试"
        Write-Host "• 记录删除与空间回收"
        Write-Host "• 数据表与页面映射关系"
        Write-Host ""
        
        $env:STORAGE_TEST_MODE = "DATA_ONLY"
        .\target\debug\storage_system_test.exe
        Remove-Item Env:STORAGE_TEST_MODE -ErrorAction SilentlyContinue
    }
    "5" {
        Write-Host "启动交互式测试模式..." -ForegroundColor Yellow
        Write-Host ""
        Write-Host "交互命令:" -ForegroundColor Cyan
        Write-Host "• create <table>     - 创建数据表"
        Write-Host "• insert <table> <id> <name> - 插入记录"
        Write-Host "• query <table> [id] - 查询记录"
        Write-Host "• delete <table> <id> - 删除记录"
        Write-Host "• flush              - 刷新缓存"
        Write-Host "• stats              - 显示统计"
        Write-Host "• cache              - 缓存状态"
        Write-Host "• help               - 显示帮助"
        Write-Host "• quit               - 退出"
        Write-Host ""
        Write-Host "提示: 可以手动执行完整的存储操作流程" -ForegroundColor Green
        Write-Host ""
        
        $env:STORAGE_TEST_MODE = "INTERACTIVE"
        .\target\debug\storage_system_test.exe
        Remove-Item Env:STORAGE_TEST_MODE -ErrorAction SilentlyContinue
    }
    "6" {
        Write-Host ""
        Write-Host "=== 存储系统测试说明 ===" -ForegroundColor Green
        Write-Host ""
        Write-Host "📋 测试目标 (严格按照要求设计):" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "1. 页式存储系统设计验证:" -ForegroundColor Yellow
        Write-Host "   • 页结构定义 (4KB页面大小)"
        Write-Host "   • 页分配与释放函数实现"
        Write-Host "   • 页读写函数，模拟磁盘I/O"
        Write-Host ""
        Write-Host "2. 缓存机制实现验证:" -ForegroundColor Yellow
        Write-Host "   • LRU缓存结构设计"
        Write-Host "   • 缓存命中判断与替换策略"
        Write-Host "   • 缓存刷新与持久化机制"
        Write-Host ""
        Write-Host "3. 接口设计与集成验证:" -ForegroundColor Yellow
        Write-Host "   • 统一存储访问接口"
        Write-Host "   • SQL编译器执行计划对接"
        Write-Host "   • 数据表与页的映射关系"
        Write-Host ""
        Write-Host "4. 测试与验证 (核心要求):" -ForegroundColor Yellow
        Write-Host "   ✓ 构造模拟数据表，执行插入、查询、删除操作"
        Write-Host "   ✓ 验证页分配与释放是否正确"
        Write-Host "   ✓ 验证缓存命中率与替换策略效果"
        Write-Host "   ✓ 输出详细日志与统计信息"
        Write-Host ""
        Write-Host "📊 测试输出包括:" -ForegroundColor Cyan
        Write-Host "   • 页面分配/释放统计"
        Write-Host "   • 数据操作计数 (插入/查询/删除)"
        Write-Host "   • 缓存性能指标 (命中率/I/O次数)"
        Write-Host "   • 实时操作日志"
        Write-Host ""
        Write-Host "💡 使用建议:" -ForegroundColor Green
        Write-Host "   • 初次使用选择模式1 (完整测试)"
        Write-Host "   • 深入验证选择模式2-4 (专项测试)"
        Write-Host "   • 手动验证选择模式5 (交互测试)"
        Write-Host ""
        Write-Host "再次运行此脚本可重新选择测试模式。"
    }
    default {
        Write-Host "无效选择，退出。" -ForegroundColor Red
    }
}