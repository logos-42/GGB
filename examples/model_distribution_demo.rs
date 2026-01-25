/**
 * 模型分发完整流程演示
 * 从下载到拆分的完整测试
 */
use model_downloader::{ModelDownloader, DownloadConfig};
use metadata_generator::{MetadataGenerator, MetadataConfig};
use model_splitter::{ModelSplitter, SplitConfig, SplitPlan};
use std::collections::HashMap;
use tracing::{info, warn, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 开始模型分发完整流程测试");
    
    // 配置信息
    let model_name = "LiquidAI/LFM2.5-1.2B-Thinking";
    let hf_token = std::env::var("HF_TOKEN").ok();
    let node_id = "test_node_001";
    
    // ==================== 步骤1: 下载模型 ====================
    info!("📥 步骤1: 下载模型...");
    
    let downloader = ModelDownloader::new(hf_token.clone());
    let download_config = DownloadConfig {
        model_name: model_name.to_string(),
        cache_dir: Some("./test_models/rust_download".to_string()),
        hf_token: hf_token.clone(),
    };
    
    let download_result = downloader.download_model(download_config).await
        .map_err(|e| {
            warn!("⚠️  下载失败，使用已存在的模型: {}", e);
            e
        });
    
    let model_path = match download_result {
        Ok(result) => {
            info!("✅ 模型下载完成: {}", result.model_path);
            result.model_path
        }
        Err(_) => {
            // 使用已下载的模型
            let existing_path = "./test_models/models--LiquidAI--LFM2.5-1.2B-Thinking/snapshots/3c58ec1db4a336594e9ac4ad3fff10fc8aa22d70".to_string();
            info!("📂 使用已存在的模型: {}", existing_path);
            existing_path
        }
    };
    
    // ==================== 步骤2: 生成元数据 ====================
    info!("📊 步骤2: 生成模型元数据...");
    
    let generator = MetadataGenerator::new();
    let metadata_config = MetadataConfig {
        model_name: model_name.to_string(),
        model_path: model_path.clone(),
        batch_size: 1,
        sequence_length: 512,
        node_id: Some(node_id.to_string()),
    };
    
    let metadata = generator.generate_metadata(metadata_config).await
        .map_err(|e| {
            error!("❌ 元数据生成失败: {}", e);
            e
        })?;
    
    info!("✅ 元数据生成完成");
    info!("   - 模型类型: {}", metadata.model_type);
    info!("   - 总层数: {}", metadata.total_layers);
    info!("   - 总计算需求: {:.2}", metadata.total_compute);
    
    // 保存元数据到文件
    let metadata_file = format!("./test_models/metadata_{}.json", model_name.replace("/", "_"));
    generator.save_metadata(&metadata, &metadata_file).await?;
    info!("📁 元数据已保存: {}", metadata_file);
    
    // ==================== 步骤3: 创建拆分方案 ====================
    info!("🎯 步骤3: 创建拆分方案...");
    
    // 模拟节点信息（实际应该从节点注册获取）
    let mut split_plan = HashMap::new();
    
    // 节点1: 处理前半部分层
    let node1_layers = metadata.layers.iter()
        .take(metadata.layers.len() / 2)
        .map(|l| l.name.clone())
        .collect();
    
    split_plan.insert(
        "node_001".to_string(),
        SplitPlan {
            node_id: "node_001".to_string(),
            layer_names: node1_layers,
            total_compute: metadata.total_compute / 2.0,
            compute_utilization: 0.8,
        },
    );
    
    // 节点2: 处理后半部分层
    let node2_layers = metadata.layers.iter()
        .skip(metadata.layers.len() / 2)
        .map(|l| l.name.clone())
        .collect();
    
    split_plan.insert(
        "node_002".to_string(),
        SplitPlan {
            node_id: "node_002".to_string(),
            layer_names: node2_layers,
            total_compute: metadata.total_compute / 2.0,
            compute_utilization: 0.8,
        },
    );
    
    info!("✅ 拆分方案创建完成");
    for (node_id, plan) in &split_plan {
        info!("   - {}: {} 层, 计算需求: {:.2}", 
              node_id, plan.layer_names.len(), plan.total_compute);
    }
    
    // 验证拆分方案
    let splitter = ModelSplitter::new();
    let all_layer_names: Vec<String> = metadata.layers.iter()
        .map(|l| l.name.clone())
        .collect();
    
    splitter.validate_split_plan(&all_layer_names, &split_plan)?;
    info!("✅ 拆分方案验证通过");
    
    // ==================== 步骤4: 执行模型拆分 ====================
    info!("⚡ 步骤4: 执行模型拆分...");
    
    let split_config = SplitConfig {
        model_name: model_name.to_string(),
        model_path: model_path.clone(),
        split_plan: split_plan.clone(),
        output_dir: Some("./test_models/model_shards".to_string()),
    };
    
    // 为每个节点执行拆分
    let mut split_results = Vec::new();
    for node_id in split_plan.keys() {
        info!("🔧 正在为节点 {} 拆分模型...", node_id);
        
        match splitter.split_model(split_config.clone(), node_id).await {
            Ok(result) => {
                info!("✅ 节点 {} 拆分完成", node_id);
                info!("   - 分片路径: {}", result.shard_path);
                info!("   - 参数数量: {}", result.total_params);
                info!("   - 分片大小: {:.2} MB", result.shard_size_mb);
                split_results.push(result);
            }
            Err(e) => {
                error!("❌ 节点 {} 拆分失败: {}", node_id, e);
                return Err(e);
            }
        }
    }
    
    // ==================== 步骤5: 生成分发报告 ====================
    info!("📋 步骤5: 生成分发报告...");
    
    let total_params: usize = split_results.iter()
        .map(|r| r.total_params)
        .sum();
    
    let total_size_mb: f64 = split_results.iter()
        .map(|r| r.shard_size_mb)
        .sum();
    
    info!("🎉 模型分发流程完成!");
    info!("📊 最终统计:");
    info!("   - 原始模型: {}", model_name);
    info!("   - 拆分节点数: {}", split_results.len());
    info!("   - 总参数数量: {}", total_params);
    info!("   - 总分片大小: {:.2} MB", total_size_mb);
    info!("   - 平均每节点: {:.2} MB", total_size_mb / split_results.len() as f64);
    
    // 保存分发报告
    let report = serde_json::json!({
        "model_name": model_name,
        "model_path": model_path,
        "metadata_file": metadata_file,
        "split_plan": split_plan,
        "split_results": split_results,
        "total_params": total_params,
        "total_size_mb": total_size_mb,
        "completed_at": chrono::Utc::now().to_rfc3339()
    });
    
    let report_file = "./test_models/distribution_report.json";
    tokio::fs::write(report_file, serde_json::to_string_pretty(&report)?).await?;
    info!("📁 分发报告已保存: {}", report_file);
    
    info!("🚀 测试完成! 可以开始分布式训练了!");
    
    Ok(())
}
