# 清理 Android Studio 配置文件
# 解决 .idea 文件损坏问题

Write-Host "清理 Android Studio 配置..." -ForegroundColor Cyan

$AndroidIdeaPath = "src-tauri\gen\android\.idea"

if (Test-Path $AndroidIdeaPath) {
    Write-Host "删除损坏的 .idea 目录..." -ForegroundColor Yellow
    Remove-Item -Path $AndroidIdeaPath -Recurse -Force
    Write-Host "✅ 已删除 .idea 目录" -ForegroundColor Green
} else {
    Write-Host ".idea 目录不存在，无需清理" -ForegroundColor Green
}

Write-Host "`nAndroid Studio 下次打开时会自动重新生成配置文件" -ForegroundColor Gray
