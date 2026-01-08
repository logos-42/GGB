# Simplified multi-node test script
param(
    [int]$Nodes = 3,
    [int]$Duration = 60
)

Write-Host "=== williw Multi-Node Test ===" -ForegroundColor Green
Write-Host "Nodes: $Nodes"
Write-Host "Duration: $Duration seconds"
Write-Host ""

# Create output directory
$OutputDir = "test_output"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Start multiple nodes
$jobs = @()
for ($i = 0; $i -lt $Nodes; $i++) {
    $nodeId = $i
    $statsFile = Join-Path $OutputDir "node_${nodeId}_stats.json"
    $logFile = Join-Path $OutputDir "node_${nodeId}.log"
    
    # Set device type
    $deviceType = switch ($i % 3) {
        0 { "low" }
        1 { "mid" }
        default { "high" }
    }
    
    Write-Host "Starting node $nodeId (device: $deviceType)..." -ForegroundColor Yellow
    
    $quicPort = 9234 + $nodeId
    # Use debug build if release doesn't work
    $exePath = if (Test-Path "C:\temp\williw-target\debug\williw.exe") {
        "C:\temp\williw-target\debug\williw.exe"
    } else {
        (Resolve-Path "target\release\williw.exe")
    }
    $job = Start-Job -ScriptBlock {
        param($nodeId, $statsFile, $logFile, $deviceType, $exePath, $quicPort)
        $env:GGB_DEVICE_TYPE = $deviceType
        $env:RUST_LOG = "info"
        $env:GGB_QUIC_PORT = $quicPort
        Set-Location $using:PWD
        & $exePath --node-id $nodeId --stats-output $statsFile --quic-port $quicPort 2>&1 | Out-File -FilePath $logFile -Encoding utf8
    } -ArgumentList $nodeId, $statsFile, $logFile, $deviceType, $exePath, $quicPort
    
    $jobs += $job
    Start-Sleep -Milliseconds 500
}

Write-Host ""
Write-Host "All nodes started. Waiting $Duration seconds..." -ForegroundColor Green
Start-Sleep -Seconds $Duration

Write-Host ""
Write-Host "Stopping all nodes..." -ForegroundColor Yellow
$jobs | Stop-Job
$jobs | Remove-Job

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Green
Write-Host "Results saved to: $OutputDir"
Write-Host ""
Write-Host "To analyze results, run:" -ForegroundColor Cyan
Write-Host "  cargo run --bin analyze_training -- --input $OutputDir"

