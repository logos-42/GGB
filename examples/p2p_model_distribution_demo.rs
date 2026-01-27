/**
 * P2P 模型分发完整演示
 * 演示发送端和接收端的完整工作流程
 */

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tokio;
use tracing::{info, warn, error};
use tracing_subscriber;

// 临时导入模块，因为示例程序无法直接访问crate
// 在实际使用中，这些模块应该作为独立的二进制程序运行
use williw::comms::{
    p2p_sender::{run_sender, P2PSenderArgs},
    p2p_receiver::{run_receiver, P2PReceiverArgs},
    transfer_protocol::{FileTransferProtocol, TransferProtocolConfig, ChecksumAlgorithm},
};

/// P2P 模型分发演示
#[derive(Parser)]
#[command(name = "p2p-demo")]
#[command(about = "P2P 模型分发完整演示")]
pub struct P2PDemoArgs {
    #[command(subcommand)]
    pub command: DemoCommand,
}

#[derive(Subcommand)]
pub enum DemoCommand {
    /// 启动发送端
    Send {
        /// 节点 ID
        #[arg(short, long, default_value = "demo_sender")]
        node_id: String,

        /// 目标节点 ID
        #[arg(short, long)]
        target_peer: String,

        /// 模型分片目录
        #[arg(short, long, default_value = "./test_models/test_models/simple_split")]
        shard_dir: PathBuf,

        /// 块大小
        #[arg(short, long, default_value = "1048576")]
        chunk_size: usize,

        /// 端口
        #[arg(short, long, default_value = "9235")]
        port: u16,
    },
    /// 启动接收端
    Receive {
        /// 节点 ID
        #[arg(short, long, default_value = "demo_receiver")]
        node_id: String,

        /// 输出目录
        #[arg(short, long, default_value = "./received_models")]
        output_dir: PathBuf,

        /// 端口
        #[arg(short, long, default_value = "9236")]
        port: u16,

        /// 自动接受
        #[arg(long, default_value = "true")]
        auto_accept: bool,
    },
    /// 运行完整演示（发送端+接收端）
    Full {
        /// 演示目录
        #[arg(short, long, default_value = "./demo_output")]
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
    },
    /// 测试文件完整性
    TestIntegrity {
        /// 测试文件路径
        #[arg(short, long)]
        file_path: PathBuf,

        /// 校验和算法
        #[arg(long, default_value = "sha256")]
        algorithm: String,
    },
}

/// P2P 演示管理器
pub struct P2PDemoManager {
    demo_dir: PathBuf,
}

impl P2PDemoManager {
    pub fn new(demo_dir: PathBuf) -> Self {
        Self { demo_dir }
    }

    /// 运行完整演示
    pub async fn run_full_demo(&self, 
                               shard_dir: PathBuf,
                               sender_port: u16,
                               receiver_port: u16) -> Result<()> {
        info!("🚀 开始 P2P 模型分发完整演示");
        info!("   分片目录: {}", shard_dir.display());
        info!("   演示目录: {}", self.demo_dir.display());
        info!("   发送端端口: {}", sender_port);
        info!("   接收端端口: {}", receiver_port);

        // 创建演示目录
        tokio::fs::create_dir_all(&self.demo_dir).await?;
        let receiver_output_dir = self.demo_dir.join("received");
        tokio::fs::create_dir_all(&receiver_output_dir).await?;

        // 步骤1: 验证源文件
        self.validate_source_files(&shard_dir).await?;

        // 步骤2: 启动接收端（后台）
        info!("📡 启动接收端...");
        let receiver_handle = self.start_receiver_background(
            "demo_receiver".to_string(),
            receiver_output_dir.clone(),
            receiver_port,
        ).await?;

        // 等待接收端启动
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 步骤3: 启动发送端
        info!("📤 启动发送端...");
        let sender_result = self.run_sender(
            "demo_sender".to_string(),
            "demo_receiver".to_string(),
            shard_dir,
            sender_port,
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
        tokio::time::sleep(Duration::from_secs(5)).await;

        // 步骤4: 验证接收的文件
        info!("🔍 验证接收的文件...");
        self.validate_received_files(&receiver_output_dir).await?;

        // 步骤5: 生成演示报告
        self.generate_demo_report(&receiver_output_dir).await?;

        info!("🎉 P2P 模型分发演示完成！");
        self.print_demo_summary(&receiver_output_dir).await;

        Ok(())
    }

    /// 验证源文件
    async fn validate_source_files(&self, shard_dir: &PathBuf) -> Result<()> {
        info!("🔍 验证源文件...");

        let mut entries = tokio::fs::read_dir(shard_dir).await?;
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
            return Err(anyhow!("未找到任何源文件"));
        }

        info!("✅ 源文件验证完成: {} 个文件, 总大小 {:.2} MB", 
              file_count, total_size as f64 / 1024.0 / 1024.0);

        Ok(())
    }

    /// 启动接收端（后台）
    async fn start_receiver_background(&self, 
                                        node_id: String,
                                        output_dir: PathBuf,
                                        port: u16) -> Result<tokio::task::JoinHandle<()>> {
        let receiver_args = P2PReceiverArgs {
            node_id,
            output_dir,
            port,
            bootstrap: None,
            auto_accept: true,
            max_concurrent: 5,
        };

        let handle = tokio::spawn(async move {
            if let Err(e) = run_receiver(receiver_args).await {
                error!("接收端错误: {}", e);
            }
        });

        Ok(handle)
    }

    /// 运行发送端
    async fn run_sender(&self, 
                        node_id: String,
                        target_peer: String,
                        shard_dir: PathBuf,
                        port: u16) -> Result<()> {
        let sender_args = P2PSenderArgs {
            node_id,
            target_peer,
            shard_dir,
            chunk_size: 1024 * 1024, // 1MB
            port,
            bootstrap: None,
        };

        run_sender(sender_args).await
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

    /// 生成演示报告
    async fn generate_demo_report(&self, received_dir: &PathBuf) -> Result<()> {
        info!("📋 生成演示报告...");

        let report_path = self.demo_dir.join("demo_report.json");
        let report = serde_json::json!({
            "demo_type": "p2p_model_distribution",
            "completed_at": chrono::Utc::now().to_rfc3339(),
            "received_files": self.get_file_list(received_dir).await?,
            "total_received_size": self.calculate_total_size(received_dir).await?,
            "success": true
        });

        tokio::fs::write(&report_path, serde_json::to_string_pretty(&report)?).await?;
        info!("📁 演示报告已保存: {}", report_path.display());

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

    /// 打印演示摘要
    async fn print_demo_summary(&self, received_dir: &PathBuf) {
        info!("📊 演示摘要:");
        info!("   演示目录: {}", self.demo_dir.display());
        info!("   接收目录: {}", received_dir.display());
        
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

    /// 测试文件完整性
    pub async fn test_file_integrity(&self, file_path: PathBuf, algorithm: String) -> Result<()> {
        info!("🔍 测试文件完整性: {}", file_path.display());
        info!("   算法: {}", algorithm);

        if !file_path.exists() {
            return Err(anyhow!("文件不存在: {}", file_path.display()));
        }

        let checksum_alg = match algorithm.to_lowercase().as_str() {
            "sha256" => ChecksumAlgorithm::SHA256,
            "sha512" => ChecksumAlgorithm::SHA512,
            "md5" => ChecksumAlgorithm::MD5,
            "blake3" => ChecksumAlgorithm::Blake3,
            _ => return Err(anyhow!("不支持的算法: {}", algorithm)),
        };

        let config = TransferProtocolConfig {
            checksum_algorithm: checksum_alg,
            ..Default::default()
        };

        let protocol = FileTransferProtocol::new(config);
        let integrity = protocol.calculate_file_integrity(&file_path).await?;

        info!("✅ 文件完整性计算完成:");
        info!("   文件大小: {} bytes", integrity.file_size);
        info!("   文件哈希: {}", integrity.sha256_hash);
        info!("   块数量: {}", integrity.chunk_hashes.len());

        // 验证完整性
        let is_valid = protocol.verify_file_integrity(&file_path, &integrity).await?;
        if is_valid {
            info!("✅ 文件完整性验证通过");
        } else {
            error!("❌ 文件完整性验证失败");
        }

        // 保存完整性信息
        let integrity_path = self.demo_dir.join("file_integrity.json");
        integrity.save_to_file(&integrity_path).await?;
        info!("📁 完整性信息已保存: {}", integrity_path.display());

        Ok(())
    }
}

/// 运行演示
pub async fn run_demo(args: P2PDemoArgs) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    match args.command {
        DemoCommand::Send { 
            node_id, 
            target_peer, 
            shard_dir, 
            chunk_size, 
            port 
        } => {
            let sender_args = P2PSenderArgs {
                node_id,
                target_peer,
                shard_dir,
                chunk_size,
                port,
                bootstrap: None,
            };
            run_sender(sender_args).await?;
        }
        DemoCommand::Receive { 
            node_id, 
            output_dir, 
            port, 
            auto_accept 
        } => {
            let receiver_args = P2PReceiverArgs {
                node_id,
                output_dir,
                port,
                bootstrap: None,
                auto_accept,
                max_concurrent: 5,
            };
            run_receiver(receiver_args).await?;
        }
        DemoCommand::Full { 
            demo_dir, 
            shard_dir, 
            sender_port, 
            receiver_port 
        } => {
            let manager = P2PDemoManager::new(demo_dir);
            manager.run_full_demo(shard_dir, sender_port, receiver_port).await?;
        }
        DemoCommand::TestIntegrity { 
            file_path, 
            algorithm 
        } => {
            let manager = P2PDemoManager::new(PathBuf::from("./demo_output"));
            manager.test_file_integrity(file_path, algorithm).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_demo_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = P2PDemoManager::new(temp_dir.path().to_path_buf());
        assert_eq!(manager.demo_dir, temp_dir.path());
    }

    #[tokio::test]
    async fn test_demo_args_parsing() {
        use clap::Parser;
        
        let args = P2PDemoArgs::try_parse_from(&[
            "p2p-demo",
            "test-integrity",
            "--file-path", "/tmp/test.txt",
            "--algorithm", "sha256"
        ]).unwrap();
        
        match args.command {
            DemoCommand::TestIntegrity { file_path, algorithm } => {
                assert_eq!(file_path, PathBuf::from("/tmp/test.txt"));
                assert_eq!(algorithm, "sha256");
            }
            _ => panic!("Expected TestIntegrity command"),
        }
    }
}
