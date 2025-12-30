//! 智能路由系统模块
//! 
//! 提供完整的智能路由功能，包括质量分析、路径选择和路由管理

pub mod quality;
pub mod selector;

// 重新导出常用类型
pub use quality::{
    ConnectionQuality, NetworkConditions, PerformanceTrend, 
    NetworkType, PerformanceMetric, NetworkImpact,
    QualityReport, QualityStatistics
};

pub use selector::{
    PrivacyPathSelector, PathSelectionStrategy, PathScore,
    MultiPathConfig, LoadBalanceStrategy, PathSelectionResult
};
