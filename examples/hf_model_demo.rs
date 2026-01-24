//! Hugging Face Llama 3.2 1B 模型加载和拆分演示

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use williw::training::{create_llama_32_1b_loader, ModelPartition};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("开始 Hugging Face Llama 3.2 1B 模型演示");

    // 设置缓存目录
    let cache_dir = PathBuf::from("./models/huggingface");
    
    // 创建模型加载器
    info!("创建 Llama 3.2 1B 模型加载器...");
    let loader = create_llama_32_1b_loader(Some(cache_dir))?;
    
    // 检查是否已安装必要的 Python 依赖
    info!("检查 Python 依赖...");
    check_python_dependencies().await?;
    
    // 加载并拆分模型
    info!("开始加载和拆分模型...");
    let partitions = loader.load_and_split().await?;
    
    // 显示拆分结果
    info!("模型拆分完成!");
    for partition in &partitions {
        info!(
            "分区 {}: {} 层, {} 参数",
            partition.part_id,
            partition.layers.len(),
            partition.total_params
        );
        
        // 显示前几层的详细信息
        for (i, layer) in partition.layers.iter().take(3).enumerate() {
            info!(
                "  层 {}: {} (类型: {}, 形状: {:?})",
                i, layer.name, layer.layer_type, layer.shape
            );
        }
        
        if partition.layers.len() > 3 {
            info!("  ... 还有 {} 层", partition.layers.len() - 3);
        }
    }
    
    info!("演示完成! 模型已成功拆分为 {} 部分", partitions.len());
    
    Ok(())
}

/// 检查必要的 Python 依赖
async fn check_python_dependencies() -> Result<()> {
    use tokio::process::Command;
    
    // 检查 Python 是否可用
    let python_check = Command::new("python")
        .arg("--version")
        .output()
        .await;
    
    match python_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("Python 版本: {}", version.trim());
        }
        _ => {
            return Err(anyhow::anyhow!("Python 未安装或不可用"));
        }
    }
    
    // 检查必要的 Python 包
    let required_packages = vec![
        "torch",
        "numpy", 
        "huggingface_hub",
        "safetensors"
    ];
    
    for package in required_packages {
        let check = Command::new("python")
            .arg("-c")
            .arg(&format!("import {}", package))
            .output()
            .await;
        
        match check {
            Ok(output) if output.status.success() => {
                info!("✓ {} 已安装", package);
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "缺少 Python 包: {}. 请运行: pip install torch numpy huggingface_hub safetensors",
                    package
                ));
            }
        }
    }
    
    Ok(())
}
