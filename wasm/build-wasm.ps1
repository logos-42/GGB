# WASM构建脚本 (PowerShell)

Write-Host "🔧 构建WASM目标..." -ForegroundColor Cyan

# 检查wasm-pack
try {
    $null = Get-Command wasm-pack -ErrorAction Stop
    Write-Host "✅ wasm-pack已安装" -ForegroundColor Green
} catch {
    Write-Host "📦 安装wasm-pack..." -ForegroundColor Yellow
    cargo install wasm-pack
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ wasm-pack安装失败" -ForegroundColor Red
    exit 1
}

# 保存当前目录
$originalDir = Get-Location

# 切换到wasm目录
Set-Location -Path "$PSScriptRoot"

# 创建输出目录
New-Item -ItemType Directory -Force -Path "pkg" | Out-Null

# 构建WASM目标
Write-Host "🚀 构建WASM..." -ForegroundColor Cyan
wasm-pack build --target web --out-dir pkg

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM构建失败" -ForegroundColor Red
    Set-Location -Path $originalDir
    exit 1
}

# 返回原目录
Set-Location -Path $originalDir

Write-Host "✅ WASM构建完成！" -ForegroundColor Green
Write-Host "📁 输出目录: wasm/pkg/" -ForegroundColor Yellow
Write-Host "📄 主要文件:" -ForegroundColor Yellow
Write-Host "   - ggb_wasm.js" -ForegroundColor Gray
Write-Host "   - ggb_wasm_bg.wasm" -ForegroundColor Gray
Write-Host "   - ggb_wasm.d.ts" -ForegroundColor Gray
