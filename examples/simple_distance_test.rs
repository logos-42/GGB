//! 简单的网络距离感知功能测试
//!
//! 这个示例演示了 NetworkDistance 类型的基本功能

use williw::types::{NetworkDistance, DistanceLevel};

fn main() {
    println!("=== 简单网络距离感知功能测试 ===");
    
    // 测试 NetworkDistance 类型
    let mut network_distance = NetworkDistance::new();
    
    // 设置 DERP 节点延迟
    network_distance.relay_delays = vec![
        ("https://ny1.derp.iroh.network:443".to_string(), 45),
        ("https://sg1.derp.iroh.network:443".to_string(), 120),
        ("https://tokyo1.derp.iroh.network:443".to_string(), 200),
    ];
    
    // 设置端到端延迟
    network_distance.end_to_end_delay = Some(60);
    
    // 设置本地 DERP 延迟
    network_distance.local_derp_delays = vec![
        ("https://local1.derp.iroh.network:443".to_string(), 30),
        ("https://local2.derp.iroh.network:443".to_string(), 50),
    ];
    
    println!("网络距离: {:?}", network_distance);
    
    // 测试距离级别分类
    let distance_level = network_distance.distance_level();
    println!("距离级别: {:?}", distance_level);
    
    // 测试模糊描述
    let description = network_distance.get_distance_description();
    println!("模糊距离描述: {}", description);
    
    // 测试不同延迟值
    println!("\n测试不同延迟值:");
    let test_cases = vec![10, 50, 150, 350];
    
    for rtt in test_cases {
        let mut test_distance = NetworkDistance::new();
        test_distance.end_to_end_delay = Some(rtt);
        let level = test_distance.distance_level();
        let desc = test_distance.get_distance_description();
        println!("  RTT {}ms -> {:?} ({})", rtt, level, desc);
    }
    
    println!("\n=== 测试完成 ===");
}