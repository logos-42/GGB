/**
 * 使用真实下载的模型数据创建元数据并上传到 HF
 * 目标仓库: https://huggingface.co/logos42/williw
 */
use metadata_uploader::{MetadataUploader, UploadConfig};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始使用真实模型数据创建并上传元数据");
    
    // ==================== 创建真实模型元数据 ====================
    println!("\n📊 基于已下载的模型创建元数据...");
    
    // 基于实际 config.json 的模型信息
    let model_metadata = serde_json::json!({
        "model_info": {
            "model_name": "LiquidAI/LFM2.5-1.2B-Thinking",
            "model_type": "lfm2",
            "architecture": "Lfm2ForCausalLM",
            "hidden_size": 2048,
            "intermediate_size": 12288,
            "num_attention_heads": 32,
            "num_key_value_heads": 8,
            "num_hidden_layers": 16,
            "vocab_size": 65536,
            "max_position_embeddings": 128000,
            "dtype": "bfloat16",
            "layer_types": [
                "conv", "conv", "full_attention", "conv", "conv", "full_attention",
                "conv", "conv", "full_attention", "conv", "full_attention",
                "conv", "full_attention", "conv", "full_attention", "conv"
            ],
            "file_size_gb": 2.34,
            "estimated_params": 1240000000,
            "download_path": "./test_models/models--LiquidAI--LFM2.5-1.2B-Thinking/snapshots/3c58ec1db4a336594e9ac4ad3fff10fc8aa22d70"
        },
        "split_plan": {
            "strategy": "layer_based_balanced",
            "num_nodes": 2,
            "total_params": 1240000000,
            "created_at": "2026-01-25T08:50:00Z",
            "splits": [
                {
                    "node_id": "node_001",
                    "layer_range": "0-7",
                    "layer_types": ["conv", "conv", "full_attention", "conv", "conv", "full_attention", "conv", "conv"],
                    "num_layers": 8,
                    "estimated_params": 620000000,
                    "estimated_size_gb": 1.17,
                    "compute_intensity": "medium",
                    "memory_requirement_mb": 1200,
                    "description": "前8层：包含4个卷积层和2个注意力层"
                },
                {
                    "node_id": "node_002", 
                    "layer_range": "8-15",
                    "layer_types": ["full_attention", "conv", "full_attention", "conv", "full_attention", "conv", "full_attention", "conv"],
                    "num_layers": 8,
                    "estimated_params": 620000000,
                    "estimated_size_gb": 1.17,
                    "compute_intensity": "high",
                    "memory_requirement_mb": 1200,
                    "description": "后8层：包含4个注意力层和4个卷积层"
                }
            ]
        },
        "distribution_config": {
            "load_balancing": "equal_split",
            "fault_tolerance": true,
            "compression": "none",
            "encryption": "optional",
            "sync_protocol": "p2p",
            "heartbeat_interval_ms": 5000
        },
        "training_config": {
            "batch_size_per_node": 32,
            "learning_rate": 1e-4,
            "optimizer": "adamw",
            "scheduler": "cosine",
            "max_epochs": 100,
            "gradient_accumulation_steps": 4,
            "mixed_precision": true
        },
        "metadata_info": {
            "version": "1.0.0",
            "created_by": "williw_model_splitter",
            "created_at": "2026-01-25T08:50:00Z",
            "purpose": "decentralized_training",
            "framework": "rust+python",
            "notes": "基于实际下载的LiquidAI LFM2.5-1.2B-Thinking模型创建的拆分元数据"
        }
    });
    
    // 保存元数据到文件
    let metadata_file = "./test_models/lfm2_1.2b_split_metadata.json";
    std::fs::create_dir_all("./test_models")?;
    std::fs::write(metadata_file, serde_json::to_string_pretty(&model_metadata)?)?;
    
    println!("✅ 真实模型元数据已创建: {}", metadata_file);
    
    // 显示元数据摘要
    println!("\n📋 元数据摘要:");
    println!("   - 模型: {}", model_metadata["model_info"]["model_name"]);
    println!("   - 层数: {}", model_metadata["model_info"]["num_hidden_layers"]);
    println!("   - 参数量: {:?}", model_metadata["model_info"]["estimated_params"]);
    println!("   - 拆分节点数: {}", model_metadata["split_plan"]["num_nodes"]);
    
    for (i, split) in model_metadata["split_plan"]["splits"].as_array().unwrap().iter().enumerate() {
        println!("   - 节点 {}: {} 层, {:.2} GB", 
                 i+1, 
                 split["num_layers"], 
                 split["estimated_size_gb"]);
    }
    
    // ==================== 上传到 Hugging Face ====================
    println!("\n📤 准备上传到 Hugging Face 仓库: logos42/williw");
    
    // 检查是否有 HF token
    let hf_token = std::env::var("HF_TOKEN").ok();
    
    if hf_token.is_none() {
        println!("⚠️  未设置 HF_TOKEN 环境变量");
        println!("💡 请设置 HF token 后重新运行:");
        println!("   set HF_TOKEN=your_huggingface_token");
        println!("   cargo run --example upload_real_metadata");
        return Ok(());
    }
    
    println!("🔑 找到 HF token，开始上传...");
    
    let uploader = MetadataUploader::new();
    let upload_config = UploadConfig {
        metadata_file: metadata_file.to_string(),
        repo_id: "logos42/williw".to_string(),
        hf_token: hf_token.unwrap(),
        commit_message: Some("Add LFM2.5-1.2B-Thinking model split metadata for decentralized training".to_string()),
    };
    
    println!("🚀 开始上传到 https://huggingface.co/logos42/williw...");
    
    match uploader.upload_metadata(upload_config).await {
        Ok(result) => {
            println!("🎉 上传成功!");
            println!("📊 上传结果:");
            println!("   - 仓库: {}", result.repo);
            println!("   - 文件: {}", result.filename);
            println!("   - 访问URL: {}", result.url);
            println!("   - 提交URL: {}", result.commit_url);
            
            println!("\n✅ 元数据已成功上传到 Hugging Face!");
            println!("💡 你现在可以:");
            println!("   1. 访问 https://huggingface.co/logos42/williw 查看元数据");
            println!("   2. 使用这个元数据进行模型拆分");
            println!("   3. 开始分布式训练");
        }
        Err(e) => {
            println!("❌ 上传失败: {}", e);
            println!("💡 可能原因:");
            println!("   - HF token 无效或过期");
            println!("   - 对 logos42/williw 仓库没有写权限");
            println!("   - 网络连接问题");
            println!("   - 仓库不存在");
            
            println!("\n🔧 解决方案:");
            println!("   1. 检查 HF token: huggingface-cli whoami");
            println!("   2. 确保仓库存在: https://huggingface.co/logos42/williw");
            println!("   3. 检查权限设置");
        }
    }
    
    Ok(())
}
