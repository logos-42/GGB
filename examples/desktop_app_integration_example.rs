/**
 * 桌面应用集成示例
 * 展示如何在桌面应用启动时自动集成 P2P 功能
 */

use std::time::Duration;
use tokio;
use tracing::{info, warn};
use anyhow::Result;

// 导入 P2P 应用集成模块
use williw::comms::p2p_app_integration::{P2PEnabledApp, P2PAppFactory};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🖥️  启动 P2P 桌面应用示例");

    // 方式1：使用工厂模式创建应用
    let app = P2PAppFactory::create_default();
    
    // 或者使用自定义配置
    // let app = P2PAppFactory::create_custom(
    //     "我的 P2P 应用".to_string(),
    //     "2.0.0".to_string(),
    // );

    // 启动应用（包含 P2P 服务初始化）
    info!("🚀 启动应用...");
    app.start().await?;

    info!("✅ 应用启动完成！");
    info!("📋 P2P 功能已自动集成并启动");
    info!("🔑 您可以在前端界面中查看和管理 P2P 连接");

    // 模拟应用运行
    info!("🔄 应用正在运行中...");
    
    // 设置运行时间（例如：运行 30 秒）
    let runtime = Duration::from_secs(30);
    let mut elapsed = Duration::from_secs(0);
    
    while elapsed < runtime {
        tokio::time::sleep(Duration::from_secs(5)).await;
        elapsed += Duration::from_secs(5);
        
        let remaining = runtime - elapsed;
        info!("⏱️  应用运行中... 剩余时间: {} 秒", remaining.as_secs());
        
        // 这里可以添加应用的主要业务逻辑
        // 例如：处理用户请求、更新界面等
    }

    info!("🛑 应用运行时间结束，准备关闭...");
    
    // 注意：在实际应用中，您可能不需要手动关闭
    // 应用会通过 Ctrl+C 信号自动处理关闭逻辑
    
    // 模拟关闭过程
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    info!("🎉 桌面应用示例运行完成！");
    
    Ok(())
}

/// 快速启动示例
pub async fn quick_start_example() -> Result<()> {
    info!("🚀 快速启动示例");
    
    // 使用快速启动函数
    // 注意：这个函数会阻塞，所以在实际使用中需要小心
    // williw::comms::p2p_app_integration::quick_start().await?;
    
    info!("✅ 快速启动示例完成");
    Ok(())
}

/// 带配置的启动示例
pub async fn custom_config_example() -> Result<()> {
    info!("⚙️  自定义配置启动示例");
    
    // 使用自定义配置启动
    // williw::comms::p2p_app_integration::start_with_config(
    //     "自定义 P2P 应用".to_string(),
    //     "3.0.0".to_string(),
    // ).await?;
    
    info!("✅ 自定义配置启动示例完成");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() -> Result<()> {
        let app = P2PAppFactory::create_default();
        assert_eq!(app.app_name, "Williw P2P 模型分发");
        assert_eq!(app.version, "1.0.0");
        Ok(())
    }

    #[tokio::test]
    async fn test_custom_app_creation() -> Result<()> {
        let app = P2PAppFactory::create_custom(
            "测试应用".to_string(),
            "1.0.0".to_string(),
        );
        assert_eq!(app.app_name, "测试应用");
        assert_eq!(app.version, "1.0.0");
        Ok(())
    }
}
