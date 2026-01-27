//! 简单测试：验证 LFM2.5-1.2B-Thinking 模型拆分组件是否可用
//! 只测试组件初始化，不执行实际操作

use std::path::Path;

fn main() {
    println!("🧪 测试 LFM2.5-1.2B-Thinking 模型拆分组件可用性");
    
    // 模型配置
    let model_name = "LiquidAI/LFM2.5-1.2B-Thinking";
    let model_path = "./test_models/models--LiquidAI--LFM2.5-1.2B-Thinking";
    
    println!("📋 测试配置:");
    println!("   模型: {}", model_name);
    println!("   路径: {}", model_path);
    
    // === 测试 1: 检查模块导入 ===
    println!("\n🔍 测试 1: 检查模块导入");
    
    // 这些导入应该能成功，如果失败说明模块有问题
    use metadata_generator::{MetadataGenerator, MetadataConfig};
    use model_splitter::{ModelSplitter, SplitConfig, SplitPlan};
    use std::collections::HashMap;
    
    println!("✅ 所有模块导入成功");
    
    // === 测试 2: 组件初始化 ===
    println!("\n🔍 测试 2: 组件初始化");
    
    let _generator = MetadataGenerator::new();
    println!("✅ MetadataGenerator 初始化成功");
    
    let _splitter = ModelSplitter::new();
    println!("✅ ModelSplitter 初始化成功");
    
    // === 测试 3: 配置结构体创建 ===
    println!("\n🔍 测试 3: 配置结构体创建");
    
    let _metadata_config = MetadataConfig {
        model_name: model_name.to_string(),
        model_path: model_path.to_string(),
        batch_size: 1,
        sequence_length: 512,
        node_id: Some("test_node".to_string()),
    };
    println!("✅ MetadataConfig 创建成功");
    
    let split_plan = {
        let mut plan = HashMap::new();
        plan.insert(
            "test_node".to_string(),
            SplitPlan {
                node_id: "test_node".to_string(),
                layer_names: vec![
                    "transformer.h.0.attn.q_proj.weight".to_string(),
                    "transformer.h.0.attn.k_proj.weight".to_string(),
                ],
                total_compute: 100.0,
                compute_utilization: 0.5,
            },
        );
        plan
    };
    println!("✅ SplitPlan 创建成功");
    
    let _split_config = SplitConfig {
        model_name: model_name.to_string(),
        model_path: model_path.to_string(),
        split_plan,
        output_dir: Some("./test_output".to_string()),
    };
    println!("✅ SplitConfig 创建成功");
    
    // === 测试 4: 检查模型路径 ===
    println!("\n🔍 测试 4: 检查模型文件");
    
    if Path::new(model_path).exists() {
        println!("✅ 模型路径存在: {}", model_path);
        
        // 列出模型目录内容
        if let Ok(entries) = std::fs::read_dir(model_path) {
            let mut count = 0;
            for entry in entries.flatten() {
                count += 1;
                if count <= 5 { // 只显示前5个文件
                    println!("   📁 {}", entry.file_name().to_string_lossy());
                }
            }
            if count > 5 {
                println!("   ... 还有 {} 个文件", count - 5);
            }
            println!("✅ 模型目录包含 {} 个项目", count);
        }
    } else {
        println!("❌ 模型路径不存在: {}", model_path);
        println!("   请确保 LFM2.5-1.2B-Thinking 模型已下载到正确位置");
    }
    
    // === 测试总结 ===
    println!("\n📊 测试总结:");
    println!("   ✅ 模块导入: 成功");
    println!("   ✅ 组件初始化: 成功");
    println!("   ✅ 配置结构体: 成功");
    
    if Path::new(model_path).exists() {
        println!("   ✅ 模型文件: 存在");
        println!("\n🎉 所有组件测试通过！");
        println!("💡 可以开始完整的模型拆分流程：");
        println!("   1. 生成元数据: generator.generate_metadata(config)");
        println!("   2. 创建拆分方案: 根据元数据分配层");
        println!("   3. 执行模型拆分: splitter.split_model(config, node_id)");
    } else {
        println!("   ⚠️  模型文件: 不存在");
        println!("\n⚠️  组件可用，但需要先下载模型文件。");
    }
}
