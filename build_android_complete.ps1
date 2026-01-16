# 完整的Android构建脚本
param(
    [switch]$Debug,
    [switch]$Release
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "完整Android构建脚本" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$buildMode = if ($Release) { "--release" } else { "" }
$profileName = if ($Release) { "release" } else { "debug" }

Write-Host "构建模式: $profileName" -ForegroundColor Yellow

# 1. 清理之前的构建
Write-Host "清理之前的构建..." -ForegroundColor Yellow
Remove-Item "src-tauri\gen\android\app\src\main\jniLibs" -Recurse -Force -ErrorAction SilentlyContinue

# 2. 构建主库
Write-Host "构建主库..." -ForegroundColor Yellow
& cargo build --features android $buildMode

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 主库构建失败" -ForegroundColor Red
    exit 1
}

# 3. 构建JNI库
Write-Host "构建JNI库..." -ForegroundColor Yellow
& cargo build --features android --lib williw_jni $buildMode

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ JNI库构建失败" -ForegroundColor Red
    exit 1
}

# 4. 构建Android目标库
Write-Host "构建Android目标库..." -ForegroundColor Yellow

$androidTargets = @(
    @{rust = "aarch64-linux-android"; android = "arm64-v8a"},
    @{rust = "armv7-linux-androideabi"; android = "armeabi-v7a"},
    @{rust = "i686-linux-android"; android = "x86"},
    @{rust = "x86_64-linux-android"; android = "x86_64"}
)

$outputDir = "src-tauri\gen\android\app\src\main\jniLibs"
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

foreach ($target in $androidTargets) {
    $rustTarget = $target.rust
    $androidArch = $target.android
    
    Write-Host "构建 $rustTarget..." -ForegroundColor Gray
    
    & cargo build --target $rustTarget --features android $buildMode
    
    if ($LASTEXITCODE -eq 0) {
        $srcPath = "target\$rustTarget\$profileName\libwilliw.so"
        $dstDir = "$outputDir\$androidArch"
        $dstPath = "$dstDir\libwilliw.so"
        
        if (Test-Path $srcPath) {
            New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
            Copy-Item $srcPath $dstPath -Force
            Write-Host "✓ $rustTarget -> $androidArch" -ForegroundColor Green
        } else {
            Write-Host "⚠ $rustTarget 构建文件未找到" -ForegroundColor Yellow
        }
    } else {
        Write-Host "✗ $rustTarget 构建失败" -ForegroundColor Red
    }
}

# 5. 构建JNI Android目标
Write-Host "构建JNI Android目标..." -ForegroundColor Yellow

$jniTargets = @(
    @{rust = "aarch64-linux-android"; android = "arm64-v8a"},
    @{rust = "armv7-linux-androideabi"; android = "armeabi-v7a"},
    @{rust = "i686-linux-android"; android = "x86"},
    @{rust = "x86_64-linux-android"; android = "x86_64"}
)

foreach ($target in $jniTargets) {
    $rustTarget = $target.rust
    $androidArch = $target.android
    
    Write-Host "构建JNI $rustTarget..." -ForegroundColor Gray
    
    & cargo build --target $rustTarget --features android --lib williw_jni $buildMode
    
    if ($LASTEXITCODE -eq 0) {
        $srcPath = "target\$rustTarget\$profileName\libwilliw_jni.so"
        $dstDir = "$outputDir\$androidArch"
        $dstPath = "$dstDir\libwilliw_jni.so"
        
        if (Test-Path $srcPath) {
            New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
            Copy-Item $srcPath $dstPath -Force
            Write-Host "✓ JNI $rustTarget -> $androidArch" -ForegroundColor Green
        } else {
            Write-Host "⚠ JNI $rustTarget 构建文件未找到" -ForegroundColor Yellow
        }
    } else {
        Write-Host "✗ JNI $rustTarget 构建失败" -ForegroundColor Red
    }
}

# 6. 构建Android应用
Write-Host "构建Android应用..." -ForegroundColor Yellow

try {
    if ($Release) {
        Write-Host "构建Release版本..." -ForegroundColor Gray
        Set-Location "src-tauri\gen\android"
        & .\gradlew assembleRelease
    } else {
        Write-Host "构建Debug版本..." -ForegroundColor Gray
        Set-Location "src-tauri\gen\android"
        & .\gradlew assembleDebug
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Android应用构建成功" -ForegroundColor Green
    } else {
        Write-Host "❌ Android应用构建失败" -ForegroundColor Red
    }
} catch {
    Write-Host "⚠ Gradle构建失败，请检查Android环境" -ForegroundColor Yellow
}

# 7. 显示结果
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "构建完成！" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($Release) {
    Write-Host "Release APK位置:" -ForegroundColor Yellow
    Write-Host "  src-tauri\gen\android\app\build\outputs\apk\release\" -ForegroundColor Gray
    Write-Host "  src-tauri\gen\android\app\build\outputs\bundle\release\" -ForegroundColor Gray
} else {
    Write-Host "Debug APK位置:" -ForegroundColor Yellow
    Write-Host "  src-tauri\gen\android\app\build\outputs\apk\debug\" -ForegroundColor Gray
}

Write-Host ""
Write-Host "库文件位置:" -ForegroundColor Yellow
Write-Host "  $outputDir" -ForegroundColor Gray
Write-Host ""

Write-Host "下一步操作:" -ForegroundColor Yellow
Write-Host "1. 连接Android设备" -ForegroundColor Gray
Write-Host "2. 安装APK: adb install app-debug.apk" -ForegroundColor Gray
Write-Host "3. 启动应用: adb shell am start -n com.williw.mobile/.MainActivity" -ForegroundColor Gray
Write-Host "4. 查看日志: adb logcat | grep williw" -ForegroundColor Gray
