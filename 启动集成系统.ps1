# 启动去中心化训练集成系统
# PowerShell版本

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  启动去中心化训练集成系统" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host

# 步骤1: 检查Python环境
Write-Host "步骤1: 检查Python环境..." -ForegroundColor Yellow
$pythonVersion = python --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[错误] 未找到Python环境，请先安装Python 3.8+" -ForegroundColor Red
    Read-Host "按回车键退出"
    exit 1
}
Write-Host "[成功] Python环境正常: $pythonVersion" -ForegroundColor Green
Write-Host

# 步骤2: 启动Python边缘服务器
Write-Host "步骤2: 启动Python边缘服务器..." -ForegroundColor Yellow
Set-Location -Path "$PSScriptRoot\williw-use-master"
Write-Host "[信息] 工作目录: $(Get-Location)" -ForegroundColor Gray

# 检查依赖是否安装
Write-Host "[信息] 检查Python依赖..." -ForegroundColor Gray
try {
    python -c "import flask, flask_cors" 2>$null
    Write-Host "[成功] 依赖检查通过" -ForegroundColor Green
} catch {
    Write-Host "[警告] 缺少依赖包，正在安装..." -ForegroundColor Yellow
    pip install -r requirements.txt
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[错误] 依赖安装失败" -ForegroundColor Red
        Read-Host "按回车键退出"
        exit 1
    }
    Write-Host "[成功] 依赖安装完成" -ForegroundColor Green
}

# 检查端口是否被占用
Write-Host "[信息] 检查端口8080..." -ForegroundColor Gray
$portInUse = Get-NetTCPConnection -LocalPort 8080 -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "[警告] 端口8080已被占用，可能是服务器已在运行" -ForegroundColor Yellow
    $killPort = Read-Host "是否终止占用进程并继续? (y/n)"
    if ($killPort -eq 'y' -or $killPort -eq 'Y') {
        $processId = $portInUse.OwningProcess | Select-Object -First 1
        Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
        Write-Host "[信息] 已终止进程 $processId" -ForegroundColor Gray
        Start-Sleep -Seconds 2
    } else {
        Write-Host "[信息] 跳过端口检查，继续启动..." -ForegroundColor Gray
    }
}

# 启动Python服务器
Write-Host "[信息] 正在启动Python边缘服务器..." -ForegroundColor Gray
Write-Host "[信息] 使用命令: python -m edge_server.api_server --port 8080" -ForegroundColor Gray

# 创建新进程启动Python服务器
$pythonProcess = Start-Process -FilePath "python" `
    -ArgumentList "-m edge_server.api_server --port 8080" `
    -WorkingDirectory (Get-Location) `
    -WindowStyle Normal `
    -PassThru

Write-Host "[信息] Python服务器进程ID: $($pythonProcess.Id)" -ForegroundColor Gray

# 等待服务器启动
Write-Host "[信息] 等待服务器启动..." -ForegroundColor Gray
Start-Sleep -Seconds 5
Write-Host

# 返回主目录
Set-Location -Path $PSScriptRoot

# 步骤3: 启动桌面应用
Write-Host "步骤3: 启动桌面应用..." -ForegroundColor Yellow
Write-Host "[信息] 工作目录: $(Get-Location)" -ForegroundColor Gray

# 检查Node.js环境
Write-Host "[信息] 检查Node.js环境..." -ForegroundColor Gray
$nodeVersion = node --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[错误] 未找到Node.js环境，请先安装Node.js 16+" -ForegroundColor Red
    Read-Host "按回车键退出"
    exit 1
}
Write-Host "[成功] Node.js环境正常: $nodeVersion" -ForegroundColor Green

# 安装依赖（如果需要）
if (-not (Test-Path "node_modules")) {
    Write-Host "[信息] 安装前端依赖..." -ForegroundColor Gray
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[错误] 依赖安装失败" -ForegroundColor Red
        Read-Host "按回车键退出"
        exit 1
    }
    Write-Host "[成功] 依赖安装完成" -ForegroundColor Green
}

# 启动桌面应用
Write-Host "[信息] 正在启动桌面应用..." -ForegroundColor Gray
Write-Host "[信息] 使用命令: npm run tauri dev" -ForegroundColor Gray
Write-Host

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  系统启动完成！" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host
Write-Host "[信息] Python边缘服务器运行在: http://localhost:8080" -ForegroundColor Green
Write-Host "[信息] 桌面应用正在启动..." -ForegroundColor Green
Write-Host
Write-Host "[提示] 首次启动可能需要几分钟时间构建应用" -ForegroundColor Yellow
Write-Host "[提示] 请保持此窗口开启" -ForegroundColor Yellow
Write-Host

# 启动桌面应用
try {
    npm run tauri dev
} finally {
    # 清理：关闭Python服务器
    Write-Host "[信息] 正在关闭Python服务器..." -ForegroundColor Yellow
    Stop-Process -Id $pythonProcess.Id -Force -ErrorAction SilentlyContinue
    Write-Host "[成功] Python服务器已关闭" -ForegroundColor Green
}

Write-Host
Read-Host "按回车键退出"
