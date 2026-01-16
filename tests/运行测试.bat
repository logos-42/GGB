@echo off
echo ========================================
echo 设备检测功能测试
echo ========================================
echo.

echo 步骤1: 编译验证程序...
cargo build --release --bin verify_detection
echo.

echo 步骤2: 运行设备检测...
target\release\verify_detection.exe
echo.

echo 步骤3: 运行单元测试...
cargo test --test device_detection_test -- --nocapture
echo.

echo ========================================
echo 测试完成！
echo ========================================
echo.
echo 查看 DEVICE_DETECTION_FIX_SUMMARY.md 获取详细信息
echo.
pause
