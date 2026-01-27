/**
 * P2P 模型分发接收端
 * 负责接收其他节点发送的模型分片
 */

use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio;
use tracing::{info, warn, error};
use tracing_subscriber;

use crate::comms::p2p_distributor::{P2PModelDistributor, FileTransferMessage};

/// P2P 模型分发接收端
#[derive(Parser)]
#[command(name = "p2p-receiver")]
#[command(about = "P2P 模型分发接收端")]
pub struct P2PReceiverArgs {
    /// 节点 ID
    #[arg(short, long, default_value = "receiver_node")]
    pub node_id: String,

    /// 接收文件的输出目录
    #[arg(short, long, default_value = "./received_models")]
    pub output_dir: PathBuf,

    /// iroh 监听端口
    #[arg(short, long, default_value = "9236")]
    pub port: u16,

    /// bootstrap 节点
    #[arg(long)]
    pub bootstrap: Option<String>,

    /// 自动接受传输
    #[arg(long, default_value = "true")]
    pub auto_accept: bool,

    /// 最大并发传输数
    #[arg(long, default_value = "5")]
    pub max_concurrent: usize,
}

/// 接收端统计信息
#[derive(Debug)]
pub struct ReceiverStats {
    pub total_requests: usize,
    pub accepted_transfers: usize,
    pub rejected_transfers: usize,
    pub completed_transfers: usize,
    pub failed_transfers: usize,
    pub total_bytes_received: u64,
}

impl ReceiverStats {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            accepted_transfers: 0,
            rejected_transfers: 0,
            completed_transfers: 0,
            failed_transfers: 0,
            total_bytes_received: 0,
        }
    }

    pub fn get_success_rate(&self) -> f32 {
        if self.accepted_transfers == 0 {
            return 0.0;
        }
        (self.completed_transfers as f32 / self.accepted_transfers as f32) * 100.0
    }
}

/// P2P 模型接收端
pub struct P2PModelReceiver {
    args: P2PReceiverArgs,
    distributor: P2PModelDistributor,
    stats: ReceiverStats,
    is_running: bool,
}

impl P2PModelReceiver {
    pub fn new(args: P2PReceiverArgs) -> Self {
        let distributor = P2PModelDistributor::new(args.node_id.clone());
        
        Self {
            args,
            distributor,
            stats: ReceiverStats::new(),
            is_running: false,
        }
    }

    /// 启动接收端
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 启动 P2P 模型接收端");
        info!("   节点 ID: {}", self.args.node_id);
        info!("   输出目录: {}", self.args.output_dir.display());
        info!("   监听端口: {}", self.args.port);
        info!("   自动接受: {}", self.args.auto_accept);
        info!("   最大并发: {}", self.args.max_concurrent);

        // 创建输出目录
        tokio::fs::create_dir_all(&self.args.output_dir).await?;
        info!("📁 输出目录已创建");

        // 初始化 iroh 连接
        self.init_iroh_connection().await?;

        self.is_running = true;
        info!("✅ 接收端已启动，等待传入的文件...");

        // 开始监听消息
        self.message_loop().await?;

        Ok(())
    }

    /// 停止接收端
    pub async fn stop(&mut self) {
        info!("🛑 停止接收端...");
        self.is_running = false;
    }

    /// 初始化 iroh 连接
    async fn init_iroh_connection(&self) -> Result<()> {
        info!("🔗 初始化 iroh P2P 连接...");
        
        // 这里应该初始化实际的 iroh 连接
        // 目前简化实现
        
        info!("✅ iroh 连接初始化完成");
        Ok(())
    }

    /// 主消息循环
    async fn message_loop(&mut self) -> Result<()> {
        while self.is_running {
            // 接收传入的消息
            if let Some((sender_id, message)) = self.receive_message().await? {
                self.handle_message(sender_id, message).await?;
            }

            // 清理已完成的传输
            self.distributor.cleanup_completed_transfers().await;

            // 短暂休眠避免CPU占用过高
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// 接收消息（模拟实现）
    async fn receive_message(&self) -> Result<Option<(String, FileTransferMessage)>> {
        // 这里应该通过 iroh 接收实际消息
        // 目前模拟实现，返回 None
        tokio::time::sleep(Duration::from_millis(1000)).await;
        Ok(None)
    }

    /// 处理接收到的消息
    async fn handle_message(&mut self, sender_id: String, message: FileTransferMessage) -> Result<()> {
        match message {
            FileTransferMessage::FileRequest { 
                file_id, 
                file_name, 
                file_size, 
                chunk_size, 
                file_hash 
            } => {
                self.handle_file_request(sender_id, file_id, file_name, file_size, chunk_size, file_hash).await?;
            }
            FileTransferMessage::FileChunk { 
                file_id, 
                chunk_index, 
                data, 
                chunk_hash 
            } => {
                self.handle_file_chunk(sender_id, file_id, chunk_index, data, chunk_hash).await?;
            }
            FileTransferMessage::FileComplete { 
                file_id, 
                total_chunks, 
                final_hash 
            } => {
                self.handle_file_complete(sender_id, file_id, total_chunks, final_hash).await?;
            }
            FileTransferMessage::TransferError { file_id, error } => {
                self.handle_transfer_error(sender_id, file_id, error).await?;
            }
            _ => {
                warn!("⚠️  收到未知类型的消息");
            }
        }

        Ok(())
    }

    /// 处理文件传输请求
    async fn handle_file_request(&mut self, 
                                 sender_id: String,
                                 file_id: String,
                                 file_name: String,
                                 file_size: u64,
                                 chunk_size: usize,
                                 file_hash: String) -> Result<()> {
        self.stats.total_requests += 1;
        
        info!("📥 收到文件传输请求:");
        info!("   发送方: {}", sender_id);
        info!("   文件名: {}", file_name);
        info!("   文件大小: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
        info!("   文件哈希: {}", file_hash);

        // 检查是否自动接受
        if self.args.auto_accept {
            info!("✅ 自动接受传输");
            self.accept_transfer(sender_id, file_id, file_name, file_size, chunk_size, file_hash).await?;
        } else {
            // 这里可以实现交互式确认
            info!("⏳ 等待用户确认传输...");
            // 暂时自动接受
            self.accept_transfer(sender_id, file_id, file_name, file_size, chunk_size, file_hash).await?;
        }

        Ok(())
    }

    /// 接受文件传输
    async fn accept_transfer(&mut self,
                             sender_id: String,
                             file_id: String,
                             file_name: String,
                             file_size: u64,
                             chunk_size: usize,
                             file_hash: String) -> Result<()> {
        self.stats.accepted_transfers += 1;

        // 创建文件请求消息
        let file_request = FileTransferMessage::FileRequest {
            file_id: file_id.clone(),
            file_name: file_name.clone(),
            file_size,
            chunk_size,
            file_hash: file_hash.clone(),
        };

        // 开始接收文件
        let transfer_id = self.distributor.receive_file(&self.args.output_dir, file_request).await?;
        
        info!("🔄 开始接收文件，传输ID: {}", transfer_id);

        // 发送接受响应
        let response = FileTransferMessage::FileResponse {
            file_id: transfer_id.clone(),
            accepted: true,
            reason: None,
        };

        self.send_message(&sender_id, response).await?;

        Ok(())
    }

    /// 处理文件块
    async fn handle_file_chunk(&mut self,
                               sender_id: String,
                               file_id: String,
                               chunk_index: u32,
                               data: Vec<u8>,
                               chunk_hash: String) -> Result<()> {
        let chunk_message = FileTransferMessage::FileChunk {
            file_id: file_id.clone(),
            chunk_index,
            data,
            chunk_hash,
        };

        self.distributor.handle_file_chunk(sender_id, chunk_message).await?;

        // 检查传输进度
        if let Some(status) = self.distributor.get_transfer_status(&file_id).await {
            match status {
                crate::comms::p2p_distributor::TransferStatus::InProgress { 
                    chunks_received, 
                    total_chunks 
                } => {
                    let progress = (chunks_received as f32 / total_chunks as f32) * 100.0;
                    info!("📊 文件 {} 接收进度: {:.1}%", file_id, progress);
                }
                crate::comms::p2p_distributor::TransferStatus::Completed => {
                    self.stats.completed_transfers += 1;
                    info!("✅ 文件 {} 接收完成", file_id);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 处理文件传输完成
    async fn handle_file_complete(&mut self,
                                  sender_id: String,
                                  file_id: String,
                                  total_chunks: u32,
                                  final_hash: String) -> Result<()> {
        info!("🎉 文件传输完成:");
        info!("   文件ID: {}", file_id);
        info!("   总块数: {}", total_chunks);
        info!("   最终哈希: {}", final_hash);

        // 更新统计
        if let Some(status) = self.distributor.get_transfer_status(&file_id).await {
            match status {
                crate::comms::p2p_distributor::TransferStatus::Completed => {
                    self.stats.completed_transfers += 1;
                    
                    // 尝试获取文件大小
                    if let Some(session) = self.get_transfer_session(&file_id).await {
                        self.stats.total_bytes_received += session.file_size;
                    }
                }
                _ => {}
            }
        }

        // 发送确认消息
        let ack_message = FileTransferMessage::FileComplete {
            file_id: file_id.clone(),
            total_chunks,
            final_hash,
        };

        self.send_message(&sender_id, ack_message).await?;

        self.print_stats();

        Ok(())
    }

    /// 处理传输错误
    async fn handle_transfer_error(&mut self,
                                    sender_id: String,
                                    file_id: String,
                                    error: String) -> Result<()> {
        self.stats.failed_transfers += 1;
        error!("❌ 传输失败: {} - {}", file_id, error);
        
        // 可以选择发送错误确认
        Ok(())
    }

    /// 发送消息（模拟实现）
    async fn send_message(&mut self, peer_id: &str, message: FileTransferMessage) -> Result<()> {
        // 这里应该通过 iroh 发送实际消息
        let _ = (peer_id, message);
        Ok(())
    }

    /// 获取传输会话
    async fn get_transfer_session(&self, file_id: &str) -> Option<crate::comms::p2p_distributor::TransferSession> {
        // 这里需要访问 distributor 的内部状态
        // 暂时返回 None
        None
    }

    /// 打印统计信息
    fn print_stats(&self) {
        info!("📊 接收统计:");
        info!("   总请求数: {}", self.stats.total_requests);
        info!("   已接受: {}", self.stats.accepted_transfers);
        info!("   已完成: {}", self.stats.completed_transfers);
        info!("   失败: {}", self.stats.failed_transfers);
        info!("   成功率: {:.1}%", self.stats.get_success_rate());
        info!("   已接收: {:.2} MB", self.stats.total_bytes_received as f64 / 1024.0 / 1024.0);
    }

    /// 获取当前统计信息
    pub fn get_stats(&self) -> &ReceiverStats {
        &self.stats
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// 运行接收端
pub async fn run_receiver(args: P2PReceiverArgs) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    let mut receiver = P2PModelReceiver::new(args);
    
    // 设置 Ctrl+C 处理
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let receiver_arc = Arc::new(Mutex::new(receiver));
    let receiver_clone = receiver_arc.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        let mut receiver = receiver_clone.lock().await;
        receiver.stop().await;
    });
    
    let mut receiver = receiver_arc.lock().await;
    
    receiver.start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_receiver_stats() {
        let mut stats = ReceiverStats::new();
        assert_eq!(stats.get_success_rate(), 0.0);
        
        stats.accepted_transfers = 10;
        stats.completed_transfers = 8;
        assert_eq!(stats.get_success_rate(), 80.0);
    }

    #[tokio::test]
    async fn test_receiver_creation() {
        let args = P2PReceiverArgs {
            node_id: "test_receiver".to_string(),
            output_dir: PathBuf::from("./received"),
            port: 9236,
            bootstrap: None,
            auto_accept: true,
            max_concurrent: 5,
        };

        let receiver = P2PModelReceiver::new(args);
        assert_eq!(receiver.args.node_id, "test_receiver");
        assert_eq!(receiver.args.port, 9236);
        assert!(!receiver.is_running());
    }
}
