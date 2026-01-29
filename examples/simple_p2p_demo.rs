/**
 * 简单的P2P演示
 * 完整的收发功能，支持文件传输和消息传递
 */

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio;
use tracing::{info, error, warn};
use tracing_subscriber;
use chrono;
use serde::{Serialize, Deserialize};

use williw::comms::{P2PAppFactory, IrohConnectionManager, IrohConnectionConfig};

/// 文件传输消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileTransferMessage {
    /// 消息类型
    message_type: String,
    /// 原始文件名
    filename: String,
    /// 文件大小
    file_size: u64,
    /// 文件内容
    content: Vec<u8>,
    /// 发送时间
    timestamp: chrono::DateTime<chrono::Utc>,
    /// 发送者节点ID
    sender_id: String,
}

impl FileTransferMessage {
    fn new(filename: String, content: Vec<u8>, sender_id: String) -> Self {
        let file_size = content.len() as u64;
        Self {
            message_type: "file_transfer".to_string(),
            filename,
            file_size,
            content,
            timestamp: chrono::Utc::now(),
            sender_id,
        }
    }
    
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
    
    fn deserialize(data: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

/// 简单P2P演示
#[derive(Parser)]
#[command(name = "simple-p2p")]
#[command(about = "简单的P2P节点演示（支持文件传输和消息传递）")]
pub struct SimpleP2PArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 启动接收节点
    Receive {
        /// 应用名称
        #[arg(long, default_value = "Williw P2P 接收节点")]
        app_name: String,
        
        /// 应用版本
        #[arg(long, default_value = "1.0.0")]
        version: String,
        
        /// 接收目录
        #[arg(long, default_value = "./received_files")]
        output_dir: PathBuf,
    },
    /// 发送文件到指定节点
    Send {
        /// 应用名称
        #[arg(long, default_value = "Williw P2P 发送节点")]
        app_name: String,
        
        /// 应用版本
        #[arg(long, default_value = "1.0.0")]
        version: String,
        
        /// 要发送的文件路径
        #[arg(long)]
        file_path: PathBuf,
        
        /// 目标节点ID
        #[arg(long)]
        target_node: String,
        
        /// 连接超时时间（秒）
        #[arg(long, default_value = "30")]
        _timeout: u64,
        
        /// 重试次数
        #[arg(long, default_value = "3")]
        retry_count: u32,
    },
    /// 启动交互式节点
    Interactive {
        /// 应用名称
        #[arg(long, default_value = "Williw P2P 交互节点")]
        app_name: String,
        
        /// 应用版本
        #[arg(long, default_value = "1.0.0")]
        version: String,
        
        /// 接收目录
        #[arg(long, default_value = "./received_files")]
        output_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志（只设置一次）
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // 解析命令行参数
    let args = SimpleP2PArgs::parse();

    match args.command {
        Commands::Receive { app_name, version, output_dir } => {
            start_receive_node(app_name, version, output_dir).await
        }
        Commands::Send { 
            app_name, 
            version, 
            file_path, 
            target_node, 
            _timeout, 
            retry_count 
        } => {
            send_file_to_node(app_name, version, file_path, target_node, _timeout, retry_count).await
        }
        Commands::Interactive { app_name, version, output_dir } => {
            start_interactive_node(app_name, version, output_dir).await
        }
    }
}

/// 启动接收节点
async fn start_receive_node(app_name: String, version: String, output_dir: PathBuf) -> Result<()> {
    info!("🚀 启动P2P接收节点");
    info!("📦 应用名称: {}", app_name);
    info!("🏷️  版本: {}", version);
    info!("📁 接收目录: {}", output_dir.display());

    // 创建P2P应用
    let app = P2PAppFactory::create_custom(app_name, version);
    app.start().await?;

    // 创建iroh连接管理器
    let connection_manager = create_connection_manager().await?;
    let node_id = connection_manager.node_id();
    
    info!("🎉 ===== P2P接收节点启动成功 =====");
    info!("🔑 iroh节点ID (z-base32格式):");
    info!("   {}", node_id);
    info!("📋 其他节点可以使用此ID连接到您的节点");
    info!("🔗 连接命令示例:");
    info!("   cargo run --example simple_p2p_demo -- send --file-path <文件> --target-node {}", node_id);
    info!("⏹️  按 Ctrl+C 停止接收节点");
    info!("================================");

    // 启动接收服务
    let connection_manager_clone = connection_manager.clone();
    let output_dir_clone = output_dir.clone();
    tokio::spawn(async move {
        start_receive_service(connection_manager_clone, output_dir_clone).await;
    });

    // 保持运行
    app.run().await
}

/// 发送文件到指定节点
async fn send_file_to_node(
    app_name: String,
    version: String,
    file_path: PathBuf,
    target_node: String,
    _timeout: u64,
    retry_count: u32,
) -> Result<()> {
    info!("🚀 启动P2P发送节点");
    info!("� 应用名称: {}", app_name);
    info!("🏷️  版本: {}", version);
    info!("📁 文件路径: {}", file_path.display());
    info!("🎯 目标节点: {}", target_node);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file_path.display()));
    }

    // 读取文件内容
    let file_content = tokio::fs::read(&file_path).await?;
    let file_size = file_content.len();
    let filename = file_path.file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    info!("📊 文件信息:");
    info!("   📄 文件名: {}", filename);
    info!("   📏 大小: {} 字节", file_size);

    // 创建P2P应用和连接管理器
    let app = P2PAppFactory::create_custom(app_name, version);
    app.start().await?;

    let connection_manager = create_connection_manager().await?;
    let sender_id = connection_manager.node_id();
    
    info!("🔑 发送方节点 ID: {}", sender_id);

    // 连接到目标节点（带重试）
    info!("🔗 尝试连接到目标节点...");
    
    for attempt in 1..=retry_count {
        match connection_manager.connect_to_peer(&target_node).await {
            Ok(_) => {
                info!("✅ 成功连接到目标节点 (尝试 {}/{})", attempt, retry_count);
                break;
            }
            Err(e) => {
                if attempt == retry_count {
                    error!("❌ 所有连接尝试都失败了");
                    return Err(anyhow::anyhow!("连接失败: {}", e));
                }
                warn!("⚠️  连接尝试 {}/{} 失败: {}", attempt, retry_count, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }

    // 创建文件传输消息
    let file_message = FileTransferMessage::new(filename.clone(), file_content.clone(), sender_id);
    let serialized_message = file_message.serialize()?;

    info!("📤 开始发送文件...");
    info!("📦 消息大小: {} 字节", serialized_message.len());

    // 发送文件
    let serialized_size = serialized_message.len();
    connection_manager.send_message(&target_node, serialized_message).await?;

    info!("🎉 文件发送成功！");
    info!("📊 发送统计:");
    info!("   📄 文件名: {}", filename);
    info!("   📏 原始大小: {} 字节", file_size);
    info!("   📦 传输大小: {} 字节", serialized_size);
    info!("   🎯 目标节点: {}", target_node);

    Ok(())
}

/// 启动交互式节点
async fn start_interactive_node(app_name: String, version: String, output_dir: PathBuf) -> Result<()> {
    info!("🚀 启动P2P交互节点");
    info!("📦 应用名称: {}", app_name);
    info!("🏷️  版本: {}", version);
    info!("📁 接收目录: {}", output_dir.display());

    // 创建P2P应用
    let app = P2PAppFactory::create_custom(app_name, version);
    app.start().await?;

    // 创建iroh连接管理器
    let connection_manager = create_connection_manager().await?;
    let node_id = connection_manager.node_id();
    
    info!("� 节点 ID: {}", node_id);
    info!("📋 您可以将此节点 ID 分享给其他节点");
    info!("⏹️  按 Ctrl+C 停止交互式节点");

    // 启动接收服务
    let connection_manager_clone = connection_manager.clone();
    let output_dir_clone = output_dir.clone();
    tokio::spawn(async move {
        start_receive_service(connection_manager_clone, output_dir_clone).await;
    });

    // 保持运行
    app.run().await
}

/// 创建iroh连接管理器
async fn create_connection_manager() -> Result<IrohConnectionManager> {
    info!("🔗 初始化 iroh 连接管理器...");
    
    let config = IrohConnectionConfig {
        bind_addr: "0.0.0.0:0".to_string(),
        node_id: Some("simple_p2p_node".to_string()),
        bootstrap_nodes: vec![],
        enable_relay: true,
        max_connections: 50,
    };

    let connection_manager = IrohConnectionManager::new(config).await?;
    
    // 显示详细的连接信息
    let node_id = connection_manager.node_id();
    info!("✅ iroh 连接管理器初始化成功");
    info!("🔑 本节点ID: {}", node_id);
    info!("🌐 已启用中继服务器支持");
    info!("📡 最大连接数: 50");
    
    Ok(connection_manager)
}

/// 启动接收服务
async fn start_receive_service(connection_manager: IrohConnectionManager, output_dir: PathBuf) {
    info!("🔄 启动P2P接收服务...");
    
    // 创建接收目录
    if let Err(e) = tokio::fs::create_dir_all(&output_dir).await {
        error!("❌ 无法创建接收目录: {}", e);
        return;
    }
    info!("📁 文件接收目录: {}", output_dir.display());
    info!("👂 正在监听传入的文件传输...");
    
    let mut file_counter = 0;
    let mut last_activity = chrono::Utc::now();
    
    // 持续监听传入的消息
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // 尝试接收消息
        match connection_manager.receive_message().await {
            Ok(Some((sender_id, data))) => {
                last_activity = chrono::Utc::now();
                info!("📥 ===== 收到文件传输 =====");
                info!("👤 发送方节点: {}", sender_id);
                info!("📦 数据大小: {} 字节", data.len());
                
                // 尝试解析文件传输消息
                match FileTransferMessage::deserialize(&data) {
                    Ok(file_message) => {
                        info!("✅ 成功解析文件传输消息");
                        info!("📄 原始文件名: {}", file_message.filename);
                        info!("📏 文件大小: {} 字节", file_message.file_size);
                        info!("🕐 发送时间: {}", file_message.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
                        info!("👤 发送方节点ID: {}", file_message.sender_id);
                        
                        // 生成接收文件名
                        file_counter += 1;
                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                        let safe_filename = sanitize_filename(&file_message.filename);
                        let filename = format!("received_{}_{}_{}", timestamp, file_counter, safe_filename);
                        let filepath = output_dir.join(filename);
                        
                        // 保存文件
                        match tokio::fs::write(&filepath, &file_message.content).await {
                            Ok(_) => {
                                info!("🎉 ===== 文件接收成功 =====");
                                info!("💾 保存路径: {}", filepath.display());
                                info!("📊 接收统计:");
                                info!("   📄 原始文件名: {}", file_message.filename);
                                info!("   📏 实际大小: {} 字节", file_message.content.len());
                                info!("   👤 发送方节点: {}", sender_id);
                                info!("   🕐 接收时间: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
                                info!("==============================");
                            }
                            Err(e) => {
                                error!("❌ 保存文件失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  无法解析文件传输消息，保存为原始数据: {}", e);
                        
                        // 如果不是文件传输消息，保存为原始数据
                        file_counter += 1;
                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                        let filename = format!("raw_data_{}_{}.bin", timestamp, file_counter);
                        let filepath = output_dir.join(filename);
                        
                        match tokio::fs::write(&filepath, &data).await {
                            Ok(_) => {
                                info!("💾 原始数据已保存: {}", filepath.display());
                                info!("📊 数据大小: {} 字节", data.len());
                                info!("👤 发送方: {}", sender_id);
                            }
                            Err(e) => {
                                error!("❌ 保存原始数据失败: {}", e);
                            }
                        }
                    }
                }
            }
            Ok(None) => {
                // 没有消息，显示状态信息（每30秒一次）
                let now = chrono::Utc::now();
                if (now - last_activity).num_seconds() > 30 {
                    info!("👂 正在监听传入连接... (已运行 {} 秒)", (now - last_activity).num_seconds());
                    last_activity = now;
                }
                continue;
            }
            Err(e) => {
                warn!("⚠️ 接收消息时出错: {}", e);
                // 继续运行，不因为单个错误停止服务
            }
        }
    }
}

/// 清理文件名，移除不安全字符
fn sanitize_filename(filename: &str) -> String {
    let mut safe = String::new();
    for c in filename.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | ' ' => safe.push(c),
            _ => safe.push('_'),
        }
    }
    
    // 限制文件名长度
    if safe.len() > 100 {
        safe.truncate(100);
    }
    
    if safe.is_empty() {
        "unnamed_file".to_string()
    } else {
        safe
    }
}
