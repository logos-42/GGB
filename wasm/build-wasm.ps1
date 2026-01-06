# WASM构建脚本 (PowerShell)

Write-Host "🔧 构建WASM目标..." -ForegroundColor Cyan

# 检查wasm-bindgen-cli
try {
    $null = Get-Command wasm-bindgen -ErrorAction Stop
    Write-Host "✅ wasm-bindgen已安装" -ForegroundColor Green
} catch {
    Write-Host "📦 安装wasm-bindgen-cli..." -ForegroundColor Yellow
    cargo install wasm-bindgen-cli
}

# 创建输出目录
New-Item -ItemType Directory -Force -Path "wasm/pkg" | Out-Null

# 构建WASM目标
Write-Host "🚀 构建WASM..." -ForegroundColor Cyan
cargo build --target wasm32-unknown-unknown --release --features wasm

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM构建失败" -ForegroundColor Red
    exit 1
}

# 生成绑定
Write-Host "🔗 生成WASM绑定..." -ForegroundColor Cyan
wasm-bindgen `
    --target web `
    --out-dir wasm/pkg `
    --out-name ggb_wasm `
    target/wasm32-unknown-unknown/release/ggb.wasm

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM绑定生成失败" -ForegroundColor Red
    exit 1
}

Write-Host "✅ WASM构建完成！" -ForegroundColor Green
Write-Host "📁 输出目录: wasm/pkg/" -ForegroundColor Yellow
Write-Host "📄 主要文件:" -ForegroundColor Yellow
Write-Host "   - ggb_wasm.js" -ForegroundColor Gray
Write-Host "   - ggb_wasm_bg.wasm" -ForegroundColor Gray
Write-Host "   - ggb_wasm.d.ts" -ForegroundColor Gray
