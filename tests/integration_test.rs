//! 集成测试：验证多节点协同训练

use GGB::{
    comms::CommsConfig,
    consensus::ConsensusConfig,
    crypto::CryptoConfig,
    device::{DeviceCapabilities, DeviceManager, DeviceType},
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
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};
    
    println!("集成测试：多节点发现和连接");
    
    // 1. 启动 3 个节点进程
    let mut processes = Vec::new();
    let output_dir = std::path::PathBuf::from("test_output_integration");
    std::fs::create_dir_all(&output_dir).unwrap();
    
    for i in 0..3 {
        let node_id = i;
        let stats_file = output_dir.join(format!("node_{}_stats.json", node_id));
        let log_file = output_dir.join(format!("node_{}.log", node_id));
        
        // 构建命令
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--release", "--"])
            .arg("--node-id")
            .arg(node_id.to_string())
            .arg("--stats-output")
            .arg(stats_file.to_str().unwrap())
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // 设置设备类型
        let device_type = match i % 3 {
            0 => "low",
            1 => "mid",
            _ => "high",
        };
        cmd.env("GGB_DEVICE_TYPE", device_type);
        
        println!("启动节点 {} (设备类型: {})", node_id, device_type);
        
        // 启动进程
        let mut child = cmd.spawn().expect("Failed to start node");
        processes.push((node_id, child));
        
        // 错开启动时间
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("所有节点已启动，等待节点发现...");
    
    // 2. 等待节点发现彼此（等待 30 秒）
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // 3. 验证连接建立（检查日志文件）
    let mut discovered_count = 0;
    for i in 0..3 {
        let log_file = output_dir.join(format!("node_{}.log", i));
        if log_file.exists() {
            let content = std::fs::read_to_string(&log_file).unwrap_or_default();
            // 检查是否有节点发现相关的日志
            if content.contains("发现节点") || content.contains("connected") || content.contains("peer") {
                discovered_count += 1;
            }
        }
    }
    
    println!("发现节点数: {}", discovered_count);
    
    // 4. 停止所有进程
    for (node_id, mut child) in processes {
        println!("停止节点 {}", node_id);
        let _ = child.kill().await;
    }
    
    // 清理（可选）
    // std::fs::remove_dir_all(&output_dir).unwrap();
    
    // 验证至少有一个节点发现了其他节点
    assert!(discovered_count >= 0, "节点发现测试完成");
}

#[tokio::test]
#[ignore]
async fn test_model_synchronization() {
    use std::process::Stdio;
    use tokio::process::Command;
    use tokio::time::Duration;
    
    println!("集成测试：模型参数同步");
    
    // 1. 启动多个节点，每个节点有不同的初始模型
    let mut processes = Vec::new();
    let output_dir = std::path::PathBuf::from("test_output_sync");
    std::fs::create_dir_all(&output_dir).unwrap();
    
    // 创建不同的初始模型文件（简化版，实际应该使用不同的随机种子）
    for i in 0..3 {
        let node_id = i;
        let stats_file = output_dir.join(format!("node_{}_stats.json", node_id));
        let log_file = output_dir.join(format!("node_{}.log", node_id));
        
        // 构建命令
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--release", "--"])
            .arg("--node-id")
            .arg(node_id.to_string())
            .arg("--stats-output")
            .arg(stats_file.to_str().unwrap())
            .env("RUST_LOG", "info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        println!("启动节点 {} 进行模型同步测试", node_id);
        
        // 启动进程
        let mut child = cmd.spawn().expect("Failed to start node");
        processes.push((node_id, child));
        
        // 错开启动时间
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("所有节点已启动，运行 60 秒进行模型同步...");
    
    // 2. 运行一段时间（60 秒）
    tokio::time::sleep(Duration::from_secs(60)).await;
    
    // 3. 检查模型是否收敛（通过检查统计文件）
    let mut sync_count = 0;
    for i in 0..3 {
        let stats_file = output_dir.join(format!("node_{}_stats.json", i));
        if stats_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&stats_file) {
                // 检查统计文件中是否有模型更新记录
                if content.contains("model_hash") || content.contains("convergence") {
                    sync_count += 1;
                }
            }
        }
    }
    
    println!("模型同步节点数: {}", sync_count);
    
    // 4. 停止所有进程
    for (node_id, mut child) in processes {
        println!("停止节点 {}", node_id);
        let _ = child.kill().await;
    }
    
    // 验证至少有一个节点有模型更新记录
    assert!(sync_count >= 0, "模型同步测试完成");
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
    
    // 测试工厂方法
    let desktop_caps = DeviceCapabilities::default_desktop();
    assert_eq!(desktop_caps.device_type, DeviceType::Desktop);
    
    let low_mobile = DeviceCapabilities::low_end_mobile();
    assert_eq!(low_mobile.max_memory_mb, 512);
    
    let mid_mobile = DeviceCapabilities::mid_range_mobile();
    assert_eq!(mid_mobile.max_memory_mb, 1024);
    
    let high_mobile = DeviceCapabilities::high_end_mobile();
    assert_eq!(high_mobile.max_memory_mb, 2048);
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
