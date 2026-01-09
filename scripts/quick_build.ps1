$ErrorActionPreference = "Stop"

Write-Host "🔧 开始构建 WASM..." -ForegroundColor Cyan

# 切换到 wasm 目录
$wasmDir = Join-Path $PSScriptRoot "..\wasm"
Set-Location -Path $wasmDir

# 清理旧的构建产物
if (Test-Path pkg) {
    Write-Host "🧹 清理旧的构建产物..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force pkg
}

# 创建 pkg 目录
New-Item -ItemType Directory -Force -Path pkg | Out-Null

# 构建 WASM
Write-Host "🚀 使用 wasm-pack 构建..." -ForegroundColor Cyan
& wasm-pack build --target web --out-dir pkg

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM 构建失败" -ForegroundColor Red
    exit 1
}

Write-Host "✅ WASM 构建完成！" -ForegroundColor Green

# 列出构建产物
Write-Host "`n📦 构建产物:" -ForegroundColor Cyan
Get-ChildItem pkg -File | ForEach-Object {
    $size = [math]::Round($_.Length / 1KB, 2)
    Write-Host "  - $($_.Name) ($size KB)" -ForegroundColor Gray
}
