# 安装和配置 Android NDK 脚本
# 使用: .\scripts\install_android_ndk.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "安装 Android NDK" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 设置 SDK 路径
$ANDROID_HOME = "$env:USERPROFILE\AppData\Local\Android\Sdk"

if (-not (Test-Path $ANDROID_HOME)) {
    Write-Host "❌ Android SDK 未找到: $ANDROID_HOME" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Android SDK 路径: $ANDROID_HOME" -ForegroundColor Green

# 方法 1：使用 Android Studio SDK Manager 安装 NDK（推荐）
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "方法 1：通过 Android Studio 安装 NDK（推荐）" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n请按以下步骤操作：" -ForegroundColor Yellow
Write-Host "1. 打开 Android Studio" -ForegroundColor Gray
Write-Host "2. 点击 Tools → SDK Manager" -ForegroundColor Gray
Write-Host "3. 切换到 'SDK Tools' 标签页" -ForegroundColor Gray
Write-Host "4. 勾选 'Show Package Details'" -ForegroundColor Gray
Write-Host "5. 展开 'NDK (Side by side)'" -ForegroundColor Gray
Write-Host "6. 勾选 '25.2.9519653' 或最新版本" -ForegroundColor Gray
Write-Host "7. 点击 Apply 安装" -ForegroundColor Gray

Write-Host "`n安装后，运行以下命令验证：" -ForegroundColor Yellow
Write-Host "Test-Path `$ANDROID_HOME\ndk" -ForegroundColor Cyan
Write-Host "Get-ChildItem `$ANDROID_HOME\ndk" -ForegroundColor Cyan

# 方法 2：使用命令行安装（如果已安装 cmdline-tools）
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "方法 2：使用命令行安装" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$SdkManagerPath = "$ANDROID_HOME\cmdline-tools\latest\bin\sdkmanager.bat"
if (Test-Path $SdkManagerPath) {
    Write-Host "找到 SDK Manager，尝试安装 NDK..." -ForegroundColor Green

    # 安装 NDK
    & $SdkManagerPath "ndk;25.2.9519653"

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ NDK 安装成功" -ForegroundColor Green
    } else {
        Write-Host "❌ NDK 安装失败，请使用方法 1 手动安装" -ForegroundColor Red
    }
} else {
    Write-Host "未找到 SDK Manager 命令行工具" -ForegroundColor Yellow
    Write-Host "请使用方法 1 通过 Android Studio 安装" -ForegroundColor Yellow
}

# 验证 NDK 安装
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "验证 NDK 安装" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$NdkPath = "$ANDROID_HOME\ndk"
if (Test-Path $NdkPath) {
    Write-Host "✅ NDK 目录存在: $NdkPath" -ForegroundColor Green

    $NdkVersions = Get-ChildItem $NdkPath | Select-Object -ExpandProperty Name
    Write-Host "已安装的 NDK 版本：" -ForegroundColor Cyan
    foreach ($Version in $NdkVersions) {
        Write-Host "  - $Version" -ForegroundColor Gray
    }

    # 设置环境变量
    $LatestNdk = Get-ChildItem $NdkPath | Sort-Object Name -Descending | Select-Object -First 1
    $env:ANDROID_NDK_HOME = "$NdkPath\$($LatestNdk.Name)"

    Write-Host "`n设置环境变量：" -ForegroundColor Yellow
    Write-Host "ANDROID_NDK_HOME = $env:ANDROID_NDK_HOME" -ForegroundColor Green
} else {
    Write-Host "❌ NDK 未安装" -ForegroundColor Red
    Write-Host "`n请按照方法 1 的步骤安装 NDK" -ForegroundColor Yellow
    exit 1
}

# 安装 cargo-ndk
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "安装 cargo-ndk" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$CargoNdkInstalled = cargo install --list | Select-String "cargo-ndk"
if (-not $CargoNdkInstalled) {
    Write-Host "安装 cargo-ndk..." -ForegroundColor Yellow
    cargo install cargo-ndk

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ cargo-ndk 安装成功" -ForegroundColor Green
    } else {
        Write-Host "⚠️  cargo-ndk 安装失败，可以稍后手动安装" -ForegroundColor Yellow
    }
} else {
    Write-Host "✅ cargo-ndk 已安装" -ForegroundColor Green
}

# 更新 local.properties
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "更新配置文件" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$LocalPropsPath = "src-tauri\gen\android\local.properties"
if (Test-Path $LocalPropsPath) {
    $Content = Get-Content $LocalPropsPath
    if ($Content -notmatch "ndk.dir") {
        Add-Content -Path $LocalPropsPath -Value "ndk.dir=$($LatestNdk.FullName -replace '\\', '\\\\')"
        Write-Host "✅ 已更新 local.properties" -ForegroundColor Green
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "配置完成！" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n现在可以运行：" -ForegroundColor Green
Write-Host "npm run tauri dev -- --target android" -ForegroundColor Cyan
Write-Host "`n" -ForegroundColor Gray
