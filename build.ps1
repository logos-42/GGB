cd 'd:\AI\去中心化训练'
cargo build 2>&1 | Out-File -FilePath build_output.txt -Encoding UTF8
Get-Content build_output.txt
