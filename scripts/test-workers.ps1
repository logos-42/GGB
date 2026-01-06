# Cloudflare Workers测试脚本

param(
    [int]$NodeCount = 3,
    [int]$Duration = 60,
    [string]$BaseUrl = "http://localhost:8787"
)

Write-Host "🧪 测试Cloudflare Workers..." -ForegroundColor Cyan
Write-Host "节点数量: $NodeCount" -ForegroundColor Yellow
Write-Host "测试时长: ${Duration}秒" -ForegroundColor Yellow
Write-Host "基础URL: $BaseUrl" -ForegroundColor Yellow

# 检查本地服务器是否运行
try {
    $healthResponse = Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get -ErrorAction Stop
    Write-Host "✅ 本地服务器运行正常" -ForegroundColor Green
} catch {
    Write-Host "❌ 本地服务器未运行，请先运行: wrangler dev" -ForegroundColor Red
    exit 1
}

# 测试数据
$testNodes = @()
for ($i = 0; $i -lt $NodeCount; $i++) {
    $testNodes += @{
        node_id = "test-node-$i"
        capabilities = @{
            cpu_cores = 4
            memory_mb = 8192
            network_type = "wifi"
            battery_level = 0.8
        }
        network_info = @{
            latency_ms = 50
            bandwidth_mbps = 100
            ip_address = "192.168.1.$($i + 100)"
        }
        location = @{
            latitude = 40.7128 + (Get-Random -Minimum -0.1 -Maximum 0.1)
            longitude = -74.0060 + (Get-Random -Minimum -0.1 -Maximum 0.1)
        }
        available = $true
        timestamp = [DateTimeOffset]::Now.ToUnixTimeSeconds()
    }
}

$testTasks = @(
    @{
        task_id = "training-task-1"
        task_type = "Training"
        input_data = [System.Text.Encoding]::UTF8.GetBytes("训练数据样本")
        requirements = @{
            min_cpu_cores = 2
            min_memory_mb = 4096
            max_latency_ms = 100
            require_gpu = $false
        }
    },
    @{
        task_id = "inference-task-1"
        task_type = "Inference"
        input_data = [System.Text.Encoding]::UTF8.GetBytes("推理数据样本")
        requirements = @{
            min_cpu_cores = 1
            min_memory_mb = 2048
            max_latency_ms = 50
            require_gpu = $false
        }
    }
)

# 测试函数
function Test-NodeRegistration {
    param($Node)
    
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/api/nodes/register" -Method Post `
            -Body ($Node | ConvertTo-Json -Depth 5) `
            -ContentType "application/json" `
            -ErrorAction Stop
        
        Write-Host "✅ 节点注册成功: $($Node.node_id)" -ForegroundColor Green
        return $response
    } catch {
        Write-Host "❌ 节点注册失败: $($Node.node_id) - $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-TaskSubmission {
    param($Task)
    
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/api/tasks/submit" -Method Post `
            -Body ($Task | ConvertTo-Json -Depth 5) `
            -ContentType "application/json" `
            -ErrorAction Stop
        
        Write-Host "✅ 任务提交成功: $($Task.task_id)" -ForegroundColor Green
        return $response
    } catch {
        Write-Host "❌ 任务提交失败: $($Task.task_id) - $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-TaskMatching {
    param($Task)
    
    $matchRequest = @{
        task = $Task
        strategy = "Hybrid"
    }
    
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/api/tasks/match" -Method Post `
            -Body ($matchRequest | ConvertTo-Json -Depth 5) `
            -ContentType "application/json" `
            -ErrorAction Stop
        
        Write-Host "✅ 任务匹配成功: $($Task.task_id)" -ForegroundColor Green
        Write-Host "   匹配到 $($response.Count) 个节点" -ForegroundColor Gray
        return $response
    } catch {
        Write-Host "❌ 任务匹配失败: $($Task.task_id) - $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-AlgorithmAllocation {
    param($Task, $Nodes)
    
    $algorithmRequest = @{
        task_id = $Task.task_id
        task_type = $Task.task_type
        available_nodes = $Nodes
        requirements = $Task.requirements
        algorithm_type = "Hybrid"
        parameters = @{
            max_iterations = 50
            convergence_threshold = 0.0001
        }
    }
    
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/api/algorithms/allocate" -Method Post `
            -Body ($algorithmRequest | ConvertTo-Json -Depth 5) `
            -ContentType "application/json" `
            -ErrorAction Stop
        
        Write-Host "✅ 算法分配成功: $($Task.task_id)" -ForegroundColor Green
        Write-Host "   分配了 $($response.allocation.assigned_nodes.Count) 个节点" -ForegroundColor Gray
        return $response
    } catch {
        Write-Host "❌ 算法分配失败: $($Task.task_id) - $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-HealthCheck {
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get -ErrorAction Stop
        Write-Host "✅ 健康检查通过" -ForegroundColor Green
        return $response
    } catch {
        Write-Host "❌ 健康检查失败: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-Stats {
    try {
        $response = Invoke-RestMethod -Uri "$BaseUrl/api/stats" -Method Get -ErrorAction Stop
        Write-Host "✅ 统计信息获取成功" -ForegroundColor Green
        return $response
    } catch {
        Write-Host "❌ 统计信息获取失败: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

# 运行测试
Write-Host "`n📊 开始测试..." -ForegroundColor Cyan

$testResults = @{
    start_time = Get-Date
    tests = @()
    successes = 0
    failures = 0
}

# 1. 健康检查
Write-Host "`n1. 健康检查测试" -ForegroundColor Yellow
$healthResult = Test-HealthCheck
if ($healthResult) {
    $testResults.successes++
    $testResults.tests += @{
        name = "健康检查"
        status = "成功"
        result = $healthResult
    }
} else {
    $testResults.failures++
    $testResults.tests += @{
        name = "健康检查"
        status = "失败"
        result = $null
    }
}

# 2. 节点注册
Write-Host "`n2. 节点注册测试" -ForegroundColor Yellow
$registeredNodes = @()
foreach ($node in $testNodes) {
    $result = Test-NodeRegistration $node
    if ($result) {
        $registeredNodes += $node
        $testResults.successes++
    } else {
        $testResults.failures++
    }
    
    Start-Sleep -Milliseconds 100
}

# 3. 任务提交和匹配
Write-Host "`n3. 任务提交和匹配测试" -ForegroundColor Yellow
foreach ($task in $testTasks) {
    # 提交任务
    $taskResult = Test-TaskSubmission $task
    if ($taskResult) {
        $testResults.successes++
    } else {
        $testResults.failures++
    }
    
    # 任务匹配
    $matchResult = Test-TaskMatching $task
    if ($matchResult) {
        $testResults.successes++
    } else {
        $testResults.failures++
    }
    
    Start-Sleep -Milliseconds 200
}

# 4. 算法分配
Write-Host "`n4. 算法分配测试" -ForegroundColor Yellow
foreach ($task in $testTasks) {
    $allocationResult = Test-AlgorithmAllocation $task $registeredNodes
    if ($allocationResult) {
        $testResults.successes++
    } else {
        $testResults.failures++
    }
    
    Start-Sleep -Milliseconds 300
}

# 5. 统计信息
Write-Host "`n5. 统计信息测试" -ForegroundColor Yellow
$statsResult = Test-Stats
if ($statsResult) {
    $testResults.successes++
    $testResults.tests += @{
        name = "统计信息"
        status = "成功"
        result = $statsResult
    }
} else {
    $testResults.failures++
    $testResults.tests += @{
        name = "统计信息"
        status = "失败"
        result = $null
    }
}

# 6. 性能测试
Write-Host "`n6. 性能测试" -ForegroundColor Yellow
$performanceResults = @()
$startTime = Get-Date

for ($i = 0; $i -lt 10; $i++) {
    $testStart = Get-Date
    $null = Test-HealthCheck
    $testEnd = Get-Date
    
    $duration = ($testEnd - $testStart).TotalMilliseconds
    $performanceResults += $duration
    
    Write-Host "   请求 $($i+1): $duration ms" -ForegroundColor Gray
    Start-Sleep -Milliseconds 100
}

$avgResponseTime = ($performanceResults | Measure-Object -Average).Average
Write-Host "   平均响应时间: $avgResponseTime ms" -ForegroundColor Green

# 输出测试结果
$testResults.end_time = Get-Date
$testResults.duration = ($testResults.end_time - $testResults.start_time).TotalSeconds
$testResults.avg_response_time = $avgResponseTime
$testResults.success_rate = if (($testResults.successes + $testResults.failures) -gt 0) {
    [math]::Round($testResults.successes / ($testResults.successes + $testResults.failures) * 100, 2)
} else { 0 }

Write-Host "`n📈 测试结果摘要" -ForegroundColor Cyan
Write-Host "=" * 50 -ForegroundColor Gray
Write-Host "测试开始时间: $($testResults.start_time)" -ForegroundColor White
Write-Host "测试结束时间: $($testResults.end_time)" -ForegroundColor White
Write-Host "测试总时长: $($testResults.duration) 秒" -ForegroundColor White
Write-Host "成功测试数: $($testResults.successes)" -ForegroundColor Green
Write-Host "失败测试数: $($testResults.failures)" -ForegroundColor Red
Write-Host "成功率: $($testResults.success_rate)%" -ForegroundColor Yellow
Write-Host "平均响应时间: $($testResults.avg_response_time) ms" -ForegroundColor Yellow
Write-Host "=" * 50 -ForegroundColor Gray

# 保存测试结果
$resultsFile = "test-results-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"
$testResults | ConvertTo-Json -Depth 5 | Out-File -FilePath $resultsFile -Encoding UTF8
Write-Host "测试结果已保存到: $resultsFile" -ForegroundColor Green

# 判断测试是否通过
if ($testResults.success_rate -ge 80) {
    Write-Host "✅ 测试通过！" -ForegroundColor Green
    exit 0
} else {
    Write-Host "❌ 测试失败，成功率低于80%" -ForegroundColor Red
    exit 1
}
