# 快速测试设备检测功能
# 直接运行方式：
# . "快速测试设备检测.ps1"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "设备检测功能快速测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查编译环境
Write-Host "检查Rust环境..." -ForegroundColor Yellow
$rustVersion = rustc --version
Write-Host "✓ Rust版本: $rustVersion" -ForegroundColor Green
Write-Host ""

# 尝试编译
Write-Host "编译设备检测程序..." -ForegroundColor Yellow
$compileResult = cargo build --release --bin verify_detection 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ 编译成功！" -ForegroundColor Green
    Write-Host ""
    
    # 运行检测程序
    Write-Host "运行设备检测..." -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Gray
    & target\release\verify_detection.exe
    Write-Host "========================================" -ForegroundColor Gray
    Write-Host ""
    
    Write-Host "✅ 设备检测完成！" -ForegroundColor Green
} else {
    Write-Host "❌ 编译失败！" -ForegroundColor Red
    Write-Host ""
    Write-Host "错误信息:" -ForegroundColor Red
    $compileResult | Select-String -Pattern "error" -Context 0,2 | ForEach-Object {
        Write-Host $_ -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "测试完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
