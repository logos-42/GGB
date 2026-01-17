@echo off
REM 清理 Android Studio 配置文件

echo 清理 Android Studio 配置...

IF EXIST "src-tauri\gen\android\.idea" (
    echo 删除损坏的 .idea 目录...
    rmdir /s /q "src-tauri\gen\android\.idea"
    echo 已删除 .idea 目录
) ELSE (
    echo .idea 目录不存在，无需清理
)

echo.
echo Android Studio 下次打开时会自动重新生成配置文件
pause
