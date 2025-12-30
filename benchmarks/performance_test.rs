//! 性能测试模块
//! 
//! 测试优化后的代码性能，包括加密、网络和设备检测。

use std::time::{Duration, Instant};
use std::sync::Arc;
use anyhow::Result;

use ggb::{
    privacy::{CryptoEngine, CryptoKey, EncryptionAlgorithm, PrivacyConfig},
    device::{DeviceDetector, DeviceCapabilities},
    core::config::{AppConfig, ConfigBuilder},
};

/// 性能测试结果
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub duration_ms: f64,
    pub throughput_mbps: f64,
    pub memory_usage_mb: f64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 性能测试套件
pub struct PerformanceTestSuite {
    results: Vec<PerformanceTestResult>,
}

impl PerformanceTestSuite {
    /// 创建新的测试套件
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    /// 运行所有测试
    pub fn run_all(&mut self) -> Result<()> {
        println!("开始性能测试...");
        
        self.run_crypto_test()?;
        self.run_device_detection_test()?;
        self.run_config_loading_test()?;
        self.run_memory_test()?;
        
        self.print_summary();
        Ok(())
    }
    
    /// 运行加密性能测试
    fn run_crypto_test(&mut self) -> Result<()> {
        let test_name = "加密性能测试";
        println!("运行 {}...", test_name);
        
        let start = Instant::now();
        
        // 测试不同算法的加密性能
        let algorithms = [
            EncryptionAlgorithm::ChaCha20Poly1305,
            EncryptionAlgorithm::Aes256Cbc,
            EncryptionAlgorithm::Blake3,
        ];
        
        let mut total_duration = Duration::new(0, 0);
        let mut total_data_size = 0;
        
        for algorithm in algorithms.iter() {
            match self.test_crypto_algorithm(*algorithm) {
                Ok((duration, data_size)) => {
                    total_duration += duration;
                    total_data_size += data_size;
                }
                Err(e) => {
                    self.results.push(PerformanceTestResult {
                        test_name: format!("{} - {:?}", test_name, algorithm),
                        duration_ms: 0.0,
                        throughput_mbps: 0.0,
                        memory_usage_mb: 0.0,
                        success: false,
                        error_message: Some(format!("{}", e)),
                    });
                }
            }
        }
        
        let duration_ms = total_duration.as_secs_f64() * 1000.0;
        let throughput_mbps = if total_duration.as_secs_f64() > 0.0 {
            (total_data_size as f64 / total_duration.as_secs_f64()) / (1024.0 * 1024.0)
        } else {
            0.0
        };
        
        self.results.push(PerformanceTestResult {
            test_name: test_name.to_string(),
            duration_ms,
            throughput_mbps,
            memory_usage_mb: Self::get_memory_usage(),
            success: true,
            error_message: None,
        });
        
        Ok(())
    }
    
    /// 测试特定加密算法
    fn test_crypto_algorithm(&self, algorithm: EncryptionAlgorithm) -> Result<(Duration, usize)> {
        let config = ggb::privacy::crypto::CryptoConfig {
            default_algorithm: algorithm,
            enable_hardware_acceleration: true,
            enable_batch_processing: true,
            enable_zero_copy: true,
            batch_size_bytes: 1024 * 1024,
        };
        
        let engine = ggb::privacy::crypto::CryptoEngine::new(config)?;
        let key = CryptoKey::new(algorithm);
        
        // 测试数据
        let test_data = vec![0u8; 1024 * 1024]; // 1MB 数据
        let iterations = 10;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let encrypted = engine.encrypt(&test_data, &key)?;
            let _decrypted = engine.decrypt(&encrypted, &key)?;
        }
        
        let duration = start.elapsed();
        let total_data_size = test_data.len() * iterations * 2; // 加密+解密
        
        Ok((duration, total_data_size))
    }
    
    /// 运行设备检测性能测试
    fn run_device_detection_test(&mut self) -> Result<()> {
        let test_name = "设备检测性能测试";
        println!("运行 {}...", test_name);
        
        let start = Instant::now();
        let iterations = 100;
        
        for _ in 0..iterations {
            let _capabilities = DeviceDetector::detect();
        }
        
        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let avg_duration_ms = duration_ms / iterations as f64;
        
        self.results.push(PerformanceTestResult {
            test_name: test_name.to_string(),
            duration_ms: avg_duration_ms,
            throughput_mbps: 0.0,
            memory_usage_mb: Self::get_memory_usage(),
            success: true,
            error_message: None,
        });
        
        Ok(())
    }
    
    /// 运行配置加载性能测试
    fn run_config_loading_test(&mut self) -> Result<()> {
        let test_name = "配置加载性能测试";
        println!("运行 {}...", test_name);
        
        // 创建测试配置
        let config = ConfigBuilder::new().build();
        let config_str = toml::to_string(&config)?;
        
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            let _: AppConfig = toml::from_str(&config_str)?;
        }
        
        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let avg_duration_ms = duration_ms / iterations as f64;
        
        self.results.push(PerformanceTestResult {
            test_name: test_name.to_string(),
            duration_ms: avg_duration_ms,
            throughput_mbps: 0.0,
            memory_usage_mb: Self::get_memory_usage(),
            success: true,
            error_message: None,
        });
        
        Ok(())
    }
    
    /// 运行内存性能测试
    fn run_memory_test(&mut self) -> Result<()> {
        let test_name = "内存性能测试";
        println!("运行 {}...", test_name);
        
        let start_memory = Self::get_memory_usage();
        let start = Instant::now();
        
        // 创建大量对象测试内存管理
        let mut objects = Vec::new();
        for i in 0..10000 {
            objects.push(format!("测试对象 {}", i));
        }
        
        // 模拟一些操作
        let processed: Vec<String> = objects.iter()
            .map(|s| s.to_uppercase())
            .collect();
        
        let _total_len: usize = processed.iter()
            .map(|s| s.len())
            .sum();
        
        let duration = start.elapsed();
        let end_memory = Self::get_memory_usage();
        let memory_increase = end_memory - start_memory;
        
        // 清理内存
        drop(objects);
        drop(processed);
        
        self.results.push(PerformanceTestResult {
            test_name: test_name.to_string(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            throughput_mbps: 0.0,
            memory_usage_mb: memory_increase,
            success: true,
            error_message: None,
        });
        
        Ok(())
    }
    
    /// 获取当前内存使用量（MB）
    fn get_memory_usage() -> f64 {
        #[cfg(target_os = "windows")]
        {
            use sysinfo::{System, SystemExt};
            let mut system = System::default();
            system.refresh_memory();
            system.used_memory() as f64 / 1024.0
        }
        #[cfg(not(target_os = "windows"))]
        {
            // 简化版本，实际项目中应该使用平台特定的内存检测
            0.0
        }
    }
    
    /// 打印测试摘要
    fn print_summary(&self) {
        println!("\n性能测试摘要:");
        println!("{:-<60}", "");
        println!("{:<30} {:<12} {:<12} {:<12} {:<8}", 
                 "测试名称", "耗时(ms)", "吞吐量(MB/s)", "内存(MB)", "状态");
        println!("{:-<60}", "");
        
        for result in &self.results {
            let status = if result.success { "✓" } else { "✗" };
            println!("{:<30} {:<12.2} {:<12.2} {:<12.2} {:<8}", 
                     result.test_name, 
                     result.duration_ms,
                     result.throughput_mbps,
                     result.memory_usage_mb,
                     status);
            
            if let Some(ref error) = result.error_message {
                println!("  错误: {}", error);
            }
        }
        println!("{:-<60}", "");
        
        // 计算总体统计
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let total_tests = self.results.len();
        let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
        
        println!("总计: {}/{} 测试通过 ({:.1}%)", 
                 successful_tests, total_tests, success_rate);
    }
    
    /// 获取测试结果
    pub fn get_results(&self) -> &[PerformanceTestResult] {
        &self.results
    }
    
    /// 生成测试报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# 性能测试报告\n\n");
        
        for result in &self.results {
            report.push_str(&format!("## {}\n", result.test_name));
            report.push_str(&format!("- 状态: {}\n", if result.success { "通过" } else { "失败" }));
            report.push_str(&format!("- 耗时: {:.2} ms\n", result.duration_ms));
            
            if result.throughput_mbps > 0.0 {
                report.push_str(&format!("- 吞吐量: {:.2} MB/s\n", result.throughput_mbps));
            }
            
            report.push_str(&format!("- 内存使用: {:.2} MB\n", result.memory_usage_mb));
            
            if let Some(ref error) = result.error_message {
                report.push_str(&format!("- 错误: {}\n", error));
            }
            
            report.push_str("\n");
        }
        
        report
    }
}

fn main() -> Result<()> {
    println!("GGB 性能测试");
    println!("============\n");
    
    let mut test_suite = PerformanceTestSuite::new();
    
    match test_suite.run_all() {
        Ok(_) => {
            // 保存测试报告
            let report = test_suite.generate_report();
            std::fs::write("performance_report.md", &report)?;
            println!("\n测试报告已保存到 performance_report.md");
        }
        Err(e) => {
            eprintln!("测试失败: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
