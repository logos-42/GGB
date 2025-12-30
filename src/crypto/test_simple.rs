//! 简单的crypto模块测试

use crate::crypto::base::{CryptoConfig, CryptoSuite};

fn main() {
    println!("测试基础加密功能...");
    
    // 测试基础加密套件
    let config = CryptoConfig::default();
    let suite = CryptoSuite::new(config).unwrap();
    
    let payload = b"Hello, Crypto!";
    let signature = suite.sign_bytes(payload).unwrap();
    
    println!("以太坊地址: {}", suite.eth_address());
    println!("Solana地址: {}", suite.sol_address());
    println!("签名验证: {}", suite.verify(payload, &signature));
    
    println!("测试完成!");
}
