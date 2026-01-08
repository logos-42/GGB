cd 'd:\AI\去中心化训练'
$ErrorActionPreference = 'Continue'
$buildOutput = cargo build 2>&1
$buildOutput | Out-File -FilePath build_output.txt -Encoding UTF8

# 统计错误数量
$errors = $buildOutput | Select-String -Pattern 'error\['
$warnings = $buildOutput | Select-String -Pattern 'warning:'

Write-Host "错误数量: $($errors.Count)"
Write-Host "警告数量: $($warnings.Count)"

if ($errors.Count -gt 0) {
    Write-Host "前10个错误:"
    $errors | Select-Object -First 10 | ForEach-Object { Write-Host $_ }
}
