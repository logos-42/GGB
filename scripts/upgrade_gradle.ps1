# 升级 Gradle 版本脚本
# 解决 Java 21 和 Gradle 8.0 不兼容问题

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "升级 Gradle" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# 检查当前 Java 版本
Write-Host "`n检查 Java 版本..." -ForegroundColor Yellow
try {
    $javaVersion = java -version 2>&1
    Write-Host $javaVersion[0] -ForegroundColor Gray
} catch {
    Write-Host "❌ 未找到 Java" -ForegroundColor Red
}

# 检查当前 Gradle 版本
Write-Host "`n检查 Gradle 版本..." -ForegroundColor Yellow

$GradleWrapperProps = "src-tauri\gen\android\gradle\wrapper\gradle-wrapper.properties"
if (Test-Path $GradleWrapperProps) {
    $Content = Get-Content $GradleWrapperProps
    $CurrentVersion = ($Content | Select-String "distributionUrl").ToString() -replace '.*gradle-', '' -replace '-bin.zip', ''
    Write-Host "当前版本: $CurrentVersion" -ForegroundColor Gray
} else {
    Write-Host "❌ 未找到 gradle-wrapper.properties" -ForegroundColor Red
    exit 1
}

# 检查需要的版本
Write-Host "`n需要的 Gradle 版本：" -ForegroundColor Yellow
Write-Host "Java 21: 需要 Gradle 8.5+" -ForegroundColor Gray
Write-Host "Java 17: 需要 Gradle 8.0+" -ForegroundColor Gray
Write-Host "Java 11: 需要 Gradle 7.3+" -ForegroundColor Gray

# 升级到 8.5（兼容 Java 21）
Write-Host "`n升级到 Gradle 8.5..." -ForegroundColor Yellow

$NewUrl = "https\://services.gradle.org/distributions/gradle-8.5-bin.zip"
$Content = $Content -replace 'distributionUrl=.*', "distributionUrl=$NewUrl"
Set-Content -Path $GradleWrapperProps -Value $Content

Write-Host "✅ Gradle 版本已更新到 8.5" -ForegroundColor Green

# 验证配置
Write-Host "`n验证配置..." -ForegroundColor Yellow
Get-Content $GradleWrapperProps | Select-String "distributionUrl"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "完成！" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`n现在可以继续构建：" -ForegroundColor Green
Write-Host "npx tauri android dev" -ForegroundColor Cyan

Write-Host "`n或者重新同步 Android Studio 项目：" -ForegroundColor Yellow
Write-Host "File → Sync Project with Gradle Files" -ForegroundColor Gray
