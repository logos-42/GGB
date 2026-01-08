//! 隐私保护功能演示
//!
//! 展示如何使用GGB的IP隐藏和隐私保护功能（基于 iroh）

use GGB::config::{AppConfig, SecurityConfig};
use GGB::privacy::crypto::security::{PrivacyChecker, TrafficObfuscator, IdentityProtector, utils};
use iroh::NodeId;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GGB 隐私保护功能演示 (基于 iroh) ===\n");

    // 演示1：创建安全配置
    println!("1. 创建安全配置");
    let security_config = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![],
        private_network_key: Some("my-secret-network-key".to_string()),
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: false,
        privacy_performance: Default::default(),
    };

    // 验证配置
    match security_config.validate() {
        Ok(_) => println!("   ✓ 安全配置验证通过"),
        Err(errors) => {
            println!("   ⚠ 配置验证警告：");
            for error in errors {
                println!("     - {}", error);
            }
        }
    }

    // 显示隐私建议
    let advice = security_config.get_privacy_advice();
    println!("   隐私建议：");
    for item in advice {
        println!("     {}", item);
    }

    // 演示2：使用隐私检查器
    println!("\n2. 隐私检查器演示");
    let privacy_checker = PrivacyChecker::new(security_config.clone());

    let test_addresses = vec![
        "/ip4/192.168.1.100/tcp/9001".to_string(),
        "/dns4/example.com/tcp/9001".to_string(),
    ];

    for addr in test_addresses {
        let exposes_ip = privacy_checker.is_address_exposing_ip(&addr);
        println!("   {} -> {}", addr, if exposes_ip { "暴露IP" } else { "安全" });
    }

    // 演示3：流量混淆
    println!("\n3. 流量混淆演示");
    let traffic_obfuscator = TrafficObfuscator::new(security_config.clone());

    let sensitive_data = b"这是一条敏感消息，需要加密传输";
    println!("   原始数据: {} 字节", sensitive_data.len());

    let obfuscated = traffic_obfuscator.obfuscate_data(sensitive_data);
    println!("   混淆后数据: {} 字节", obfuscated.len());
    println!("   增加填充: {} 字节", obfuscated.len() - sensitive_data.len());

    let deobfuscated = traffic_obfuscator.deobfuscate_data(&obfuscated, sensitive_data.len());
    println!("   解混淆验证: {}",
        if deobfuscated == sensitive_data { "✓ 成功" } else { "✗ 失败" }
    );

    // 演示4：身份保护
    println!("\n4. 身份保护演示");
    let identity_protector = IdentityProtector::new(security_config.clone());

    let node_id1 = identity_protector.get_current_node_id();
    println!("   当前NodeId: {}", node_id1);

    // 模拟时间流逝（在实际应用中，身份会定期更换）
    println!("   身份历史:");
    let history = identity_protector.get_identity_history();
    for (i, (pid, time)) in history.iter().enumerate() {
        println!("     {}. {} (生成于: {:?})", i + 1, pid, time);
    }

    // 演示5：安全工具函数
    println!("\n5. 安全工具函数演示");

    let relay_addr = "/ip4/127.0.0.1/tcp/9001/p2p/12D3KooWRelay".to_string();
    println!("   中继地址: {}", relay_addr);
    println!("   是中继地址: {}", utils::is_relay_address(&relay_addr));

    let direct_addr = "/ip4/192.168.1.100/tcp/9001".to_string();
    println!("   直接地址: {}", direct_addr);
    println!("   是中继地址: {}", utils::is_relay_address(&direct_addr));

    // 演示6：配置验证
    println!("\n6. 完整配置验证");

    // 创建完整的应用配置
    let app_config = AppConfig::default();

    match app_config.validate() {
        Ok(_) => println!("   ✓ 应用配置验证通过"),
        Err(errors) => {
            println!("   ⚠ 配置验证问题：");
            for error in errors {
                println!("     - {}", error);
            }
        }
    }

    // 演示7：隐私模式对比
    println!("\n7. 隐私模式对比");

    let public_mode = SecurityConfig {
        hide_ip: false,
        use_relay: false,
        relay_nodes: vec![],
        private_network_key: None,
        max_hops: 3,
        enable_autonat: true,
        enable_dcutr: true,
        privacy_performance: Default::default(),
    };

    let private_mode = SecurityConfig {
        hide_ip: true,
        use_relay: true,
        relay_nodes: vec![],
        private_network_key: Some("secret".to_string()),
        max_hops: 3,
        enable_autonat: false,
        enable_dcutr: false,
        privacy_performance: Default::default(),
    };

    println!("   公共模式：");
    for advice in public_mode.get_privacy_advice() {
        println!("     {}", advice);
    }

    println!("\n   隐私模式：");
    for advice in private_mode.get_privacy_advice() {
        println!("     {}", advice);
    }

    // 演示8：Iroh NodeId 演示
    println!("\n8. Iroh NodeId 演示");
    let node_id = NodeId::from_bytes(rand::random::<[u8; 32]>());
    println!("   随机 NodeId: {}", node_id);
    println!("   NodeId 字节长度: {}", node_id.as_bytes().len());

    println!("\n=== 演示完成 ===");
    println!("\n使用建议：");
    println!("1. 生产环境中启用 hide_ip 和 use_relay");
    println!("2. 配置可靠的中继节点");
    println!("3. 定期更换中继节点以增强隐私");
    println!("4. 监控网络日志，确保没有IP泄露");
    println!("5. 使用配置文件管理安全设置");
    println!("6. 利用 iroh 的内建 NAT 穿透和中继功能");

    Ok(())
}
