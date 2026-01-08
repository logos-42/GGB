# 编译和测试脚本

Write-Host "开始编译项目..." -ForegroundColor Green

$ErrorActionPreference = "Stop"

try {
    cargo build 2>&1 | Tee-Object -FilePath build_output.txt

    Write-Host "编译完成！" -ForegroundColor Green
} catch {
    Write-Host "编译失败：$_" -ForegroundColor Red
    Get-Content build_output.txt | Select-String -Pattern "error" | Select-Object -First 20
    exit 1
}
