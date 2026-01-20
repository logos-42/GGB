@echo off
chcp 65001 >nul
echo ============================================
echo  启动去中心化训练集成系统
echo ============================================
echo.

:: 检查Python环境
echo 步骤1: 检查Python环境...
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到Python环境，请先安装Python 3.8+
    pause
    exit /b 1
)
echo [成功] Python环境正常
echo.

:: 启动Python边缘服务器
echo 步骤2: 启动Python边缘服务器...
cd williw-use-master
echo [信息] 工作目录: %CD%

:: 检查依赖是否安装
echo [信息] 检查Python依赖...
python -c "import flask, flask_cors" >nul 2>&1
if %errorlevel% neq 0 (
    echo [警告] 缺少依赖包，正在安装...
    pip install -r requirements.txt
    if %errorlevel% neq 0 (
        echo [错误] 依赖安装失败
        pause
        exit /b 1
    )
    echo [成功] 依赖安装完成
)

:: 检查端口是否被占用
echo [信息] 检查端口8080...
netstat -ano | findstr "8080" >nul
if %errorlevel% equ 0 (
    echo [警告] 端口8080已被占用，可能是服务器已在运行
    set /p killPort=是否终止占用进程并继续? (y/n): 
    if /i "%killPort%"=="y" (
        for /f "tokens=5" %%p in ('netstat -ano ^| findstr "8080"') do (
            taskkill /PID %%p /F >nul 2>&1
        )
        timeout /t 2 /nobreak >nul
    ) else (
        echo [信息] 跳过端口检查，继续启动...
    )
)

:: 启动Python服务器
echo [信息] 正在启动Python边缘服务器...
echo [信息] 使用命令: python -m edge_server.api_server --port 8080
start "Python边缘服务器" python -m edge_server.api_server --port 8080

:: 等待服务器启动
echo [信息] 等待服务器启动...
timeout /t 5 /nobreak >nul
echo.

:: 返回主目录
cd ..

:: 启动桌面应用
echo 步骤3: 启动桌面应用...
echo [信息] 工作目录: %CD%

:: 检查Node.js环境
echo [信息] 检查Node.js环境...
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到Node.js环境，请先安装Node.js 16+
    pause
    exit /b 1
)
echo [成功] Node.js环境正常

:: 安装依赖（如果需要）
if not exist "node_modules" (
    echo [信息] 安装前端依赖...
    npm install
    if %errorlevel% neq 0 (
        echo [错误] 依赖安装失败
        pause
        exit /b 1
    )
    echo [成功] 依赖安装完成
)

:: 启动桌面应用
echo [信息] 正在启动桌面应用...
echo [信息] 使用命令: npm run tauri dev
echo.
echo ============================================
echo  系统启动完成！
echo ============================================
echo.
echo [信息] Python边缘服务器运行在: http://localhost:8080
echo [信息] 桌面应用正在启动...
echo.
echo [提示] 首次启动可能需要几分钟时间构建应用
echo [提示] 请保持此窗口开启
echo.

npm run tauri dev

pause
