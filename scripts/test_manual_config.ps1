# GGB 手动配置多节点测试脚本
# 演示如何在同一台机器上模拟三台不同电脑的场景

param(
    [int]$Duration = 30
)

Write-Host "=== GGB 手动配置多节点发现测试 ===" -ForegroundColor Green
Write-Host "运行时长: $Duration 秒"
Write-Host ""

# 创建输出目录
$outputDir = "manual_test_output"
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

# 启动三个节点，使用手动指定的bootstrap地址
$jobs = @()

# 节点0
Write-Host "启动节点0 (端口9234)..." -ForegroundColor Yellow
$job0 = Start-Job -ScriptBlock {
    $outputDir = $using:outputDir
    $statsFile = Join-Path $outputDir "node_0_manual_stats.json"
    $logFile = Join-Path $outputDir "node_0_manual.log"
    cargo run -- --node-id 0 --quic-port 9234 --bootstrap 127.0.0.1:9235 --bootstrap 127.0.0.1:9236 --stats-output $statsFile 2>&1 | Out-File -FilePath $logFile
} -ArgumentList $outputDir
$jobs += $job0
Start-Sleep -Milliseconds 1000

# 节点1
Write-Host "启动节点1 (端口9235)..." -ForegroundColor Yellow
$job1 = Start-Job -ScriptBlock {
    $outputDir = $using:outputDir
    $statsFile = Join-Path $outputDir "node_1_manual_stats.json"
    $logFile = Join-Path $outputDir "node_1_manual.log"
    cargo run -- --node-id 1 --quic-port 9235 --bootstrap 127.0.0.1:9234 --bootstrap 127.0.0.1:9236 --stats-output $statsFile 2>&1 | Out-File -FilePath $logFile
} -ArgumentList $outputDir
$jobs += $job1
Start-Sleep -Milliseconds 1000

# 节点2
Write-Host "启动节点2 (端口9236)..." -ForegroundColor Yellow
$job2 = Start-Job -ScriptBlock {
    $outputDir = $using:outputDir
    $statsFile = Join-Path $outputDir "node_2_manual_stats.json"
    $logFile = Join-Path $outputDir "node_2_manual.log"
    cargo run -- --node-id 2 --quic-port 9236 --bootstrap 127.0.0.1:9234 --bootstrap 127.0.0.1:9235 --stats-output $statsFile 2>&1 | Out-File -FilePath $logFile
} -ArgumentList $outputDir
$jobs += $job2

Write-Host ""
Write-Host "所有节点已启动，等待 $Duration 秒..." -ForegroundColor Green

# 等待指定时间
Start-Sleep -Seconds $Duration

Write-Host ""
Write-Host "停止所有节点..." -ForegroundColor Yellow

# 停止所有作业
$jobs | Stop-Job
$jobs | Remove-Job

Write-Host ""
Write-Host "=== 测试完成 ===" -ForegroundColor Green
Write-Host "日志和统计数据已保存到: $outputDir"
Write-Host ""
Write-Host "查看发现日志：" -ForegroundColor Cyan
Get-ChildItem $outputDir -Filter "*.log" | ForEach-Object {
    Write-Host "  $outputDir\$($_.Name)"
}