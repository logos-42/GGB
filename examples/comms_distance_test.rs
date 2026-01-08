//! 通信模块网络距离感知测试
//!
//! 测试通信模块中网络距离感知功能的集成

use williw::comms::iroh::QuicGateway;
use williw::types::NetworkDistance;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 通信模块网络距离感知测试 ===");
    
    // 创建一个示例地址（实际使用时需要真实的地址）
    let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
    
    // 创建网关实例
    let gateway = QuicGateway::new(bind_addr)?;
    
    // 测试获取本地 DERP 延迟
    let local_delays = gateway.get_local_derp_delays().await;
    println!("本地 DERP 节点延迟: {:?}", local_delays);
    
    // 测试获取网络报告
    let net_report = gateway.get_net_report().await;
    if let Some(report) = net_report {
        println!("网络报告包含 {} 个 DERP 节点", report.relay_latency.len());
    } else {
        println!("无法获取网络报告");
    }
    
    println!("=== 测试完成 ===");
    
    Ok(())
}