//! 安全功能测试
//! 
//! 测试IP隐藏、中继连接、隐私保护等功能

use GGB::config::{AppConfig, SecurityConfig};
use GGB::security::{PrivacyChecker, TrafficObfuscator, IdentityProtector};
use libp2p::Multiaddr;
use std::path::Path;

/// 测试安全配置验证
#[test]
fn test_security_config_validation() {
    // 测试有效的安全配置
    let valid_config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![
            "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWTestRelay1".parse().unwrap(),
        ],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    assert!(valid_config.validate().is_ok());
    
    // 测试无效配置：隐藏IP但未使用中继
    let invalid_config = SecurityConfig {
        hide_ip: true,
        use_relay: false,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let result = invalid_config.validate();
    assert!(result.is_err());
    if let Err(errors) = result {
        assert!(errors.iter().any(|e| e.contains("隐私保护可能不完整")));
    }
    
    // 测试无效跳数
    let invalid_hops = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec!["/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWTestRelay1".parse().unwrap()],
        private_network_key: None,
        max_hops: 0, // 无效跳数
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let result = invalid_hops.validate();
    assert!(result.is_err());
}

/// 测试流量混淆
#[test]
fn test_traffic_obfuscation() {
    let config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let obfuscator = TrafficObfuscator::new(config);
    
    // 测试数据混淆
    let original_data = b"Hello, secure world!";
    let obfuscated = obfuscator.obfuscate_data(original_data);
    
    // 混淆后的数据应该比原始数据长
    assert!(obfuscated.len() > original_data.len());
    
    // 应该能够正确解混淆
    let deobfuscated = obfuscator.deobfuscate_data(&obfuscated, original_data.len());
    assert_eq!(deobfuscated, original_data);
}

/// 测试隐私检查器
#[test]
fn test_privacy_checker() {
    let config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let checker = PrivacyChecker::new(config);
    
    // 测试直接IP地址（应该暴露IP）
    let direct_ip: Multiaddr = "/ip4/192.168.1.100/tcp/9001".parse().unwrap();
    assert!(checker.is_address_exposing_ip(&direct_ip));
    
    // 测试中继地址（不应该暴露IP）
    let relay_addr: Multiaddr = "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWRelay/p2p-circuit/p2p/12D3KooWTarget".parse().unwrap();
    assert!(!checker.is_address_exposing_ip(&relay_addr));
    
    // 测试域名地址
    let dns_addr: Multiaddr = "/dns4/example.com/tcp/9001".parse().unwrap();
    assert!(!checker.is_address_exposing_ip(&dns_addr));
}

/// 测试身份保护
#[test]
fn test_identity_protection() {
    let config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let protector = IdentityProtector::new(config);
    
    // 获取第一个PeerId
    let peer_id1 = protector.get_current_peer_id();
    assert!(!peer_id1.to_string().is_empty());
    
    // 再次获取应该返回相同的PeerId（除非时间到了需要更换）
    let peer_id2 = protector.get_current_peer_id();
    assert_eq!(peer_id1, peer_id2);
    
    // 清理历史
    protector.cleanup_old_identities();
    
    // 获取历史
    let history = protector.get_identity_history();
    assert!(!history.is_empty());
}

/// 测试配置建议
#[test]
fn test_privacy_advice() {
    // 测试完整隐私配置
    let full_privacy = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec!["/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWRelay".parse().unwrap()],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    let advice = full_privacy.get_privacy_advice();
    assert!(advice.iter().any(|a| a.contains("IP隐藏已启用")));
    assert!(advice.iter().any(|a| a.contains("使用 1 个中继节点")));
    
    // 测试不完整隐私配置
    let partial_privacy = SecurityConfig {
        hide_ip: false,
        use_relay: false,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: true,
    };
    
    let advice = partial_privacy.get_privacy_advice();
    assert!(advice.iter().any(|a| a.contains("IP隐藏未启用")));
    assert!(advice.iter().any(|a| a.contains("建议")));
}

/// 测试安全工具函数
#[test]
fn test_security_utils() {
    use GGB::security::utils;
    
    // 测试中继地址创建
    let relay_addr: Multiaddr = "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWRelay".parse().unwrap();
    let target_peer_id = libp2p::PeerId::random();
    
    let secure_addr = utils::create_secure_address(&relay_addr, &target_peer_id);
    assert!(utils::is_relay_address(&secure_addr));
    
    // 测试目标PeerId提取
    let extracted = utils::extract_target_peer_id(&secure_addr);
    assert_eq!(extracted, Some(target_peer_id));
    
    // 测试非中继地址
    let direct_addr: Multiaddr = "/ip4/192.168.1.100/tcp/9001".parse().unwrap();
    assert!(!utils::is_relay_address(&direct_addr));
    assert_eq!(utils::extract_target_peer_id(&direct_addr), None);
}

/// 测试配置文件加载
#[test]
fn test_config_file_loading() {
    // 创建临时配置文件
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test_privacy_config.toml");
    
    let config_content = r#"
[security]
hide_ip = true
use_relay = true
relay_nodes = ["/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWTestRelay"]
max_hops = 3
enable_autonat = true
enable_dcutr = false

[comms]
topic = "ggb-test"
enable_dht = false
    "#;
    
    std::fs::write(&config_path, config_content).unwrap();
    
    // 测试配置文件加载
    let result = AppConfig::from_toml_file(&config_path);
    assert!(result.is_ok());
    
    // 清理临时文件
    std::fs::remove_file(&config_path).unwrap();
}

/// 集成测试：完整的隐私保护流程
#[test]
fn test_integrated_privacy_protection() {
    // 创建完整的安全配置
    let security_config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![
            "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWRelay1".parse().unwrap(),
            "/ip4/127.0.0.1/tcp/9002/p2p/12D3KooWRelay2".parse().unwrap(),
        ],
        private_network_key: Some("test-network-key".to_string()),
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
    };
    
    // 验证配置
    assert!(security_config.validate().is_ok());
    
    // 测试所有隐私组件
    let privacy_checker = PrivacyChecker::new(security_config.clone());
    let traffic_obfuscator = TrafficObfuscator::new(security_config.clone());
    let identity_protector = IdentityProtector::new(security_config);
    
    // 测试隐私建议
    let advice = privacy_checker.get_privacy_advice();
    assert!(!advice.is_empty());
    
    // 测试流量混淆
    let test_data = b"Test data for obfuscation";
    let obfuscated = traffic_obfuscator.obfuscate_data(test_data);
    assert!(obfuscated.len() >= test_data.len());
    
    // 测试身份保护
    let peer_id = identity_protector.get_current_peer_id();
    assert!(!peer_id.to_string().is_empty());
    
    println!("集成测试通过：");
    println!("  - 隐私建议: {:?}", advice);
    println!("  - 流量混淆: {} -> {} 字节", test_data.len(), obfuscated.len());
    println!("  - 身份保护: PeerId = {}", peer_id);
}
