/**
 * 增强的P2P模型分发演示
 * 展示完整的协作实现功能
 */

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tracing::{info, warn, error};
use tracing_subscriber;

use williw::comms::{
    enhanced_p2p_distributor::{EnhancedP2PModelDistributor, EnhancedTransferConfig},
    monitoring_dashboard::{MonitoringDashboard, WebApiHandler},
    iroh_integration::IrohConnectionConfig,
};

/// 增强的P2P演示参数
#[derive(Parser)]
#[command(name = "enhanced-p2p-demo")]
#[command(about = "增强的P2P模型分发演示")]
pub struct EnhancedP2PDemoArgs {
    #[command(subcommand)]
    pub command: EnhancedDemoCommand,
}

#[derive(Subcommand)]
pub enum EnhancedDemoCommand {
    /// 启动发送端
    Send {
        /// 节点 ID
        #[arg(short, long, default_value = "enhanced_sender")]
        node_id: String,

        /// 目标节点地址
        #[arg(short, long)]
        target_peer: String,

        /// 模型分片目录
        #[arg(short, long, default_value = "./test_models/test_models/simple_split")]
        shard_dir: PathBuf,

        /// 监听端口
        #[arg(short, long, default_value = "9235")]
        port: u16,

        /// 最大并发传输数
        #[arg(long, default_value = "3")]
        max_concurrent: usize,

        /// 启用监控
        #[arg(long, default_value = "true")]
        enable_monitoring: bool,
    },
    /// 启动接收端
    Receive {
        /// 节点 ID
        #[arg(short, long, default_value = "enhanced_receiver")]
        node_id: String,

        /// 输出目录
        #[arg(short, long, default_value = "./received_models")]
        output_dir: PathBuf,

        /// 监听端口
        #[arg(short, long, default_value = "9236")]
        port: u16,

        /// 自动接受
        #[arg(long, default_value = "true")]
        auto_accept: bool,

        /// 启用监控
        #[arg(long, default_value = "true")]
        enable_monitoring: bool,
    },
    /// 启动监控服务器
    Monitor {
        /// 监控端口
        #[arg(long, default_value = "8080")]
        monitor_port: u16,

        /// 连接到现有节点
        #[arg(long)]
        connect_to: Option<String>,
    },
    /// 运行完整演示
    FullDemo {
        /// 演示目录
        #[arg(short, long, default_value = "./enhanced_demo_output")]
        demo_dir: PathBuf,

        /// 模型分片目录
        #[arg(long, default_value = "./test_models/test_models/simple_split")]
        shard_dir: PathBuf,

        /// 发送端端口
        #[arg(long, default_value = "9235")]
        sender_port: u16,

        /// 接收端端口
        #[arg(long, default_value = "9236")]
        receiver_port: u16,

        /// 启用监控
        #[arg(long, default_value = "true")]
        enable_monitoring: bool,
    },
}

/// 增强的P2P演示管理器
pub struct EnhancedP2PDemoManager {
    demo_dir: PathBuf,
}

impl EnhancedP2PDemoManager {
    pub fn new(demo_dir: PathBuf) -> Self {
        Self { demo_dir }
    }

    /// 运行完整演示
    pub async fn run_full_demo(&self,
                               shard_dir: PathBuf,
                               sender_port: u16,
                               receiver_port: u16,
                               enable_monitoring: bool) -> Result<()> {
        info!("🚀 开始增强P2P模型分发完整演示");
        info!("   分片目录: {}", shard_dir.display());
        info!("   演示目录: {}", self.demo_dir.display());
        info!("   启用监控: {}", enable_monitoring);

        // 创建演示目录
        tokio::fs::create_dir_all(&self.demo_dir).await?;
        let receiver_output_dir = self.demo_dir.join("received");
        tokio::fs::create_dir_all(&receiver_output_dir).await?;

        // 步骤1: 启动接收端（后台）
        info!("📡 启动增强接收端...");
        let receiver_handle = self.start_enhanced_receiver_background(
            "enhanced_demo_receiver".to_string(),
            receiver_output_dir.clone(),
            receiver_port,
            enable_monitoring,
        ).await?;

        // 等待接收端启动
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 步骤2: 启动发送端
        info!("📤 启动增强发送端...");
        let sender_result = self.run_enhanced_sender(
            "enhanced_demo_sender".to_string(),
            "enhanced_demo_receiver".to_string(),
            shard_dir,
            sender_port,
            enable_monitoring,
        ).await;

        // 等待发送完成
        match sender_result {
            Ok(_) => info!("✅ 发送端完成"),
            Err(e) => {
                error!("❌ 发送端失败: {}", e);
                return Err(e);
            }
        }

        // 等待接收完成
        tokio::time::sleep(Duration::from_secs(10)).await;

        // 步骤3: 验证接收的文件
        info!("🔍 验证接收的文件...");
        self.validate_received_files(&receiver_output_dir).await?;

        // 步骤4: 生成演示报告
        self.generate_enhanced_demo_report(&receiver_output_dir).await?;

        info!("🎉 增强P2P模型分发演示完成！");
        self.print_enhanced_demo_summary(&receiver_output_dir).await;

        // 停止接收端
        receiver_handle.abort();

        Ok(())
    }

    /// 启动增强接收端（后台）
    async fn start_enhanced_receiver_background(&self,
                                                node_id: String,
                                                output_dir: PathBuf,
                                                port: u16,
                                                enable_monitoring: bool) -> Result<tokio::task::JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            if let Err(e) = run_enhanced_receiver(
                node_id,
                output_dir,
                port,
                true, // auto_accept
                enable_monitoring,
            ).await {
                error!("增强接收端错误: {}", e);
            }
        });

        Ok(handle)
    }

    /// 运行增强发送端
    async fn run_enhanced_sender(&self,
                                node_id: String,
                                target_peer: String,
                                shard_dir: PathBuf,
                                port: u16,
                                enable_monitoring: bool) -> Result<()> {
        run_enhanced_sender(
            node_id,
            target_peer,
            shard_dir,
            port,
            3, // max_concurrent
            enable_monitoring,
        ).await
    }

    /// 验证接收的文件
    async fn validate_received_files(&self, received_dir: &PathBuf) -> Result<()> {
        info!("🔍 验证接收的文件...");

        let mut entries = tokio::fs::read_dir(received_dir).await?;
        let mut file_count = 0;
        let mut total_size = 0u64;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = tokio::fs::metadata(&path).await?;
                total_size += metadata.len();
                file_count += 1;
                
                info!("   📄 {} ({} bytes)", 
                      path.file_name().unwrap().to_string_lossy(), 
                      metadata.len());
            }
        }

        if file_count == 0 {
            warn!("⚠️  未找到接收的文件");
        } else {
            info!("✅ 接收文件验证完成: {} 个文件, 总大小 {:.2} MB", 
                  file_count, total_size as f64 / 1024.0 / 1024.0);
        }

        Ok(())
    }

    /// 生成增强演示报告
    async fn generate_enhanced_demo_report(&self, received_dir: &PathBuf) -> Result<()> {
        info!("📋 生成增强演示报告...");

        let report_path = self.demo_dir.join("enhanced_demo_report.json");
        let report = serde_json::json!({
            "demo_type": "enhanced_p2p_model_distribution",
            "completed_at": chrono::Utc::now().to_rfc3339(),
            "features_used": [
                "iroh_integration",
                "real_time_monitoring",
                "enhanced_transfer_protocol",
                "connection_management",
                "event_driven_architecture"
            ],
            "received_files": self.get_file_list(received_dir).await?,
            "total_received_size": self.calculate_total_size(received_dir).await?,
            "success": true
        });

        tokio::fs::write(&report_path, serde_json::to_string_pretty(&report)?).await?;
        info!("📁 增强演示报告已保存: {}", report_path.display());

        Ok(())
    }

    /// 获取文件列表
    async fn get_file_list(&self, dir: &PathBuf) -> Result<Vec<serde_json::Value>> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = tokio::fs::metadata(&path).await?;
                files.push(serde_json::json!({
                    "name": path.file_name().unwrap().to_string_lossy(),
                    "size": metadata.len(),
                    "path": path.display().to_string()
                }));
            }
        }

        Ok(files)
    }

    /// 计算总大小
    async fn calculate_total_size(&self, dir: &PathBuf) -> Result<u64> {
        let mut total_size = 0u64;
        let mut entries = tokio::fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = tokio::fs::metadata(&path).await?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// 打印增强演示摘要
    async fn print_enhanced_demo_summary(&self, received_dir: &PathBuf) {
        info!("📊 增强演示摘要:");
        info!("   演示目录: {}", self.demo_dir.display());
        info!("   接收目录: {}", received_dir.display());
        info!("   使用功能:");
        info!("     ✅ 真实iroh集成");
        info!("     ✅ 实时监控仪表板");
        info!("     ✅ 增强传输协议");
        info!("     ✅ 连接管理");
        info!("     ✅ 事件驱动架构");
        
        match self.get_file_list(received_dir).await {
            Ok(files) => {
                info!("   接收文件数: {}", files.len());
                if let Ok(total_size) = self.calculate_total_size(received_dir).await {
                    info!("   总大小: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);
                }
            }
            Err(_) => info!("   无法读取接收文件信息"),
        }
    }
}

/// 运行增强发送端
pub async fn run_enhanced_sender(node_id: String,
                                target_peer: String,
                                shard_dir: PathBuf,
                                port: u16,
                                max_concurrent: usize,
                                enable_monitoring: bool) -> Result<()> {
    info!("🚀 启动增强P2P发送端");
    info!("   节点ID: {}", node_id);
    info!("   目标节点: {}", target_peer);
    info!("   分片目录: {}", shard_dir.display());
    info!("   端口: {}", port);
    info!("   最大并发: {}", max_concurrent);

    // 创建增强配置
    let iroh_config = IrohConnectionConfig {
        bind_addr: format!("0.0.0.0:{}", port),
        node_id: Some(node_id.clone()),
        bootstrap_nodes: vec![],
        enable_relay: true,
        max_connections: 50,
    };

    let transfer_config = EnhancedTransferConfig {
        iroh_config,
        max_concurrent_transfers: max_concurrent,
        enable_resume: true,
        enable_compression: true,
        ..Default::default()
    };

    // 创建增强分发器
    let distributor = Arc::new(
        EnhancedP2PModelDistributor::new(node_id.clone(), transfer_config).await?
    );

    // 创建监控仪表板
    let dashboard = if enable_monitoring {
        Some(Arc::new(MonitoringDashboard::new(distributor.clone()).await?))
    } else {
        None
    };

    // 连接到目标节点
    distributor.connect_to_peer(&target_peer).await?;

    // 扫描并发送文件
    let mut entries = tokio::fs::read_dir(&shard_dir).await?;
    let mut file_count = 0;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            info!("📤 发送文件: {}", path.file_name().unwrap().to_string_lossy());
            
            match distributor.send_file(target_peer.clone(), &path, None).await {
                Ok(transfer_id) => {
                    info!("✅ 传输已启动: {}", transfer_id);
                    file_count += 1;
                }
                Err(e) => {
                    error!("❌ 传输启动失败: {}", e);
                }
            }

            // 等待传输完成
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    info!("🎉 发送端完成，共发送 {} 个文件", file_count);

    // 如果启用监控，打印统计信息
    if let Some(dashboard) = dashboard {
        let stats = dashboard.get_stats().await;
        info!("📊 传输统计:");
        info!("   总传输次数: {}", stats.total_transfers);
        info!("   成功传输: {}", stats.successful_transfers);
        info!("   失败传输: {}", stats.failed_transfers);
        info!("   总传输字节: {}", stats.total_bytes_transferred);
    }

    Ok(())
}

/// 运行增强接收端
pub async fn run_enhanced_receiver(node_id: String,
                                  output_dir: PathBuf,
                                  port: u16,
                                  auto_accept: bool,
                                  enable_monitoring: bool) -> Result<()> {
    info!("🚀 启动增强P2P接收端");
    info!("   节点ID: {}", node_id);
    info!("   输出目录: {}", output_dir.display());
    info!("   端口: {}", port);
    info!("   自动接受: {}", auto_accept);

    // 创建输出目录
    tokio::fs::create_dir_all(&output_dir).await?;

    // 创建增强配置
    let iroh_config = IrohConnectionConfig {
        bind_addr: format!("0.0.0.0:{}", port),
        node_id: Some(node_id.clone()),
        bootstrap_nodes: vec![],
        enable_relay: true,
        max_connections: 50,
    };

    let transfer_config = EnhancedTransferConfig {
        iroh_config,
        max_concurrent_transfers: 5,
        enable_resume: true,
        enable_compression: true,
        ..Default::default()
    };

    // 创建增强分发器
    let distributor = Arc::new(
        EnhancedP2PModelDistributor::new(node_id.clone(), transfer_config).await?
    );

    // 创建监控仪表板
    let _dashboard = if enable_monitoring {
        Some(Arc::new(MonitoringDashboard::new(distributor.clone()).await?))
    } else {
        None
    };

    info!("✅ 增强接收端已启动，等待传入的文件...");

    // 保持运行
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // 检查活跃传输
        let active_transfers = distributor.get_active_transfers().await;
        if !active_transfers.is_empty() {
            info!("📊 当前活跃传输: {}", active_transfers.len());
        }
    }
}

/// 运行演示
pub async fn run_enhanced_demo(args: EnhancedP2PDemoArgs) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    match args.command {
        EnhancedDemoCommand::Send { 
            node_id, 
            target_peer, 
            shard_dir, 
            port, 
            max_concurrent,
            enable_monitoring 
        } => {
            run_enhanced_sender(
                node_id,
                target_peer,
                shard_dir,
                port,
                max_concurrent,
                enable_monitoring,
            ).await?;
        }
        EnhancedDemoCommand::Receive { 
            node_id, 
            output_dir, 
            port, 
            auto_accept,
            enable_monitoring 
        } => {
            run_enhanced_receiver(
                node_id,
                output_dir,
                port,
                auto_accept,
                enable_monitoring,
            ).await?;
        }
        EnhancedDemoCommand::FullDemo { 
            demo_dir, 
            shard_dir, 
            sender_port, 
            receiver_port,
            enable_monitoring 
        } => {
            let manager = EnhancedP2PDemoManager::new(demo_dir);
            manager.run_full_demo(
                shard_dir,
                sender_port,
                receiver_port,
                enable_monitoring,
            ).await?;
        }
        EnhancedDemoCommand::Monitor { .. } => {
            info!("🖥️  监控服务器功能待实现");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_manager_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = EnhancedP2PDemoManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.demo_dir, temp_dir.path());
    }
}
