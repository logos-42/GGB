/**
 * P2P节点带文件发送功能
 * 基于simple_p2p_demo，添加文件传输能力
 */

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tokio;
use tracing::info;
use tracing_subscriber;

use williw::comms::{P2PAppFactory, IrohConnectionManager, IrohConnectionConfig};

/// P2P节点参数
#[derive(Parser)]
#[command(name = "p2p-with-send")]
#[command(about = "P2P节点带文件发送功能")]
pub struct P2PWithSendArgs {
    /// 应用名称
    #[arg(long, default_value = "Williw P2P 节点")]
    pub app_name: String,
    
    /// 应用版本
    #[arg(long, default_value = "1.0.0")]
    pub version: String,
    
    /// 发送文件模式
    #[arg(long)]
    pub send_file: Option<PathBuf>,
    
    /// 目标节点ID
    #[arg(long)]
    pub peer_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // 解析命令行参数
    let args = P2PWithSendArgs::parse();

    info!("🚀 启动P2P节点（带文件发送功能）");
    info!("📦 应用名称: {}", args.app_name);
    info!("🏷️  版本: {}", args.version);

    // 创建P2P应用
    let app = P2PAppFactory::create_custom(args.app_name, args.version);

    // 启动应用
    app.start().await?;

    info!("✅ P2P应用启动成功");
    info!("📋 您可以将此节点ID分享给其他节点进行连接");

    // 如果指定了发送文件，则执行发送
    if let (Some(file_path), Some(peer_id)) = (args.send_file, args.peer_id) {
        info!("📤 检测到文件发送请求");
        info!("📁 文件路径: {}", file_path.display());
        info!("🎯 目标节点: {}", peer_id);
        
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {}", file_path.display()));
        }

        // 读取文件内容
        let file_content = tokio::fs::read(&file_path).await?;
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
        connection_manager.connect_to_peer(&peer_id).await?;
        
        info!("✅ 已连接到目标节点");
        
        // 发送文件内容
        info!("📤 开始发送文件...");
        
        connection_manager.send_message(&peer_id, file_content).await?;
        
        info!("✅ 文件发送成功！");
        info!("📊 发送了 {} 字节到 {}", file_size, peer_id);
        
        // 发送完成后继续运行节点
        info!("🔄 文件发送完成，继续运行P2P节点...");
    }

    info!("⏹️  按 Ctrl+C 停止应用");

    // 保持运行
    app.run().await
}
