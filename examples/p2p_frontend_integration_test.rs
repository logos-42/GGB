/**
 * P2P 前端集成测试
 * 验证前端桌面应用的 P2P 功能集成
 */

use std::time::Duration;
use tokio;
use tracing::{info, warn};
use anyhow::Result;

// 导入 P2P 前端模块
use williw::comms::p2p_frontend_starter::{auto_initialize_p2p_service, get_global_p2p_starter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧪 开始 P2P 前端集成测试");

    // 测试 1: 自动初始化 P2P 服务
    info!("📋 测试 1: 自动初始化 P2P 服务");
    match auto_initialize_p2p_service().await {
        Ok(_) => info!("✅ P2P 服务初始化成功"),
        Err(e) => {
            warn!("❌ P2P 服务初始化失败: {}", e);
            return Err(e);
        }
    }

    // 获取全局启动器
    let starter = get_global_p2p_starter().await;

    // 测试 2: 获取本地节点 ID
    info!("📋 测试 2: 获取本地节点 ID");
    match starter.get_local_node_id().await {
        Ok(node_id) => {
            info!("✅ 本地节点 ID: {}", node_id);
            info!("📋 您可以将此 ID 分享给其他节点进行连接");
        }
        Err(e) => {
            warn!("❌ 获取节点 ID 失败: {}", e);
        }
    }

    // 测试 3: 获取前端状态
    info!("📋 测试 3: 获取前端状态");
    match starter.get_frontend_state().await {
        Ok(state) => {
            info!("✅ 前端状态获取成功");
            info!("   - 本地节点: {}", state.local_node.node_id);
            info!("   - 节点类型: {:?}", state.local_node.node_type);
            info!("   - 连接状态: {:?}", state.local_node.status);
            info!("   - 活跃连接: {}", state.connection_stats.active_connections);
            info!("   - 总连接数: {}", state.connection_stats.total_connections);
        }
        Err(e) => {
            warn!("❌ 获取前端状态失败: {}", e);
        }
    }

    // 测试 4: 添加远程节点
    info!("📋 测试 4: 添加远程节点");
    let test_node_id = "12D3KooWTestNode123456789012345678901234567890123456789012345678901234567890";
    let test_addresses = vec![
        "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWTestNode123456789012345678901234567890123456789012345678901234567890".to_string(),
    ];

    match starter.add_remote_node(test_node_id.to_string(), test_addresses).await {
        Ok(_) => info!("✅ 远程节点添加成功"),
        Err(e) => warn!("❌ 添加远程节点失败: {}", e),
    }

    // 等待一段时间让节点连接
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 测试 5: 获取连接的节点
    info!("📋 测试 5: 获取连接的节点");
    match starter.get_frontend_state().await {
        Ok(state) => {
            info!("✅ 获取连接节点成功，总数: {}", state.connected_nodes.len());
            for (i, node) in state.connected_nodes.iter().enumerate() {
                info!("   节点 {}: {} - {:?}", i + 1, node.node_id[..20].to_string(), node.status);
            }
        }
        Err(e) => {
            warn!("❌ 获取连接节点失败: {}", e);
        }
    }

    // 测试 6: 获取连接统计
    info!("📋 测试 6: 获取连接统计");
    match starter.get_frontend_state().await {
        Ok(state) => {
            let stats = state.connection_stats;
            info!("✅ 连接统计获取成功");
            info!("   - 活跃连接: {}", stats.active_connections);
            info!("   - 总连接数: {}", stats.total_connections);
            info!("   - 上传速度: {:.2} KB/s", stats.upload_speed / 1024.0);
            info!("   - 下载速度: {:.2} KB/s", stats.download_speed / 1024.0);
        }
        Err(e) => {
            warn!("❌ 获取连接统计失败: {}", e);
        }
    }

    // 测试 7: 复制节点 ID
    info!("📋 测试 7: 复制节点 ID");
    match starter.copy_node_id().await {
        Ok(_) => info!("✅ 节点 ID 复制成功"),
        Err(e) => warn!("❌ 复制节点 ID 失败: {}", e),
    }

    // 测试 8: 移除节点
    info!("📋 测试 8: 移除节点");
    match starter.remove_node(test_node_id).await {
        Ok(_) => info!("✅ 节点移除成功"),
        Err(e) => warn!("❌ 移除节点失败: {}", e),
    }

    // 运行一段时间以观察后台任务
    info!("📋 运行 10 秒以观察后台任务...");
    tokio::time::sleep(Duration::from_secs(10)).await;

    // 测试 9: 停止 P2P 服务
    info!("📋 测试 9: 停止 P2P 服务");
    match starter.shutdown().await {
        Ok(_) => info!("✅ P2P 服务停止成功"),
        Err(e) => warn!("❌ 停止 P2P 服务失败: {}", e),
    }

    info!("🎉 P2P 前端集成测试完成！");
    Ok(())
}
