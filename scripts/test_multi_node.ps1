# GGB 多节点测试脚本 (PowerShell)
# 使用方法: .\scripts\test_multi_node.ps1 -Nodes 3 -Duration 300

param(
    [int]$Nodes = 3,
    [int]$Duration = 300,
    [int]$ModelDim = 256,
    [string]$OutputDir = "test_output"
)

Write-Host "=== GGB 多节点协同训练测试 ===" -ForegroundColor Green
Write-Host "节点数量: $Nodes"
Write-Host "训练时长: $Duration 秒"
Write-Host "模型维度: $ModelDim"
Write-Host "输出目录: $OutputDir"
Write-Host ""

# 创建输出目录
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# 启动多个节点
$jobs = @()
for ($i = 0; $i -lt $Nodes; $i++) {
    $nodeId = $i
    $statsFile = Join-Path $OutputDir "node_${nodeId}_stats.json"
    $logFile = Join-Path $OutputDir "node_${nodeId}.log"
    
    # 设置设备类型
    $deviceType = switch ($i % 3) {
        0 { "low" }
        1 { "mid" }
        default { "high" }
    }
    
    Write-Host "启动节点 $nodeId (设备类型: $deviceType)..." -ForegroundColor Yellow
    
    $env:GGB_DEVICE_TYPE = $deviceType
    $env:RUST_LOG = "info"
    
    $job = Start-Job -ScriptBlock {
        param($nodeId, $statsFile, $logFile)
        $env:GGB_DEVICE_TYPE = $using:deviceType
        cargo run --release -- --node-id $nodeId --stats-output $statsFile 2>&1 | Out-File -FilePath $logFile
    } -ArgumentList $nodeId, $statsFile, $logFile
    
    $jobs += $job
    
    # 错开启动时间
    Start-Sleep -Milliseconds 500
}

Write-Host ""
Write-Host "所有节点已启动，开始训练..." -ForegroundColor Green
Write-Host "等待 $Duration 秒后自动停止..." -ForegroundColor Yellow
Write-Host ""

# 等待指定时间
Start-Sleep -Seconds $Duration

Write-Host ""
Write-Host "训练时间到，正在停止所有节点..." -ForegroundColor Yellow

# 停止所有作业
$jobs | Stop-Job
$jobs | Remove-Job

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
Write-Host "统计数据已保存到: $OutputDir"
Write-Host ""
Write-Host "可以使用以下命令分析结果：" -ForegroundColor Cyan
Write-Host "  cargo run --bin analyze_training -- --input $OutputDir"

