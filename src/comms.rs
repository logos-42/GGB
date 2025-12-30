//! 通信模块 - 负责节点间的网络通信
//! 
//! 该模块已拆分为多个子模块以提高可维护性：
//! - `comms::config`: 通信配置和带宽预算管理
//! - `comms::quic`: QUIC网关实现
//! - `comms::routing`: 智能路由系统
//! - `comms::libp2p`: libp2p网络行为定义
//! - `comms::handle`: 主通信句柄

pub mod comms;

// 重新导出主要类型，保持向后兼容
pub use comms::{
    CommsConfig, BandwidthBudgetConfig, 
    Behaviour, OutEvent,
    RouteType, RouteQuality, RouteScore, RouteInfo, RouteSelection, RoutingError,
    CommsHandle,
};