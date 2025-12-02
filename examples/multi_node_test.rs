//! 多节点协同训练测试工具
//!
//! 使用方法：
//! ```bash
//! cargo run --example multi_node_test -- --nodes 3 --duration 300
//! ```

use anyhow::Result;
use clap::Parser;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(name = "multi_node_test")]
#[command(about = "多节点协同训练测试工具")]
struct Args {
    /// 节点数量
    #[arg(short, long, default_value_t = 3)]
    nodes: usize,

    /// 训练持续时间（秒）
    #[arg(short, long, default_value_t = 300)]
    duration: u64,

    /// 模型维度
    #[arg(short, long, default_value_t = 256)]
    model_dim: usize,

    /// 日志级别（debug, info, warn, error）
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// 输出目录（用于保存统计数据和日志）
    #[arg(short, long, default_value = "test_output")]
    output_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("=== GGS 多节点协同训练测试 ===");
    println!("节点数量: {}", args.nodes);
    println!("训练时长: {} 秒", args.duration);
    println!("模型维度: {}", args.model_dim);
    println!("输出目录: {}", args.output_dir);

    // 创建输出目录
    std::fs::create_dir_all(&args.output_dir)?;

    // 启动多个节点进程
    let mut handles = Vec::new();
    for i in 0..args.nodes {
        let node_id = i;
        let output_dir = args.output_dir.clone();
        let model_dim = args.model_dim;
        let log_level = args.log_level.clone();

        let handle = tokio::spawn(async move {
            start_node(node_id, output_dir, model_dim, log_level).await
        });
        handles.push(handle);

        // 错开启动时间，避免端口冲突
        sleep(Duration::from_millis(500)).await;
    }

    println!("\n所有节点已启动，开始训练...");
    println!("等待 {} 秒后自动停止...\n", args.duration);

    // 等待指定时间
    sleep(Duration::from_secs(args.duration)).await;

    println!("\n训练时间到，正在停止所有节点...");

    // 停止所有节点（通过发送信号或等待进程结束）
    for handle in handles {
        let _ = handle.await;
    }

    println!("\n=== 测试完成 ===");
    println!("统计数据已保存到: {}", args.output_dir);
    println!("\n可以使用以下命令分析结果：");
    println!("  cargo run --bin analyze_training -- --input {}", args.output_dir);

    Ok(())
}

async fn start_node(
    node_id: usize,
    output_dir: String,
    model_dim: usize,
    log_level: String,
) -> Result<()> {
    // 设置环境变量来模拟不同的设备
    let device_type = match node_id % 3 {
        0 => "low",
        1 => "mid",
        _ => "high",
    };

    let log_file = format!("{}/node_{}.log", output_dir, node_id);
    let stats_file = format!("{}/node_{}_stats.json", output_dir, node_id);

    // 启动节点进程
    // 注意：这里需要实际编译后的二进制文件
    // 在实际使用中，可能需要先编译，然后运行编译后的二进制
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--",
            "--node-id",
            &node_id.to_string(),
            "--model-dim",
            &model_dim.to_string(),
            "--stats-output",
            &stats_file,
        ])
        .env("GGS_DEVICE_TYPE", device_type)
        .env("RUST_LOG", &log_level)
        .stdout(Stdio::from(
            std::fs::File::create(&log_file).expect("无法创建日志文件"),
        ))
        .stderr(Stdio::from(
            std::fs::File::create(format!("{}/node_{}_error.log", output_dir, node_id))
                .expect("无法创建错误日志文件"),
        ))
        .spawn()?;

    // 等待进程结束
    let _ = child.wait()?;

    Ok(())
}

