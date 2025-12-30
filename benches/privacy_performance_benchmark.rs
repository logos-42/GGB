//! 隐私性能平衡基准测试
//! 
//! 测试隐私性能平衡方案的性能指标

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

use ggb::config::{PrivacyPerformanceConfig, BalanceMode};
use ggb::routing::{ConnectionQualityAnalyzer, PrivacyPathSelector};
use ggb::quic::PrivacyOverlay;

/// 基准测试：连接质量分析器性能
fn bench_connection_quality_analyzer(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_quality_analyzer");
    group.measurement_time(Duration::from_secs(10));
    
    // 测试不同窗口大小的性能
    for window_size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("update_quality", window_size),
            window_size,
            |b, &size| {
                let analyzer = ConnectionQualityAnalyzer::new(size);
                let quality = ggb::routing::ConnectionQuality {
                    latency_ms: 50.0,
                    bandwidth_mbps: 100.0,
                    packet_loss_percent: 0.1,
                    jitter_ms: 5.0,
                    reliability: 0.95,
                    stability: 0.9,
                    last_updated: std::time::Instant::now(),
                };
                
                b.iter(|| {
                    analyzer.update_quality(quality.clone());
                });
            },
        );
    }
    
    // 测试趋势分析性能
    group.bench_function("analyze_performance_trend", |b| {
        let analyzer = ConnectionQualityAnalyzer::new(100);
        
        // 填充一些测试数据
        for i in 0..100 {
            let quality = ggb::routing::ConnectionQuality {
                latency_ms: 50.0 + (i as f32 * 0.1),
                bandwidth_mbps: 100.0 - (i as f32 * 0.1),
                packet_loss_percent: 0.1,
                jitter_ms: 5.0,
                reliability: 0.95,
                stability: 0.9,
                last_updated: std::time::Instant::now(),
            };
            analyzer.update_quality(quality);
        }
        
        b.iter(|| {
            analyzer.analyze_performance_trend(ggb::routing::PerformanceMetric::Latency);
        });
    });
    
    group.finish();
}

/// 基准测试：隐私路径选择器性能
fn bench_privacy_path_selector(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_path_selector");
    group.measurement_time(Duration::from_secs(10));
    
    // 测试不同配置模式的性能
    let modes = [
        (BalanceMode::Performance, "performance"),
        (BalanceMode::Balanced, "balanced"),
        (BalanceMode::Privacy, "privacy"),
        (BalanceMode::Adaptive, "adaptive"),
    ];
    
    for (mode, mode_name) in modes.iter() {
        group.bench_with_input(
            BenchmarkId::new("select_best_path", mode_name),
            mode,
            |b, &mode| {
                let config = PrivacyPerformanceConfig {
                    mode,
                    performance_weight: match mode {
                        BalanceMode::Performance => 0.8,
                        BalanceMode::Balanced => 0.6,
                        BalanceMode::Privacy => 0.3,
                        BalanceMode::Adaptive => 0.6,
                    },
                    enable_hardware_acceleration: true,
                    connection_pool_size: 10,
                    enable_0rtt: true,
                    congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                    routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                    min_privacy_score: 0.7,
                    min_performance_score: 0.8,
                    fallback_to_direct: true,
                    monitoring_interval_secs: 30,
                };
                
                let selector = PrivacyPathSelector::new(config);
                
                b.iter(|| {
                    // 注意：这里需要先添加测试路径
                    // 在实际基准测试中，应该设置测试数据
                    let _ = selector.select_best_path("test_target");
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：隐私覆盖层性能
fn bench_privacy_overlay(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_overlay");
    group.measurement_time(Duration::from_secs(10));
    
    // 测试不同数据大小的加密性能
    for data_size in [64, 256, 1024, 4096, 16384].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_outbound", data_size),
            data_size,
            |b, &size| {
                let config = PrivacyPerformanceConfig {
                    mode: BalanceMode::Balanced,
                    performance_weight: 0.6,
                    enable_hardware_acceleration: true,
                    connection_pool_size: 10,
                    enable_0rtt: true,
                    congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                    routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                    min_privacy_score: 0.7,
                    min_performance_score: 0.8,
                    fallback_to_direct: true,
                    monitoring_interval_secs: 30,
                };
                
                let overlay = PrivacyOverlay::new(config).unwrap();
                let test_data = vec![0u8; size];
                
                b.iter(|| {
                    // 注意：这里需要异步运行时
                    // 在实际基准测试中，应该使用tokio::runtime
                    let _ = overlay.process_outbound(&test_data);
                });
            },
        );
    }
    
    // 测试不同隐私模式的性能
    let modes = [
        (BalanceMode::Performance, "performance"),
        (BalanceMode::Balanced, "balanced"),
        (BalanceMode::Privacy, "privacy"),
    ];
    
    for (mode, mode_name) in modes.iter() {
        group.bench_with_input(
            BenchmarkId::new("encryption_overhead", mode_name),
            mode,
            |b, &mode| {
                let config = PrivacyPerformanceConfig {
                    mode,
                    performance_weight: match mode {
                        BalanceMode::Performance => 0.8,
                        BalanceMode::Balanced => 0.6,
                        BalanceMode::Privacy => 0.3,
                        BalanceMode::Adaptive => 0.6,
                    },
                    enable_hardware_acceleration: true,
                    connection_pool_size: 10,
                    enable_0rtt: true,
                    congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                    routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                    min_privacy_score: match mode {
                        BalanceMode::Performance => 0.5,
                        BalanceMode::Balanced => 0.7,
                        BalanceMode::Privacy => 0.9,
                        BalanceMode::Adaptive => 0.7,
                    },
                    min_performance_score: 0.8,
                    fallback_to_direct: true,
                    monitoring_interval_secs: 30,
                };
                
                let overlay = PrivacyOverlay::new(config).unwrap();
                let test_data = vec![0u8; 1024]; // 1KB测试数据
                
                b.iter(|| {
                    let _ = overlay.process_outbound(&test_data);
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：路由评分计算性能
fn bench_route_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("route_scoring");
    group.measurement_time(Duration::from_secs(10));
    
    // 测试不同路径数量的评分性能
    for path_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("score_paths", path_count),
            path_count,
            |b, &count| {
                let config = PrivacyPerformanceConfig {
                    mode: BalanceMode::Balanced,
                    performance_weight: 0.6,
                    enable_hardware_acceleration: true,
                    connection_pool_size: 10,
                    enable_0rtt: true,
                    congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                    routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                    min_privacy_score: 0.7,
                    min_performance_score: 0.8,
                    fallback_to_direct: true,
                    monitoring_interval_secs: 30,
                };
                
                let selector = PrivacyPathSelector::new(config);
                
                b.iter(|| {
                    // 在实际基准测试中，这里应该创建测试路径并计算评分
                    // 目前是占位符
                });
            },
        );
    }
    
    group.finish();
}

/// 基准测试：内存使用情况
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(5));
    
    // 测试连接质量分析器的内存使用
    group.bench_function("connection_quality_analyzer_memory", |b| {
        b.iter(|| {
            let analyzer = ConnectionQualityAnalyzer::new(1000);
            // 填充大量数据
            for i in 0..1000 {
                let quality = ggb::routing::ConnectionQuality {
                    latency_ms: 50.0,
                    bandwidth_mbps: 100.0,
                    packet_loss_percent: 0.1,
                    jitter_ms: 5.0,
                    reliability: 0.95,
                    stability: 0.9,
                    last_updated: std::time::Instant::now(),
                };
                analyzer.update_quality(quality);
            }
        });
    });
    
    // 测试隐私路径选择器的内存使用
    group.bench_function("privacy_path_selector_memory", |b| {
        b.iter(|| {
            let config = PrivacyPerformanceConfig {
                mode: BalanceMode::Balanced,
                performance_weight: 0.6,
                enable_hardware_acceleration: true,
                connection_pool_size: 100,
                enable_0rtt: true,
                congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                min_privacy_score: 0.7,
                min_performance_score: 0.8,
                fallback_to_direct: true,
                monitoring_interval_secs: 30,
            };
            
            let _selector = PrivacyPathSelector::new(config);
        });
    });
    
    group.finish();
}

/// 基准测试：并发性能
fn bench_concurrent_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_performance");
    group.measurement_time(Duration::from_secs(10));
    
    // 测试多线程下的连接质量分析
    group.bench_function("concurrent_quality_analysis", |b| {
        use std::sync::Arc;
        use std::thread;
        
        b.iter(|| {
            let analyzer = Arc::new(ConnectionQualityAnalyzer::new(100));
            let mut handles = vec![];
            
            for _ in 0..10 {
                let analyzer_clone = analyzer.clone();
                let handle = thread::spawn(move || {
                    for i in 0..100 {
                        let quality = ggb::routing::ConnectionQuality {
                            latency_ms: 50.0 + (i as f32 * 0.1),
                            bandwidth_mbps: 100.0,
                            packet_loss_percent: 0.1,
                            jitter_ms: 5.0,
                            reliability: 0.95,
                            stability: 0.9,
                            last_updated: std::time::Instant::now(),
                        };
                        analyzer_clone.update_quality(quality);
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

/// 基准测试：端到端性能
fn bench_end_to_end_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_performance");
    group.measurement_time(Duration::from_secs(15));
    
    // 测试完整工作流程的性能
    group.bench_function("complete_workflow", |b| {
        b.iter(|| {
            // 1. 创建配置
            let config = PrivacyPerformanceConfig {
                mode: BalanceMode::Balanced,
                performance_weight: 0.6,
                enable_hardware_acceleration: true,
                connection_pool_size: 10,
                enable_0rtt: true,
                congestion_control: ggb::config::CongestionControlAlgorithm::Bbr,
                routing_strategy: ggb::config::RoutingStrategy::SmartBalance,
                min_privacy_score: 0.7,
                min_performance_score: 0.8,
                fallback_to_direct: true,
                monitoring_interval_secs: 30,
            };
            
            // 2. 创建各个组件
            let analyzer = ConnectionQualityAnalyzer::new(100);
            let selector = PrivacyPathSelector::new(config.clone());
            let overlay = PrivacyOverlay::new(config).unwrap();
            
            // 3. 模拟工作流程
            // 更新连接质量
            let quality = ggb::routing::ConnectionQuality {
                latency_ms: 50.0,
                bandwidth_mbps: 100.0,
                packet_loss_percent: 0.1,
                jitter_ms: 5.0,
                reliability: 0.95,
                stability: 0.9,
                last_updated: std::time::Instant::now(),
            };
            analyzer.update_quality(quality);
            
            // 分析趋势
            let _trend = analyzer.analyze_performance_trend(ggb::routing::PerformanceMetric::Latency);
            
            // 处理数据
            let test_data = b"Test data for end-to-end benchmark";
            let _processed = overlay.process_outbound(test_data);
            
            // 选择路径
            let _path = selector.select_best_path("test_target");
        });
    });
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10));
    targets = 
        bench_connection_quality_analyzer,
        bench_privacy_path_selector,
        bench_privacy_overlay,
        bench_route_scoring,
        bench_memory_usage,
        bench_concurrent_performance,
        bench_end_to_end_performance
);

criterion_main!(benches);
