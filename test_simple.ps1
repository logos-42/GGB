# Simple 3-node test
$OutputDir = "test_output"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Write-Host "Starting 3 nodes for 60 seconds..." -ForegroundColor Green

# Start nodes in background
$env:WILLIW_DEVICE_TYPE="low"
Start-Process -FilePath "C:\temp\williw-target\debug\williw.exe" -ArgumentList "--node-id 0 --quic-port 9234 --stats-output $OutputDir\node_0_stats.json" -RedirectStandardOutput "$OutputDir\node_0.log" -RedirectStandardError "$OutputDir\node_0_err.log" -PassThru | Out-Null

$env:WILLIW_DEVICE_TYPE="mid"
Start-Process -FilePath "C:\temp\williw-target\debug\williw.exe" -ArgumentList "--node-id 1 --quic-port 9235 --stats-output $OutputDir\node_1_stats.json" -RedirectStandardOutput "$OutputDir\node_1.log" -RedirectStandardError "$OutputDir\node_1_err.log" -PassThru | Out-Null

$env:WILLIW_DEVICE_TYPE="high"
Start-Process -FilePath "C:\temp\williw-target\debug\williw.exe" -ArgumentList "--node-id 2 --quic-port 9236 --stats-output $OutputDir\node_2_stats.json" -RedirectStandardOutput "$OutputDir\node_2.log" -RedirectStandardError "$OutputDir\node_2_err.log" -PassThru | Out-Null

Write-Host "Nodes started. Waiting 120 seconds..." -ForegroundColor Yellow
Start-Sleep -Seconds 120

Write-Host "Stopping nodes..." -ForegroundColor Yellow
Get-Process williw -ErrorAction SilentlyContinue | Stop-Process -Force

Write-Host "Test complete! Check logs in $OutputDir" -ForegroundColor Green
Get-ChildItem $OutputDir | Select-Object Name, Length | Format-Table -AutoSize

