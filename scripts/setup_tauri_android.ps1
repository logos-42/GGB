# Tauri 2.x Android 设置脚本
# 解决 "Target android does not exist" 错误

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Tauri 2.x Android 配置" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 1. 检查并安装 NDK
Write-Host "`n步骤 1: 检查 Android NDK" -ForegroundColor Yellow

$NdkPath = "$env:USERPROFILE\AppData\Local\Android\Sdk\ndk"
if (Test-Path $NdkPath) {
    $NdkVersions = Get-ChildItem $NdkPath | Select-Object -ExpandProperty Name
    Write-Host "✅ NDK 已安装:" -ForegroundColor Green
    foreach ($Version in $NdkVersions) {
        Write-Host "  - $Version" -ForegroundColor Gray
    }
} else {
    Write-Host "❌ NDK 未安装" -ForegroundColor Red
    Write-Host "`n请按照以下步骤安装 NDK：" -ForegroundColor Yellow
    Write-Host "1. 打开 Android Studio" -ForegroundColor Gray
    Write-Host "2. Tools → SDK Manager" -ForegroundColor Gray
    Write-Host "3. SDK Tools 标签页" -ForegroundColor Gray
    Write-Host "4. 勾选 'Show Package Details'" -ForegroundColor Gray
    Write-Host "5. 展开 'NDK (Side by side)'" -ForegroundColor Gray
    Write-Host "6. 勾选 '25.2.9519653' 或最新版本" -ForegroundColor Gray
    Write-Host "7. 点击 Apply 安装" -ForegroundColor Gray
    Write-Host "`n安装后重新运行此脚本" -ForegroundColor Yellow
    exit 1
}

# 2. 设置环境变量
Write-Host "`n步骤 2: 设置环境变量" -ForegroundColor Yellow

$LatestNdk = Get-ChildItem $NdkPath | Sort-Object Name -Descending | Select-Object -First 1
$env:ANDROID_NDK_HOME = "$NdkPath\$($LatestNdk.Name)"
$env:ANDROID_HOME = "$env:USERPROFILE\AppData\Local\Android\Sdk"

Write-Host "ANDROID_NDK_HOME = $env:ANDROID_NDK_HOME" -ForegroundColor Green
Write-Host "ANDROID_HOME = $env:ANDROID_HOME" -ForegroundColor Green

# 3. 检查 Rust Android 目标
Write-Host "`n步骤 3: 检查 Rust Android 目标" -ForegroundColor Yellow

$Targets = @("aarch64-linux-android", "armv7-linux-androideabi", "i686-linux-android", "x86_64-linux-android")
$InstalledTargets = rustup target list --installed

$AllInstalled = $true
foreach ($Target in $Targets) {
    if ($InstalledTargets -like "*$Target*") {
        Write-Host "✅ $Target 已安装" -ForegroundColor Green
    } else {
        Write-Host "❌ $Target 未安装" -ForegroundColor Red
        $AllInstalled = $false
    }
}

if (-not $AllInstalled) {
    Write-Host "`n安装 Android Rust 目标..." -ForegroundColor Yellow
    rustup target add $Targets
}

# 4. 安装 cargo-ndk
Write-Host "`n步骤 4: 检查 cargo-ndk" -ForegroundColor Yellow

$CargoNdkInstalled = cargo install --list | Select-String "cargo-ndk"
if (-not $CargoNdkInstalled) {
    Write-Host "安装 cargo-ndk..." -ForegroundColor Yellow
    cargo install cargo-ndk
} else {
    Write-Host "✅ cargo-ndk 已安装" -ForegroundColor Green
}

# 5. 初始化 Android 项目
Write-Host "`n步骤 5: 初始化 Tauri Android 项目" -ForegroundColor Yellow

Set-Location "d:\AI\去中心化训练"

Write-Host "尝试初始化 Android 项目..." -ForegroundColor Gray
npx tauri android init

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Android 项目初始化成功" -ForegroundColor Green
} else {
    Write-Host "⚠️  初始化可能失败，继续下一步" -ForegroundColor Yellow
}

# 6. 构建命令说明
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "构建命令" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n开发模式（推荐）：" -ForegroundColor Yellow
Write-Host "npx tauri android dev" -ForegroundColor Cyan

Write-Host "`n构建发布版：" -ForegroundColor Yellow
Write-Host "npx tauri android build" -ForegroundColor Cyan

Write-Host "`n查看可用命令：" -ForegroundColor Yellow
Write-Host "npx tauri android --help" -ForegroundColor Cyan

# 7. 检查设备连接
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "检查设备连接" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$devices = adb devices 2>$null | Select-String -Pattern "device$" -CaseSensitive:$false
if ($devices) {
    Write-Host "✅ 已连接设备：" -ForegroundColor Green
    adb devices 2>$null | Select-String -Pattern "device$" -CaseSensitive:$false
} else {
    Write-Host "⚠️  未找到连接的设备" -ForegroundColor Yellow
    Write-Host "请启动模拟器或连接设备" -ForegroundColor Gray
}

Write-Host "`n配置完成！" -ForegroundColor Green
