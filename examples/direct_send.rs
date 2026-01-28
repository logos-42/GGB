/**
 * 直接发送模型文件 - 使用iroh底层API
 */

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tokio;
use tracing::info;
use tracing_subscriber;

use williw::comms::{IrohConnectionManager, IrohConnectionConfig};

/// 直接发送参数
#[derive(Parser)]
#[command(name = "direct-send")]
#[command(about = "直接发送文件到指定节点")]
pub struct DirectSendArgs {
    /// 要发送的文件路径
    #[arg(short, long)]
    pub file_path: PathBuf,
    
    /// 目标节点ID
    #[arg(short, long)]
    pub peer_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // 解析参数
    let args = DirectSendArgs::parse();

    info!("🚀 启动直接文件传输");
    info!("📁 文件路径: {}", args.file_path.display());
    info!("🎯 目标节点: {}", args.peer_id);

    // 检查文件是否存在
    if !args.file_path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", args.file_path.display()));
    }

    // 读取文件内容
    let file_content = tokio::fs::read(&args.file_path).await?;
    let file_size = file_content.len();
    
    info!("📊 文件大小: {} 字节", file_size);

    // 创建iroh连接管理器
    let config = IrohConnectionConfig {
        bind_addr: "0.0.0.0:0".to_string(),
        node_id: Some("sender".to_string()),
        bootstrap_nodes: vec![],
        enable_relay: true,
        max_connections: 10,
    };

    let connection_manager = IrohConnectionManager::new(config).await?;
    
    info!("🔗 尝试连接到目标节点...");
    
    // 连接到目标节点
    connection_manager.connect_to_peer(&args.peer_id).await?;
    
    info!("✅ 已连接到目标节点");
    
    // 发送文件内容
    info!("📤 开始发送文件...");
    
    connection_manager.send_message(&args.peer_id, file_content).await?;
    
    info!("✅ 文件发送成功！");
    info!("📊 发送了 {} 字节到 {}", file_size, args.peer_id);
    
    Ok(())
}
