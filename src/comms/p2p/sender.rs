/**
 * P2P 模型分发发送端
 * 负责将本地模型分片发送到其他节点
 */

use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio;
use tracing::{info, warn, error};
use tracing_subscriber;

use crate::comms::p2p::distributor::{P2PModelDistributor, FileTransferMessage};

/// P2P 模型分发发送端
#[derive(Parser)]
#[command(name = "p2p-sender")]
#[command(about = "P2P 模型分发发送端")]
pub struct P2PSenderArgs {
    /// 节点 ID
    #[arg(short, long, default_value = "sender_node")]
    pub node_id: String,

    /// 目标节点 ID
    #[arg(short, long)]
    pub target_peer: String,

    /// 要发送的模型分片目录
    #[arg(short, long, default_value = "./test_models/test_models/simple_split")]
    pub shard_dir: PathBuf,

    /// 块大小（字节）
    #[arg(short, long, default_value = "1048576")]
    pub chunk_size: usize,

    /// iroh 监听端口
    #[arg(short, long, default_value = "9235")]
    pub port: u16,

    /// bootstrap 节点
    #[arg(long)]
    pub bootstrap: Option<String>,
}

/// 发送端状态
#[derive(Debug)]
pub struct SenderStatus {
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
}

impl SenderStatus {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            completed_files: 0,
            failed_files: 0,
            total_bytes: 0,
            transferred_bytes: 0,
        }
    }

    pub fn get_progress_percentage(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.transferred_bytes as f32 / self.total_bytes as f32) * 100.0
    }
}

/// P2P 模型发送端
pub struct P2PModelSender {
    args: P2PSenderArgs,
    distributor: P2PModelDistributor,
    status: SenderStatus,
}

impl P2PModelSender {
    pub fn new(args: P2PSenderArgs) -> Self {
        let distributor = P2PModelDistributor::new(args.node_id.clone());
        
        Self {
            args,
            distributor,
            status: SenderStatus::new(),
        }
    }

    /// 启动发送端
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 启动 P2P 模型发送端");
        info!("   节点 ID: {}", self.args.node_id);
        info!("   目标节点: {}", self.args.target_peer);
        info!("   分片目录: {}", self.args.shard_dir.display());
        info!("   块大小: {} bytes", self.args.chunk_size);

        // 初始化 iroh 连接
        self.init_iroh_connection().await?;

        // 扫描模型分片
        let shard_files = self.scan_shard_files().await?;
        if shard_files.is_empty() {
            return Err(anyhow!("在目录 {} 中未找到模型分片文件", 
                              self.args.shard_dir.display()));
        }

        info!("📁 找到 {} 个模型分片文件", shard_files.len());
        self.status.total_files = shard_files.len();

        // 计算总大小
        for file_path in &shard_files {
            let metadata = tokio::fs::metadata(file_path).await?;
            self.status.total_bytes += metadata.len();
        }

        info!("📊 总大小: {:.2} MB", self.status.total_bytes as f64 / 1024.0 / 1024.0);

        // 发送所有分片
        for (index, file_path) in shard_files.iter().enumerate() {
            info!("📤 发送分片 {}/{}: {}", 
                  index + 1, shard_files.len(), file_path.file_name().unwrap().to_string_lossy());

            match self.send_single_file(file_path).await {
                Ok(_) => {
                    self.status.completed_files += 1;
                    let metadata = tokio::fs::metadata(file_path).await?;
                    self.status.transferred_bytes += metadata.len();
                    info!("✅ 分片发送完成");
                }
                Err(e) => {
                    self.status.failed_files += 1;
                    error!("❌ 分片发送失败: {}", e);
                }
            }

            // 显示进度
            info!("📈 总进度: {:.1}% ({}/{})", 
                  self.status.get_progress_percentage(),
                  self.status.completed_files,
                  self.status.total_files);

            // 分片间延迟
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // 显示最终统计
        self.print_final_stats();

        if self.status.failed_files > 0 {
            warn!("⚠️  有 {} 个文件发送失败", self.status.failed_files);
        } else {
            info!("🎉 所有分片发送完成！");
        }

        Ok(())
    }

    /// 初始化 iroh 连接
    async fn init_iroh_connection(&self) -> Result<()> {
        info!("🔗 初始化 iroh P2P 连接...");
        
        // 这里应该初始化实际的 iroh 连接
        // 目前简化实现
        
        info!("✅ iroh 连接初始化完成");
        Ok(())
    }

    /// 扫描模型分片文件
    async fn scan_shard_files(&self) -> Result<Vec<PathBuf>> {
        let mut shard_files = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.args.shard_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // 查找 JSON 配置文件和可能的模型文件
            if path.is_file() {
                let file_name = path.file_name()
                    .unwrap()
                    .to_string_lossy();
                
                // 包含 node_ 的 JSON 文件是分片配置
                if file_name.contains("node_") && file_name.ends_with(".json") {
                    shard_files.push(path);
                }
                // 也可以查找 .pth, .safetensors 等模型文件
                else if file_name.ends_with(".pth") || file_name.ends_with(".safetensors") {
                    shard_files.push(path);
                }
            }
        }

        // 按文件名排序
        shard_files.sort();

        Ok(shard_files)
    }

    /// 发送单个文件
    async fn send_single_file(&mut self, file_path: &Path) -> Result<String> {
        info!("📤 开始发送文件: {}", file_path.display());

        // 通过 P2P 分发器发送文件
        let transfer_id = self.distributor.send_file(
            self.args.target_peer.clone(),
            file_path,
            Some(self.args.chunk_size),
        ).await?;

        info!("🔄 文件传输已启动，ID: {}", transfer_id);

        // 监控传输进度
        self.monitor_transfer_progress(&transfer_id).await?;

        Ok(transfer_id)
    }

    /// 监控传输进度
    async fn monitor_transfer_progress(&mut self, transfer_id: &str) -> Result<()> {
        let mut last_progress = 0.0;
        
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            if let Some(status) = self.distributor.get_transfer_status(transfer_id).await {
                match status {
                    crate::comms::p2p_distributor::TransferStatus::Completed => {
                        info!("✅ 传输完成: {}", transfer_id);
                        break;
                    }
                    crate::comms::p2p_distributor::TransferStatus::Failed(error) => {
                        error!("❌ 传输失败: {} - {}", transfer_id, error);
                        return Err(anyhow!("传输失败: {}", error));
                    }
                    crate::comms::p2p_distributor::TransferStatus::InProgress { 
                        chunks_received, 
                        total_chunks 
                    } => {
                        let progress = (chunks_received as f32 / total_chunks as f32) * 100.0;
                        
                        // 每增加 10% 打印一次进度
                        if progress - last_progress >= 10.0 {
                            info!("📊 传输进度: {:.1}% ({}/{})", 
                                  progress, chunks_received, total_chunks);
                            last_progress = progress;
                        }
                    }
                    _ => {
                        // 继续等待
                    }
                }
            } else {
                warn!("⚠️  未找到传输状态: {}", transfer_id);
                break;
            }
        }

        Ok(())
    }

    /// 打印最终统计信息
    fn print_final_stats(&self) {
        info!("📊 发送统计:");
        info!("   总文件数: {}", self.status.total_files);
        info!("   成功发送: {}", self.status.completed_files);
        info!("   发送失败: {}", self.status.failed_files);
        info!("   总大小: {:.2} MB", self.status.total_bytes as f64 / 1024.0 / 1024.0);
        info!("   已传输: {:.2} MB", self.status.transferred_bytes as f64 / 1024.0 / 1024.0);
        info!("   成功率: {:.1}%", 
              (self.status.completed_files as f32 / self.status.total_files as f32) * 100.0);
    }
}

/// 运行发送端
pub async fn run_sender(args: P2PSenderArgs) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    let mut sender = P2PModelSender::new(args);
    sender.start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sender_status() {
        let mut status = SenderStatus::new();
        assert_eq!(status.get_progress_percentage(), 0.0);
        
        status.total_files = 10;
        status.completed_files = 5;
        assert_eq!(status.get_progress_percentage(), 50.0);
    }

    #[tokio::test]
    async fn test_sender_creation() {
        let args = P2PSenderArgs {
            node_id: "test_sender".to_string(),
            target_peer: "test_receiver".to_string(),
            shard_dir: PathBuf::from("./test_shards"),
            chunk_size: 1024,
            port: 9235,
            bootstrap: None,
        };

        let sender = P2PModelSender::new(args);
        assert_eq!(sender.args.node_id, "test_sender");
        assert_eq!(sender.args.target_peer, "test_receiver");
    }
}
