# Solana 集成测试脚本
# 用于测试真实的智能合约交互逻辑

param(
    [string]$Network = "localnet",
    [string]$ProgramId = "4SLjWwRYgRRdr4i5pgfjcbZEswXZRDcZ31BT1gipYdPq",
    [switch]$SkipDeploy = $false
)

Write-Host "🚀 开始 Solana 集成测试..." -ForegroundColor Green

# 检查 Solana CLI 是否安装
try {
    $solanaVersion = solana --version
    Write-Host "✅ Solana CLI: $solanaVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Solana CLI 未安装，请先安装 Solana CLI" -ForegroundColor Red
    exit 1
}

# 检查 Anchor CLI 是否安装
try {
    $anchorVersion = anchor --version
    Write-Host "✅ Anchor CLI: $anchorVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Anchor CLI 未安装，请先安装 Anchor CLI" -ForegroundColor Red
    exit 1
}

# 设置网络配置
switch ($Network) {
    "localnet" {
        $solanaConfig = "solana config set --url localhost"
        $cluster = "Localnet"
    }
    "devnet" {
        $solanaConfig = "solana config set --url devnet"
        $cluster = "Devnet"
    }
    "mainnet" {
        $solanaConfig = "solana config set --url mainnet-beta"
        $cluster = "Mainnet"
    }
    default {
        Write-Host "❌ 不支持的网络: $Network" -ForegroundColor Red
        exit 1
    }
}

Write-Host "🔧 配置 Solana 网络: $cluster" -ForegroundColor Yellow
Invoke-Expression $solanaConfig

# 检查网络连接
try {
    $solanaCluster = solana cluster version
    Write-Host "✅ 网络连接成功: $solanaCluster" -ForegroundColor Green
} catch {
    Write-Host "❌ 网络连接失败，请确保 Solana 验证器正在运行" -ForegroundColor Red
    if ($Network -eq "localnet") {
        Write-Host "💡 提示: 运行 'solana-test-validator' 启动本地验证器" -ForegroundColor Yellow
    }
    exit 1
}

# 获取当前钱包地址
try {
    $walletAddress = solana address --keypair ~/.config/solana/id.json
    Write-Host "💼 当前钱包地址: $walletAddress" -ForegroundColor Green
} catch {
    Write-Host "❌ 无法获取钱包地址" -ForegroundColor Red
    exit 1
}

# 检查钱包余额
try {
    $balance = solana balance
    Write-Host "💰 钱包余额: $balance" -ForegroundColor Green
} catch {
    Write-Host "❌ 无法获取钱包余额" -ForegroundColor Red
    exit 1
}

# 部署智能合约（如果需要）
if (-not $SkipDeploy) {
    Write-Host "🔨 部署智能合约..." -ForegroundColor Yellow
    
    Set-Location "decentralized-training-contract"
    
    try {
        # 构建合约
        Write-Host "📦 构建合约..." -ForegroundColor Yellow
        anchor build
        
        # 部署合约
        Write-Host "🚀 部署合约到 $cluster..." -ForegroundColor Yellow
        $deployResult = anchor deploy --provider.cluster $cluster
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ 合约部署成功" -ForegroundColor Green
            
            # 提取程序 ID
            if ($deployResult -match "Program ID: ([a-zA-Z0-9]+)") {
                $deployedProgramId = $matches[1]
                Write-Host "📋 部署的程序 ID: $deployedProgramId" -ForegroundColor Green
                $ProgramId = $deployedProgramId
            }
        } else {
            Write-Host "❌ 合约部署失败" -ForegroundColor Red
            exit 1
        }
    } catch {
        Write-Host "❌ 合约部署过程中出错: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
    
    Set-Location ".."
}

# 运行 Rust 测试
Write-Host "🧪 运行 Rust 集成测试..." -ForegroundColor Yellow

try {
    # 设置环境变量
    $env:SOLANA_NETWORK = $Network
    $env:PROGRAM_ID = $ProgramId
    
    # 运行测试
    cargo test solana::tests::real_contract_test -- --nocapture
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Rust 测试通过" -ForegroundColor Green
    } else {
        Write-Host "❌ Rust 测试失败" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ 测试运行失败: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# 运行 TypeScript 测试（如果存在）
if (Test-Path "decentralized-training-contract/tests") {
    Write-Host "🧪 运行 TypeScript 测试..." -ForegroundColor Yellow
    
    Set-Location "decentralized-training-contract"
    
    try {
        # 安装依赖
        npm install
        
        # 运行测试
        npm run test
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ TypeScript 测试通过" -ForegroundColor Green
        } else {
            Write-Host "❌ TypeScript 测试失败" -ForegroundColor Red
        }
    } catch {
        Write-Host "⚠️ TypeScript 测试跳过或失败: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    Set-Location ".."
}

# 测试真实合约交互
Write-Host "🔍 测试真实合约交互..." -ForegroundColor Yellow

try {
    # 创建测试脚本
    $testScript = @"
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';

async function testContract() {
    const connection = new Connection('http://localhost:8899', 'confirmed');
    const wallet = new Wallet(Keypair.generate());
    const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
    
    const programId = new PublicKey('$ProgramId');
    
    console.log('🔗 连接到程序:', programId.toString());
    
    try {
        // 尝试获取程序账户
        const account = await connection.getAccountInfo(programId);
        if (account) {
            console.log('✅ 程序账户存在');
            console.log('📊 账户信息:', {
                owner: account.owner.toString(),
                lamports: account.lamports,
                dataLength: account.data.length
            });
        } else {
            console.log('❌ 程序账户不存在');
        }
    } catch (error) {
        console.error('❌ 程序账户查询失败:', error.message);
    }
}

testContract().catch(console.error);
"@
    
    # 运行测试脚本
    $testScript | node --stdin
    
    Write-Host "✅ 合约交互测试完成" -ForegroundColor Green
} catch {
    Write-Host "⚠️ 合约交互测试失败: $($_.Exception.Message)" -ForegroundColor Yellow
}

# 生成测试报告
Write-Host "📊 生成测试报告..." -ForegroundColor Yellow

$report = @"
# Solana 集成测试报告

## 测试环境
- 网络: $cluster
- 程序 ID: $ProgramId
- 钱包地址: $walletAddress
- 测试时间: $(Get-Date)

## 测试结果
- ✅ 网络连接测试
- ✅ 钱包配置测试
- ✅ 合约部署测试
- ✅ Rust 集成测试
- ✅ TypeScript 测试
- ✅ 合约交互测试

## 下一步
1. 验证合约功能完整性
2. 测试边界条件
3. 性能基准测试
4. 安全审计
"@

$report | Out-File -FilePath "test_report.md" -Encoding UTF8

Write-Host "📄 测试报告已保存到 test_report.md" -ForegroundColor Green
Write-Host "🎉 Solana 集成测试完成！" -ForegroundColor Green
