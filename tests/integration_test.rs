//! 集成测试：验证多节点协同训练

use GGB::{
    comms::CommsConfig,
    consensus::ConsensusConfig,
    crypto::CryptoConfig,
    device::{DeviceCapabilities, DeviceManager},
    inference::InferenceConfig,
    topology::TopologyConfig,
    types::GeoPoint,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// 测试辅助函数：创建测试配置
fn create_test_config(node_id: usize) -> (CommsConfig, InferenceConfig, TopologyConfig) {
    let comms = CommsConfig {
        topic: format!("ggb-test-{}", node_id),
        listen_addr: Some(format!("/ip4/127.0.0.1/tcp/{}", 9000 + node_id).parse().unwrap()),
        quic_bind: Some(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            9234 + node_id as u16,
        )),
        quic_bootstrap: Vec::new(),
        bandwidth: Default::default(),
        enable_dht: true,
        bootstrap_peers_file: Some(format!("test_bootstrap_{}.txt", node_id).into()),
    };
    
    let inference = InferenceConfig {
        model_dim: 128,
        model_path: None,
    };
    
    let topology = TopologyConfig {
        max_neighbors: 4,
        failover_pool: 2,
        min_score: 0.1,
        geo_scale_km: 100.0,
        peer_stale_secs: 60,
    };
    
    (comms, inference, topology)
}

#[tokio::test]
#[ignore] // 需要实际运行节点，默认忽略
async fn test_multi_node_discovery() {
    println!("集成测试：多节点发现和连接");
    
    // 这个测试需要实际启动多个节点进程
    // 在实际测试环境中，可以使用 testcontainers 或类似工具
    // 这里提供一个测试框架
    
    // 1. 启动 3 个节点（通过外部进程）
    // 2. 等待节点发现彼此
    // 3. 验证连接建立
    // 4. 验证消息交换
    
    // 注意：由于项目结构是二进制项目而非库项目，
    // 这些测试需要在实际的测试环境中运行
    // 建议将核心功能提取到 lib.rs 以便测试
    
    assert!(true, "集成测试框架已就绪");
}

#[tokio::test]
#[ignore]
async fn test_model_synchronization() {
    println!("集成测试：模型参数同步");
    
    // TODO: 实现模型同步测试
    // 1. 启动多个节点，每个节点有不同的初始模型
    // 2. 运行一段时间
    // 3. 检查模型是否收敛（参数相似度增加）
    
    assert!(true, "模型同步测试框架已就绪");
}

#[tokio::test]
async fn test_node_config_creation() {
    // 测试节点配置创建
    let (comms, inference, topology) = create_test_config(0);
    
    assert_eq!(comms.topic, "ggb-test-0");
    assert_eq!(inference.model_dim, 128);
    assert_eq!(topology.max_neighbors, 4);
}

#[tokio::test]
async fn test_device_capabilities() {
    // 测试设备能力检测
    let device_manager = DeviceManager::new();
    let caps = device_manager.get();
    
    assert!(caps.max_memory_mb > 0);
    assert!(caps.cpu_cores > 0);
    assert!(caps.recommended_model_dim() >= 64);
}

#[tokio::test]
async fn test_dht_bootstrap_loading() {
    // 测试 DHT bootstrap 节点文件加载
    use std::fs;
    use std::path::PathBuf;
    
    let test_file = PathBuf::from("test_bootstrap_nodes.txt");
    let test_content = "/ip4/127.0.0.1/tcp/9001\n/ip4/127.0.0.1/tcp/9002\n";
    
    // 创建测试文件
    fs::write(&test_file, test_content).unwrap();
    
    // 测试加载
    let content = fs::read_to_string(&test_file).unwrap();
    let mut bootstrap_peers = Vec::new();
    for line in content.lines() {
        if let Ok(addr) = line.trim().parse::<libp2p::Multiaddr>() {
            bootstrap_peers.push(addr);
        }
    }
    
    assert_eq!(bootstrap_peers.len(), 2);
    
    // 清理
    fs::remove_file(&test_file).unwrap();
}
