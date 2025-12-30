# 启动隐私保护节点脚本 (PowerShell)
# 使用方法: .\scripts\start_private_node.ps1 [配置文件]

param(
    [string]$ConfigFile = "config\security.toml"
)

Write-Host "=== 启动 GGB 隐私保护节点 ===" -ForegroundColor Cyan
Write-Host "配置文件: $ConfigFile"

# 检查配置文件是否存在
if (-not (Test-Path $ConfigFile)) {
    Write-Host "错误: 配置文件 $ConfigFile 不存在" -ForegroundColor Red
    Write-Host ""
    Write-Host "请创建配置文件或使用示例配置:" -ForegroundColor Yellow
    Write-Host "  copy config\privacy_example.toml config\security.toml"
    Write-Host "  # 编辑 config\security.toml 配置中继节点"
    exit 1
}

# 检查中继节点配置
$configContent = Get-Content $ConfigFile -Raw
if ($configContent -match 'relay_nodes = \[\]') {
    Write-Host "警告: 配置文件中未设置中继节点" -ForegroundColor Yellow
    Write-Host "隐私保护需要至少一个可用的中继节点" -ForegroundColor Yellow
    Write-Host "请编辑 $ConfigFile 添加中继节点地址" -ForegroundColor Yellow
    
    $response = Read-Host "是否继续? (y/N)"
    if ($response -notmatch '^[Yy]$') {
        exit 1
    }
}

# 设置环境变量
$env:RUST_LOG = "info,ggb=debug"
$env:GGB_PRIVACY_MODE = "enabled"

Write-Host ""
Write-Host "启动参数:" -ForegroundColor Green
Write-Host "  - 配置文件: $ConfigFile"
Write-Host "  - 日志级别: $env:RUST_LOG"
Write-Host "  - 隐私模式: $env:GGB_PRIVACY_MODE"
Write-Host ""

# 运行节点
Write-Host "启动节点..." -ForegroundColor Cyan
cargo run --release -- `
    --config $ConfigFile `
    --log-level info

Write-Host ""
Write-Host "节点已停止" -ForegroundColor Cyan
Write-Host "=== 完成 ===" -ForegroundColor Cyan
