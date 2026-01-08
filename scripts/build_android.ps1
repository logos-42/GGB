# Android 构建脚本 (PowerShell)
# 构建 Rust 库并集成到 Android 项目

$ErrorActionPreference = "Stop"

$PROJECT_ROOT = Split-Path -Parent $PSScriptRoot
$ANDROID_DIR = Join-Path $PROJECT_ROOT "android"
$TARGET_DIR = Join-Path $PROJECT_ROOT "target"

$ANDROID_ABIS = @(
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "i686-linux-android",
    "x86_64-linux-android"
)

$ANDROID_ABI_NAMES = @(
    "arm64-v8a",
    "armeabi-v7a",
    "x86",
    "x86_64"
)

# 检查 Rust 工具链
Write-Host "检查 Rust Android 工具链..."
foreach ($target in $ANDROID_ABIS) {
    $installed = rustup target list --installed | Select-String $target
    if (-not $installed) {
        Write-Host "安装 $target 工具链..."
        rustup target add $target
    }
}

# 构建 Rust 库
Write-Host "构建 Rust 库..."
Set-Location $PROJECT_ROOT

for ($i = 0; $i -lt $ANDROID_ABIS.Length; $i++) {
    $target = $ANDROID_ABIS[$i]
    $abi_name = $ANDROID_ABI_NAMES[$i]
    
    Write-Host "构建 $target..."
    cargo build --target $target --release --features ffi
    
    # 复制库文件到 Android 项目
    $jniLibsDir = Join-Path $ANDROID_DIR "src\main\jniLibs\$abi_name"
    New-Item -ItemType Directory -Force -Path $jniLibsDir | Out-Null
    
    # 查找生成的库文件
    $libPath1 = Join-Path $TARGET_DIR "$target\release\libwilliw.so"
    $libPath2 = Join-Path $TARGET_DIR "$target\release\libWilliw.so"
    $destPath = Join-Path $jniLibsDir "libwilliw.so"
    
    if (Test-Path $libPath1) {
        Copy-Item $libPath1 $destPath -Force
        Write-Host "已复制 libggb.so 到 $abi_name"
    } elseif (Test-Path $libPath2) {
        Copy-Item $libPath2 $destPath -Force
        Write-Host "已复制 libGGB.so 到 $abi_name (重命名为 libggb.so)"
    } else {
        Write-Warning "未找到 $target 的库文件"
    }
}

Write-Host "Android 构建完成！"
Write-Host "库文件位置: $ANDROID_DIR\src\main\jniLibs\"

