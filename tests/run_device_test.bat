@echo off
chcp 65001 >nul
echo ========================================
echo 设备检测功能测试
echo ========================================
echo.

echo 步骤1: 编译验证程序...
cargo build --release --bin verify_detection
if %errorlevel% neq 0 (
    echo ❌ 编译失败！
    pause
    exit /b 1
)

echo.
echo 步骤2: 运行设备检测...
echo.
start /wait target\release\verify_detection.exe

echo.
echo 步骤3: 运行单元测试...
cargo test --test device_detection_test -- --nocapture

echo.
echo ========================================
echo 测试完成！
echo ========================================
echo.
echo 建议手动验证：
echo 1. 验证内存: systeminfo ^| findstr "物理内存"
echo 2. 验证GPU: wmic path win32_VideoController get name
echo 3. 验证CPU: wmic cpu get NumberOfCores
pause
