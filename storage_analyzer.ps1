# 存储层分析工具
# 用于深入分析 MiniDB 的底层存储结构和数据页内容

param(
    [string]$DataDir = "minidb_data",
    [switch]$Detailed = $false,
    [switch]$ExportAnalysis = $false
)

function Write-StorageHeader($message) {
    Write-Host "`n" -NoNewline
    Write-Host "=" * 50 -ForegroundColor Cyan
    Write-Host " $message" -ForegroundColor Yellow
    Write-Host "=" * 50 -ForegroundColor Cyan
}

function Analyze-StorageStructure($dataPath) {
    Write-StorageHeader "存储结构分析"
    
    if (-not (Test-Path $dataPath)) {
        Write-Host "❌ 数据目录不存在: $dataPath" -ForegroundColor Red
        return $false
    }
    
    $items = Get-ChildItem $dataPath -Recurse
    Write-Host "📁 数据目录: $dataPath" -ForegroundColor Green
    Write-Host "📊 总文件数: $($items.Count)" -ForegroundColor Cyan
    
    # 按类型分组分析
    $fileGroups = $items | Where-Object { -not $_.PSIsContainer } | Group-Object Extension
    
    Write-Host "`n文件类型分布:" -ForegroundColor Yellow
    foreach ($group in $fileGroups) {
        $ext = if ($group.Name) { $group.Name } else { "(无扩展名)" }
        $totalSize = ($group.Group | Measure-Object Length -Sum).Sum
        Write-Host "  $ext : $($group.Count) 个文件, 总大小: $totalSize 字节" -ForegroundColor Gray
    }
    
    return $true
}

function Analyze-PageFiles($dataPath) {
    Write-StorageHeader "数据页分析"
    
    $pageFiles = Get-ChildItem $dataPath -Recurse | Where-Object { 
        $_.Name -match "\.(page|data|db)$" -or $_.Name -match "page" 
    }
    
    if ($pageFiles.Count -eq 0) {
        Write-Host "ℹ️  未发现页文件" -ForegroundColor Blue
        return
    }
    
    Write-Host "发现 $($pageFiles.Count) 个页文件:" -ForegroundColor Green
    
    foreach ($pageFile in $pageFiles) {
        Write-Host "`n📄 文件: $($pageFile.Name)" -ForegroundColor Cyan
        Write-Host "   大小: $($pageFile.Length) 字节" -ForegroundColor Gray
        Write-Host "   修改时间: $($pageFile.LastWriteTime)" -ForegroundColor Gray
        
        if ($Detailed) {
            try {
                # 尝试以文本形式读取内容
                $content = Get-Content $pageFile.FullName -Raw -Encoding UTF8 -ErrorAction Stop
                
                Write-Host "   内容预览:" -ForegroundColor Yellow
                $preview = $content.Substring(0, [Math]::Min(200, $content.Length))
                Write-Host "   $($preview.Replace("`n", " ").Replace("`r", ""))" -ForegroundColor Gray
                
                # 检查是否包含已知的测试数据
                $knownPatterns = @(
                    "employees", "products", "persistent_test", "perf_test",
                    "Alice", "Bob", "Carol", "Engineering", "Marketing"
                )
                
                $foundPatterns = @()
                foreach ($pattern in $knownPatterns) {
                    if ($content -match $pattern) {
                        $foundPatterns += $pattern
                    }
                }
                
                if ($foundPatterns.Count -gt 0) {
                    Write-Host "   ✅ 发现数据模式: $($foundPatterns -join ', ')" -ForegroundColor Green
                }
                
            } catch {
                # 尝试以二进制形式读取
                try {
                    $bytes = [System.IO.File]::ReadAllBytes($pageFile.FullName)
                    Write-Host "   📊 二进制文件，大小: $($bytes.Length) 字节" -ForegroundColor Yellow
                    
                    # 分析字节分布
                    $nullBytes = ($bytes | Where-Object { $_ -eq 0 }).Count
                    $printableBytes = ($bytes | Where-Object { $_ -ge 32 -and $_ -le 126 }).Count
                    
                    Write-Host "   空字节: $nullBytes ($([math]::Round($nullBytes/$bytes.Length*100,1))%)" -ForegroundColor Gray
                    Write-Host "   可打印字节: $printableBytes ($([math]::Round($printableBytes/$bytes.Length*100,1))%)" -ForegroundColor Gray
                    
                } catch {
                    Write-Host "   ❌ 无法读取文件: $($_.Exception.Message)" -ForegroundColor Red
                }
            }
        }
    }
}

function Analyze-DataIntegrity($dataPath) {
    Write-StorageHeader "数据完整性检查"
    
    # 检查文件时间戳一致性
    $allFiles = Get-ChildItem $dataPath -Recurse -File
    
    if ($allFiles.Count -eq 0) {
        Write-Host "ℹ️  目录为空" -ForegroundColor Blue
        return
    }
    
    $newestFile = $allFiles | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $oldestFile = $allFiles | Sort-Object LastWriteTime | Select-Object -First 1
    
    Write-Host "📅 最新文件: $($newestFile.Name) ($($newestFile.LastWriteTime))" -ForegroundColor Green
    Write-Host "📅 最旧文件: $($oldestFile.Name) ($($oldestFile.LastWriteTime))" -ForegroundColor Green
    
    $timeDiff = $newestFile.LastWriteTime - $oldestFile.LastWriteTime
    Write-Host "⏱️  时间跨度: $($timeDiff.TotalMinutes.ToString('F1')) 分钟" -ForegroundColor Cyan
    
    # 检查文件大小分布
    $totalSize = ($allFiles | Measure-Object Length -Sum).Sum
    $avgSize = if ($allFiles.Count -gt 0) { $totalSize / $allFiles.Count } else { 0 }
    
    Write-Host "📊 总大小: $totalSize 字节" -ForegroundColor Cyan
    Write-Host "📊 平均大小: $($avgSize.ToString('F0')) 字节" -ForegroundColor Cyan
    
    # 检查是否有损坏的文件
    $corruptedFiles = @()
    foreach ($file in $allFiles) {
        if ($file.Length -eq 0) {
            $corruptedFiles += $file.Name
        }
    }
    
    if ($corruptedFiles.Count -gt 0) {
        Write-Host "⚠️  发现空文件: $($corruptedFiles -join ', ')" -ForegroundColor Yellow
    } else {
        Write-Host "✅ 所有文件大小正常" -ForegroundColor Green
    }
}

function Test-DataPersistence($dataPath) {
    Write-StorageHeader "持久性验证测试"
    
    Write-Host "🔍 创建测试数据..." -ForegroundColor Yellow
    
    # 创建测试数据
    $testSQL = @"
CREATE TABLE storage_test (
    id INT,
    test_data TEXT,
    created_at TEXT
);

INSERT INTO storage_test VALUES (1, 'StorageTest_Data1', '2025-09-15 12:00:00');
INSERT INTO storage_test VALUES (2, 'StorageTest_Data2', '2025-09-15 12:01:00');

SELECT * FROM storage_test;
"@

    # 记录执行前的文件状态
    $beforeFiles = @()
    if (Test-Path $dataPath) {
        $beforeFiles = Get-ChildItem $dataPath -Recurse -File
    }
    
    # 执行SQL
    $output = echo $testSQL | cargo run 2>&1
    Start-Sleep -Seconds 1  # 等待文件系统同步
    
    # 记录执行后的文件状态
    $afterFiles = @()
    if (Test-Path $dataPath) {
        $afterFiles = Get-ChildItem $dataPath -Recurse -File
    }
    
    # 分析文件变化
    $newFiles = $afterFiles | Where-Object { $_.FullName -notin $beforeFiles.FullName }
    $modifiedFiles = $afterFiles | Where-Object { 
        $before = $beforeFiles | Where-Object { $_.FullName -eq $_.FullName }
        $before -and $_.LastWriteTime -gt $before.LastWriteTime
    }
    
    Write-Host "📝 执行结果:" -ForegroundColor Green
    Write-Host "   新文件: $($newFiles.Count) 个" -ForegroundColor Gray
    Write-Host "   修改文件: $($modifiedFiles.Count) 个" -ForegroundColor Gray
    
    # 检查数据是否正确写入
    $dataFound = $false
    foreach ($file in $afterFiles) {
        try {
            $content = Get-Content $file.FullName -Raw -ErrorAction SilentlyContinue
            if ($content -and $content -match "StorageTest_Data") {
                $dataFound = $true
                Write-Host "   ✅ 在 $($file.Name) 中发现测试数据" -ForegroundColor Green
                break
            }
        } catch {
            # 忽略读取错误
        }
    }
    
    if (-not $dataFound) {
        Write-Host "   ⚠️  未在文件中发现测试数据" -ForegroundColor Yellow
    }
    
    # 验证数据可读性
    Write-Host "`n🔍 验证数据可读性..." -ForegroundColor Yellow
    $readSQL = "SELECT * FROM storage_test;"
    $readOutput = echo $readSQL | cargo run 2>&1
    
    if ($readOutput -match "StorageTest_Data") {
        Write-Host "   ✅ 数据读取成功" -ForegroundColor Green
    } else {
        Write-Host "   ❌ 数据读取失败" -ForegroundColor Red
    }
}

function Export-AnalysisReport($dataPath) {
    if (-not $ExportAnalysis) { return }
    
    Write-StorageHeader "导出分析报告"
    
    $reportPath = "storage_analysis_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
    
    $analysis = @{
        Timestamp = Get-Date
        DataPath = $dataPath
        Structure = @{}
        Files = @()
        Summary = @{}
    }
    
    if (Test-Path $dataPath) {
        $allFiles = Get-ChildItem $dataPath -Recurse
        
        $analysis.Structure = @{
            TotalFiles = ($allFiles | Where-Object { -not $_.PSIsContainer }).Count
            TotalDirectories = ($allFiles | Where-Object { $_.PSIsContainer }).Count
            TotalSize = ($allFiles | Where-Object { -not $_.PSIsContainer } | Measure-Object Length -Sum).Sum
        }
        
        foreach ($file in ($allFiles | Where-Object { -not $_.PSIsContainer })) {
            $analysis.Files += @{
                Name = $file.Name
                Path = $file.FullName
                Size = $file.Length
                LastModified = $file.LastWriteTime
                Extension = $file.Extension
            }
        }
    }
    
    $analysis | ConvertTo-Json -Depth 5 | Out-File -FilePath $reportPath -Encoding UTF8
    Write-Host "📄 分析报告已导出: $reportPath" -ForegroundColor Green
}

# 主分析流程
Write-Host "🔍 MiniDB 存储层分析工具" -ForegroundColor Cyan
Write-Host "分析目录: $DataDir" -ForegroundColor Gray
Write-Host "详细模式: $Detailed" -ForegroundColor Gray

try {
    if (Analyze-StorageStructure $DataDir) {
        Analyze-PageFiles $DataDir
        Analyze-DataIntegrity $DataDir
        Test-DataPersistence $DataDir
        Export-AnalysisReport $DataDir
    }
    
    Write-Host "`n✅ 存储层分析完成" -ForegroundColor Green
    
} catch {
    Write-Host "`n❌ 分析过程中发生错误: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}