/**
 * P2P 前端集成示例
 * 展示如何在桌面应用启动时自动初始化 P2P 服务
 */

use anyhow::{Result, anyhow};
use std::time::Duration;
use tokio;
use tracing::{info, warn, error};
use tracing_subscriber;

// 导入 P2P 前端模块
use crate::comms::frontend::starter::P2PFrontendStarter;

/// 应用主结构
pub struct P2PEnabledApp {
    app_name: String,
    version: String,
}

/// 应用工厂
pub struct P2PAppFactory;

impl P2PAppFactory {
    /// 创建默认配置的应用
    pub fn create_default() -> P2PEnabledApp {
        P2PEnabledApp::new("Williw P2P 模型分发".to_string(), "1.0.0".to_string())
    }
    
    /// 创建自定义配置的应用
    pub fn create_custom(name: String, version: String) -> P2PEnabledApp {
        P2PEnabledApp::new(name, version)
    }
}

/// 应用集成器
pub struct P2PAppIntegration {
    app: P2PEnabledApp,
}

impl P2PAppIntegration {
    /// 创建新的集成器
    pub fn new(app: P2PEnabledApp) -> Self {
        Self { app }
    }
    
    /// 初始化应用
    pub async fn initialize(&self) -> Result<()> {
        self.app.init_logging().await?;
        self.app.init_p2p_service().await?;
        Ok(())
    }
    
    /// 运行应用
    pub async fn run(&self) -> Result<()> {
        self.app.run().await
    }
}

impl P2PEnabledApp {
    /// 创建新的应用实例
    pub fn new(app_name: String, version: String) -> Self {
        Self {
            app_name,
            version,
        }
    }

    /// 启动应用（包含 P2P 服务初始化）
    pub async fn start(&self) -> Result<()> {
        info!("🚀 启动应用: {} v{}", self.app_name, self.version);
        
        // 初始化日志系统
        self.init_logging().await?;
        
        // 初始化 P2P 服务
        self.init_p2p_service().await?;
        
        // 显示节点状态
        self.display_node_status().await?;
        
        Ok(())
    }
    
    /// 初始化日志系统
    async fn init_logging(&self) -> Result<()> {
        // 尝试初始化日志系统，如果已经初始化则跳过
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        info!("📝 日志系统已初始化");
        Ok(())
    }

    /// 初始化 P2P 服务
    async fn init_p2p_service(&self) -> Result<()> {
        info!("🌐 初始化 P2P 服务...");

        // 自动初始化 P2P 服务
        let starter = P2PFrontendStarter::new();
        match starter.initialize().await {
            Ok(_) => {
                info!("✅ P2P 服务初始化成功");
                
                // 获取本地节点 ID 并显示
                match starter.get_local_node_id().await {
                    Ok(node_id) => {
                        info!("🔑 本地节点 ID: {}", node_id);
                        info!("📋 您可以将此 ID 分享给其他节点进行连接");
                    }
                    Err(e) => {
                        warn!("获取节点 ID 失败: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ P2P 服务初始化失败: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 显示节点状态
    async fn display_node_status(&self) -> Result<()> {
        let starter = P2PFrontendStarter::new();
        
        match starter.get_frontend_state().await {
            Ok(state) => {
                info!("📊 节点状态:");
                info!("   - 本地节点: {}", state.local_node.node_id);
                info!("   - 节点类型: {:?}", state.local_node.node_type);
                info!("   - 连接状态: {:?}", state.local_node.status);
                info!("   - 活跃连接: {}", state.connection_stats.active_connections);
                info!("   - 总连接数: {}", state.connection_stats.total_connections);
                
                // 显示连接的节点列表
                if !state.connected_nodes.is_empty() {
                    info!("   - 连接的节点:");
                    for node in &state.connected_nodes {
                        info!("     * {} ({:?})", node.node_id, node.node_type);
                    }
                } else {
                    info!("   - 当前无连接的节点");
                }
            }
            Err(e) => {
                warn!("获取节点状态失败: {}", e);
            }
        }

        Ok(())
    }

    /// 运行应用
    pub async fn run(&self) -> Result<()> {
        info!("🔄 应用进入运行状态...");

        // 设置 Ctrl+C 处理
        let app_name = self.app_name.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            info!("🛑 收到停止信号，正在关闭 {}...", app_name);
            
            // 停止 P2P 服务
            let starter = P2PFrontendStarter::new();
            if let Err(e) = starter.shutdown().await {
                error!("停止 P2P 服务失败: {}", e);
            }
            
            std::process::exit(0);
        });

        // 保持应用运行
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

/// 快速启动函数
pub async fn quick_start() -> Result<()> {
    let app = P2PAppFactory::create_default();
    app.start().await?;
    app.run().await
}

/// 带配置的启动函数
pub async fn start_with_config(name: String, version: String) -> Result<()> {
    let app = P2PAppFactory::create_custom(name, version);
    app.start().await?;
    app.run().await
}

/// 示例：如何在 main 函数中使用
#[cfg(not(test))]
pub mod example {
    use super::*;

    /// 示例主函数
    pub async fn main_example() -> Result<()> {
        // 方式1：快速启动（使用默认配置）
        quick_start().await?;

        // 方式2：自定义配置启动
        start_with_config(
            "我的 P2P 应用".to_string(),
            "1.0.0".to_string(),
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_enabled_app() -> Result<()> {
        let app = P2PAppFactory::create_default();
        
        // 测试应用创建
        assert_eq!(app.app_name, "Williw P2P 模型分发");
        assert_eq!(app.version, "1.0.0");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_app_factory() -> Result<()> {
        let app1 = P2PAppFactory::create_default();
        let app2 = P2PAppFactory::create_custom(
            "Test App".to_string(),
            "2.0.0".to_string(),
        );
        
        assert_eq!(app1.app_name, "Williw P2P 模型分发");
        assert_eq!(app1.version, "1.0.0");
        
        assert_eq!(app2.app_name, "Test App");
        assert_eq!(app2.version, "2.0.0");
        
        Ok(())
    }
}
