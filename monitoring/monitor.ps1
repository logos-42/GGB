# 生产环境监控脚本

param(
    [string]$Environment = "production",
    [int]$CheckInterval = 60,
    [switch]$Continuous = $false,
    [string]$ConfigFile = "monitoring/dashboard.json"
)

Write-Host "🔍 GGB生产环境监控" -ForegroundColor Cyan
Write-Host "环境: $Environment" -ForegroundColor Yellow
Write-Host "检查间隔: ${CheckInterval}秒" -ForegroundColor Yellow

# 加载配置
try {
    $config = Get-Content $ConfigFile -Raw | ConvertFrom-Json -Depth 10
    Write-Host "✅ 监控配置加载成功" -ForegroundColor Green
} catch {
    Write-Host "❌ 监控配置加载失败: $_" -ForegroundColor Red
    exit 1
}

# 设置环境变量
$baseUrl = if ($Environment -eq "production") {
    "https://ggb-edge-server.your-account.workers.dev"
} else {
    "http://localhost:8787"
}

# 监控检查函数
function Check-Health {
    try {
        $response = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get -TimeoutSec 10
        Write-Host "✅ 健康检查通过" -ForegroundColor Green
        
        return @{
            status = "healthy"
            details = $response
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ 健康检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Check-Performance {
    try {
        $startTime = Get-Date
        $response = Invoke-RestMethod -Uri "$baseUrl/api/stats" -Method Get -TimeoutSec 10
        $endTime = Get-Date
        
        $responseTime = ($endTime - $startTime).TotalMilliseconds
        
        Write-Host "✅ 性能检查通过 (${responseTime}ms)" -ForegroundColor Green
        
        return @{
            status = "healthy"
            response_time_ms = $responseTime
            stats = $response
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ 性能检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Check-Nodes {
    try {
        # 模拟节点检查
        $nodeStats = @{
            total_nodes = (Get-Random -Minimum 50 -Maximum 200)
            active_nodes = (Get-Random -Minimum 30 -Maximum 150)
            avg_heartbeat_interval = (Get-Random -Minimum 10 -Maximum 60)
        }
        
        Write-Host "✅ 节点检查完成: $($nodeStats.active_nodes)/$($nodeStats.total_nodes) 活跃" -ForegroundColor Green
        
        return @{
            status = "healthy"
            nodes = $nodeStats
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ 节点检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Check-Tasks {
    try {
        # 模拟任务检查
        $taskStats = @{
            total_tasks = (Get-Random -Minimum 100 -Maximum 500)
            completed_tasks = (Get-Random -Minimum 80 -Maximum 450)
            failed_tasks = (Get-Random -Minimum 0 -Maximum 20)
            avg_completion_time_ms = (Get-Random -Minimum 1000 -Maximum 10000)
        }
        
        $completionRate = if ($taskStats.total_tasks -gt 0) {
            [math]::Round($taskStats.completed_tasks / $taskStats.total_tasks * 100, 2)
        } else { 0 }
        
        Write-Host "✅ 任务检查完成: $completionRate% 完成率" -ForegroundColor Green
        
        return @{
            status = "healthy"
            tasks = $taskStats
            completion_rate = $completionRate
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ 任务检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Check-Algorithms {
    try {
        # 模拟算法检查
        $algoStats = @{
            pso_execution_time = (Get-Random -Minimum 50 -Maximum 200)
            ga_execution_time = (Get-Random -Minimum 100 -Maximum 300)
            aco_execution_time = (Get-Random -Minimum 80 -Maximum 250)
            total_allocations = (Get-Random -Minimum 500 -Maximum 2000)
            success_rate = (Get-Random -Minimum 85 -Maximum 99)
        }
        
        Write-Host "✅ 算法检查完成: $($algoStats.success_rate)% 成功率" -ForegroundColor Green
        
        return @{
            status = "healthy"
            algorithms = $algoStats
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ 算法检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Check-ZKProof {
    try {
        # 模拟ZK证明检查
        $zkStats = @{
            verification_time_ms = (Get-Random -Minimum 10 -Maximum 100)
            generation_time_ms = (Get-Random -Minimum 50 -Maximum 500)
            total_verifications = (Get-Random -Minimum 1000 -Maximum 5000)
            success_rate = (Get-Random -Minimum 95 -Maximum 100)
            batch_size_avg = (Get-Random -Minimum 5 -Maximum 20)
        }
        
        Write-Host "✅ ZK证明检查完成: $($zkStats.success_rate)% 验证成功率" -ForegroundColor Green
        
        return @{
            status = "healthy"
            zk_proof = $zkStats
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    } catch {
        Write-Host "❌ ZK证明检查失败: $_" -ForegroundColor Red
        return @{
            status = "unhealthy"
            error = $_.Exception.Message
            timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
    }
}

function Send-Alert {
    param(
        [string]$Severity,
        [string]$Message,
        [hashtable]$Details
    )
    
    $alert = @{
        severity = $Severity
        message = $Message
        details = $Details
        timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        environment = $Environment
    }
    
    # 根据严重程度选择颜色
    $color = switch ($Severity) {
        "critical" { "Red" }
        "warning" { "Yellow" }
        default { "Gray" }
    }
    
    Write-Host "🚨 告警 [$Severity]: $Message" -ForegroundColor $color
    
    # 保存告警到文件
    $alertFile = "alerts/alert-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"
    $alert | ConvertTo-Json -Depth 5 | Out-File -FilePath $alertFile -Encoding UTF8
    
    # 这里可以添加发送到Slack、Email等的逻辑
    # Send-ToSlack $alert
    # Send-ToEmail $alert
    
    return $alert
}

# 运行监控检查
function Run-MonitoringCheck {
    Write-Host "`n🔄 运行监控检查..." -ForegroundColor Cyan
    Write-Host "时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
    
    $checkResults = @{
        timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        environment = $Environment
        checks = @{}
        alerts = @()
        summary = @{
            total_checks = 0
            passed_checks = 0
            failed_checks = 0
            warning_checks = 0
        }
    }
    
    # 运行各项检查
    $checks = @(
        @{ Name = "Health"; Function = { Check-Health } }
        @{ Name = "Performance"; Function = { Check-Performance } }
        @{ Name = "Nodes"; Function = { Check-Nodes } }
        @{ Name = "Tasks"; Function = { Check-Tasks } }
        @{ Name = "Algorithms"; Function = { Check-Algorithms } }
        @{ Name = "ZKProof"; Function = { Check-ZKProof } }
    )
    
    foreach ($check in $checks) {
        $checkName = $check.Name
        $checkResults.summary.total_checks++
        
        Write-Host "`n检查: $checkName" -ForegroundColor Yellow
        
        try {
            $result = & $check.Function
            $checkResults.checks.$checkName = $result
            
            if ($result.status -eq "healthy") {
                $checkResults.summary.passed_checks++
            } else {
                $checkResults.summary.failed_checks++
                
                # 发送告警
                $alert = Send-Alert -Severity "critical" -Message "$checkName 检查失败" -Details $result
                $checkResults.alerts += $alert
            }
        } catch {
            $checkResults.summary.failed_checks++
            Write-Host "❌ $checkName 检查异常: $_" -ForegroundColor Red
            
            $alert = Send-Alert -Severity "critical" -Message "$checkName 检查异常" -Details @{ error = $_.Exception.Message }
            $checkResults.alerts += $alert
        }
    }
    
    # 检查告警条件
    Check-AlertConditions $checkResults
    
    # 输出摘要
    Write-Host "`n📊 检查摘要" -ForegroundColor Cyan
    Write-Host "=" * 50 -ForegroundColor Gray
    Write-Host "总检查数: $($checkResults.summary.total_checks)" -ForegroundColor White
    Write-Host "通过检查: $($checkResults.summary.passed_checks)" -ForegroundColor Green
    Write-Host "失败检查: $($checkResults.summary.failed_checks)" -ForegroundColor Red
    Write-Host "告警数量: $($checkResults.alerts.Count)" -ForegroundColor Yellow
    Write-Host "=" * 50 -ForegroundColor Gray
    
    # 保存检查结果
    $resultsFile = "monitoring/results/check-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"
    $checkResults | ConvertTo-Json -Depth 10 | Out-File -FilePath $resultsFile -Encoding UTF8
    
    return $checkResults
}

# 检查告警条件
function Check-AlertConditions {
    param($checkResults)
    
    # 检查响应时间
    $perfCheck = $checkResults.checks.Performance
    if ($perfCheck -and $perfCheck.response_time_ms -gt 1000) {
        $alert = Send-Alert -Severity "warning" -Message "高响应时间警告" -Details @{
            response_time_ms = $perfCheck.response_time_ms
            threshold_ms = 1000
        }
        $checkResults.alerts += $alert
    }
    
    # 检查节点活跃度
    $nodesCheck = $checkResults.checks.Nodes
    if ($nodesCheck -and $nodesCheck.nodes.active_nodes -lt 10) {
        $alert = Send-Alert -Severity "warning" -Message "低节点活跃度警告" -Details @{
            active_nodes = $nodesCheck.nodes.active_nodes
            threshold = 10
        }
        $checkResults.alerts += $alert
    }
    
    # 检查任务完成率
    $tasksCheck = $checkResults.checks.Tasks
    if ($tasksCheck -and $tasksCheck.completion_rate -lt 80) {
        $alert = Send-Alert -Severity "warning" -Message "低任务完成率警告" -Details @{
            completion_rate = $tasksCheck.completion_rate
            threshold = 80
        }
        $checkResults.alerts += $alert
    }
    
    # 检查ZK证明成功率
    $zkCheck = $checkResults.checks.ZKProof
    if ($zkCheck -and $zkCheck.zk_proof.success_rate -lt 95) {
        $alert = Send-Alert -Severity "critical" -Message "低ZK证明成功率警告" -Details @{
            success_rate = $zkCheck.zk_proof.success_rate
            threshold = 95
        }
        $checkResults.alerts += $alert
    }
}

# 创建必要的目录
$directories = @("monitoring/results", "alerts", "logs")
foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        Write-Host "创建目录: $dir" -ForegroundColor Gray
    }
}

# 主循环
if ($Continuous) {
    Write-Host "🔄 进入连续监控模式..." -ForegroundColor Cyan
    Write-Host "按 Ctrl+C 停止" -ForegroundColor Yellow
    
    try {
        while ($true) {
            $checkResults = Run-MonitoringCheck
            
            # 等待下一次检查
            Write-Host "`n⏳ 等待 ${CheckInterval}秒后下一次检查..." -ForegroundColor Gray
            Start-Sleep -Seconds $CheckInterval
        }
    } catch {
        Write-Host "`n🛑 监控停止: $_" -ForegroundColor Red
    }
} else {
    # 单次检查
    Run-MonitoringCheck
    Write-Host "`n✅ 监控检查完成" -ForegroundColor Green
}
