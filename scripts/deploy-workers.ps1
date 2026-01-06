# Cloudflare Workers部署脚本

param(
    [string]$Environment = "production",
    [switch]$BuildOnly = $false,
    [switch]$TestOnly = $false
)

Write-Host "🚀 部署GGB到Cloudflare Workers..." -ForegroundColor Cyan
Write-Host "环境: $Environment" -ForegroundColor Yellow

# 检查必要工具
function Check-Tool {
    param([string]$ToolName, [string]$InstallCommand)
    
    try {
        $null = Get-Command $ToolName -ErrorAction Stop
        Write-Host "✅ $ToolName 已安装" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "❌ $ToolName 未安装" -ForegroundColor Red
        Write-Host "请运行: $InstallCommand" -ForegroundColor Yellow
        return $false
    }
}

# 检查工具
$toolsOk = $true
$toolsOk = $toolsOk -and (Check-Tool "cargo" "安装Rust: https://rustup.rs/")
$toolsOk = $toolsOk -and (Check-Tool "wasm-pack" "cargo install wasm-pack")
$toolsOk = $toolsOk -and (Check-Tool "wrangler" "npm install -g wrangler")

if (-not $toolsOk) {
    Write-Host "❌ 必要工具缺失，请先安装上述工具" -ForegroundColor Red
    exit 1
}

# 设置环境变量
$env:CARGO_TARGET_DIR = "target-wasm"
$env:RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory,+mutable-globals"

# 构建WASM
Write-Host "🔨 构建WASM..." -ForegroundColor Cyan
cargo build --target wasm32-unknown-unknown --release --features workers

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM构建失败" -ForegroundColor Red
    exit 1
}

Write-Host "🔗 生成WASM绑定..." -ForegroundColor Cyan
wasm-pack build --target web --out-dir workers/pkg --out-name ggb_wasm --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WASM绑定生成失败" -ForegroundColor Red
    exit 1
}

if ($BuildOnly) {
    Write-Host "✅ 构建完成，跳过部署" -ForegroundColor Green
    exit 0
}

# 测试
if ($TestOnly) {
    Write-Host "🧪 启动本地测试服务器..." -ForegroundColor Cyan
    wrangler dev
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 本地测试启动失败" -ForegroundColor Red
        exit 1
    }
    
    exit 0
}

# 部署到Cloudflare Workers
Write-Host "☁️  部署到Cloudflare Workers..." -ForegroundColor Cyan

# 根据环境选择配置
$wranglerArgs = @("publish")
if ($Environment -eq "staging") {
    $wranglerArgs += "--env", "staging"
}

wrangler @wranglerArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 部署失败" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 部署成功！" -ForegroundColor Green
Write-Host "🌐 访问地址: https://ggb-edge-server.your-account.workers.dev" -ForegroundColor Yellow
Write-Host "📊 监控面板: https://dash.cloudflare.com/" -ForegroundColor Yellow

# 运行健康检查
Write-Host "🏥 运行健康检查..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

try {
    $healthResponse = Invoke-RestMethod -Uri "https://ggb-edge-server.your-account.workers.dev/health" -Method Get
    Write-Host "✅ 健康检查通过:" -ForegroundColor Green
    Write-Host ($healthResponse | ConvertTo-Json -Depth 3) -ForegroundColor Gray
} catch {
    Write-Host "⚠️  健康检查失败: $_" -ForegroundColor Yellow
}

Write-Host "🎉 部署流程完成！" -ForegroundColor Green
