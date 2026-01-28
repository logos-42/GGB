/**
 * 发送模型文件到指定节点
 */

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tokio;
use tracing::info;
use tracing_subscriber;

use williw::comms::{P2PAppFactory, P2PModelDistributor, TransferEvent};

/// 发送模型参数
#[derive(Parser)]
#[command(name = "send-model")]
#[command(about = "发送模型文件到指定P2P节点")]
pub struct SendModelArgs {
    /// 要发送的文件路径
    #[arg(short, long)]
    pub file_path: PathBuf,
    
    /// 目标节点ID
    #[arg(short, long)]
    pub peer_id: String,
    
    /// 分片大小（字节）
    #[arg(long, default_value = "65536")]
    pub chunk_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // 解析参数
    let args = SendModelArgs::parse();

    info!("🚀 启动模型发送工具");
    info!("📁 文件路径: {}", args.file_path.display());
    info!("🎯 目标节点: {}", args.peer_id);
    info!("📦 分片大小: {} 字节", args.chunk_size);

    // 检查文件是否存在
    if !args.file_path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", args.file_path.display()));
    }

    // 创建P2P应用
    let app = P2PAppFactory::create_default();
    app.start().await?;

    info!("✅ P2P应用启动成功");

    // 创建模型分发器
    let distributor = P2PModelDistributor::new("sender_node".to_string());
    
    info!("🔗 开始连接到目标节点...");
    
    // 连接到目标节点
    distributor.connect_to_peer(&args.peer_id).await?;
    
    info!("✅ 已连接到目标节点");
    
    // 发送文件
    info!("📤 开始发送文件...");
    
    let transfer_id = distributor.send_file(
        &args.file_path,
        &args.peer_id,
        args.chunk_size
    ).await?;
    
    info!("📊 传输ID: {}", transfer_id);
    info!("⏳ 等待传输完成...");
    
    // 监控传输进度
    let mut event_rx = distributor.get_event_receiver();
    
    while let Some(event) = event_rx.recv().await {
        match event {
            TransferEvent::TransferStarted { transfer_id: id, file_name, .. } => {
                if id == transfer_id {
                    info!("🚀 传输开始: {}", file_name);
                }
            }
            TransferEvent::ProgressUpdate { transfer_id: id, progress, speed_bps } => {
                if id == transfer_id {
                    info!("📊 传输进度: {:.1}% (速度: {:.2} MB/s)", progress, speed_bps as f64 / 1024.0 / 1024.0);
                }
            }
            TransferEvent::TransferCompleted { transfer_id: id, .. } => {
                if id == transfer_id {
                    info!("✅ 传输完成！");
                    break;
                }
            }
            TransferEvent::TransferFailed { transfer_id: id, error, .. } => {
                if id == transfer_id {
                    return Err(anyhow::anyhow!("传输失败: {}", error));
                }
            }
            _ => {}
        }
    }
    
    info!("🎉 模型发送成功！");
    
    Ok(())
}
