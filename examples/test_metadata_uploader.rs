/**
 * 测试元数据上传到 Hugging Face
 * 注意：这需要有效的 HF token
 */
use metadata_uploader::{MetadataUploader, UploadConfig};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始测试元数据上传功能");
    
    // ==================== 创建测试元数据 ====================
    println!("\n📄 创建测试元数据...");
    
    let test_metadata = serde_json::json!({
        "model_name": "LiquidAI/LFM2.5-1.2B-Thinking",
        "split_info": {
            "total_params": 224397312,
            "num_nodes": 2,
            "splits": [
                {
                    "node_id": "node_001",
                    "layers": ["model.embed_tokens.weight", "model.layers.0.conv1.weight"],
                    "params": 213909504,
                    "size_mb": 816.0
                },
                {
                    "node_id": "node_002", 
                    "layers": ["model.layers.2.attn.q_proj.weight", "model.layers.15.output_layernorm.weight"],
                    "params": 10487808,
                    "size_mb": 40.0
                }
            ]
        },
        "test_timestamp": "2026-01-25T08:45:00Z",
        "test_mode": true
    });
    
    // 保存测试元数据到文件
    let metadata_file = "./test_models/test_upload_metadata.json";
    std::fs::create_dir_all("./test_models")?;
    std::fs::write(metadata_file, serde_json::to_string_pretty(&test_metadata)?)?;
    
    println!("✅ 测试元数据已创建: {}", metadata_file);
    
    // ==================== 测试 MetadataUploader ====================
    println!("\n📤 测试 MetadataUploader 模块...");
    
    // 注意：这里需要真实的 HF token
    let hf_token = std::env::var("HF_TOKEN").ok();
    
    if hf_token.is_none() {
        println!("⚠️  未设置 HF_TOKEN 环境变量");
        println!("💡 要测试上传功能，请设置:");
        println!("   export HF_TOKEN=your_huggingface_token");
        println!("   或者在 Windows 中:");
        println!("   set HF_TOKEN=your_huggingface_token");
        
        // 我们继续测试模块的基本功能，但不实际上传
        println!("\n🧪 测试模块基本功能（不上传）...");
        
        let uploader = MetadataUploader::new();
        println!("✅ MetadataUploader 创建成功");
        
        // 创建一个模拟的上传配置
        let upload_config = UploadConfig {
            metadata_file: metadata_file.to_string(),
            repo_id: "test-repo/model-metadata".to_string(),
            hf_token: "dummy_token_for_testing".to_string(),
            commit_message: Some("Test upload from Rust module".to_string()),
        };
        
        println!("✅ UploadConfig 创建成功");
        println!("   - 元数据文件: {}", upload_config.metadata_file);
        println!("   - 目标仓库: {}", upload_config.repo_id);
        println!("   - 提交信息: {:?}", upload_config.commit_message);
        
        return Ok(());
    }
    
    // 如果有 token，尝试实际上传
    println!("🔑 找到 HF token，尝试实际上传...");
    
    let uploader = MetadataUploader::new();
    let hf_token_clone = hf_token.clone().unwrap(); // 克隆以避免所有权问题
    let upload_config = UploadConfig {
        metadata_file: metadata_file.to_string(),
        repo_id: "your-username/test-model-metadata".to_string(), // 需要修改为实际的仓库
        hf_token: hf_token_clone,
        commit_message: Some("Test model split metadata upload".to_string()),
    };
    
    println!("🚀 开始上传...");
    match uploader.upload_metadata(upload_config).await {
        Ok(result) => {
            println!("✅ 上传成功!");
            println!("   - 仓库: {}", result.repo);
            println!("   - 文件: {}", result.filename);
            println!("   - URL: {}", result.url);
            println!("   - 提交URL: {}", result.commit_url);
        }
        Err(e) => {
            println!("❌ 上传失败: {}", e);
            println!("💡 可能原因:");
            println!("   - HF token 无效");
            println!("   - 目标仓库不存在或无权限");
            println!("   - 网络问题");
        }
    }
    
    // ==================== 总结 ====================
    println!("\n🎉 元数据上传测试完成!");
    println!("📊 测试结果:");
    println!("   ✅ MetadataUploader: 模块创建成功");
    println!("   ✅ UploadConfig: 配置正常");
    println!("   ✅ 元数据生成: JSON 格式正确");
    
    if hf_token.is_some() {
        println!("   📤 实际上传: 已尝试");
    } else {
        println!("   ⚠️  实际上传: 需要 HF_TOKEN");
    }
    
    Ok(())
}
