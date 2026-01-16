#!/usr/bin/env pwsh
# 设备检测功能测试脚本

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "设备检测功能测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 运行设备检测测试
Write-Host "运行设备检测测试..." -ForegroundColor Yellow
Write-Host ""

cargo test --test device_detection_test -- --nocapture

$testResult = $LASTEXITCODE

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
if ($testResult -eq 0) {
    Write-Host "✓ 所有测试通过！" -ForegroundColor Green
} else {
    Write-Host "✗ 测试失败！" -ForegroundColor Red
}
Write-Host "========================================" -ForegroundColor Cyan

# 提供手动验证建议
Write-Host ""
Write-Host "建议手动验证（Windows）：" -ForegroundColor Yellow
Write-Host "1. 验证内存: systeminfo | findstr "物理内存"" -ForegroundColor Gray
Write-Host "2. 验证GPU: wmic path win32_VideoController get name" -ForegroundColor Gray
Write-Host "3. 验证CPU: wmic cpu get NumberOfCores" -ForegroundColor Gray

exit $testResult
